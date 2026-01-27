use std::{fs, io::Write, ops::RangeInclusive, path::Path};

use anyhow::{anyhow, Result};
use image::{codecs::png::PngEncoder, imageops, GenericImage, ImageEncoder, RgbaImage, SubImage};
use rand::{prelude::*, rng};
use libks::map_bin::{LayerData, ScreenData, Tile};
use libks_ini::{Ini, VirtualSection};

use crate::{
    definitions::{AnimSync, DrawParams, ObjectDef, ObjectDefs, ObjectId, ObjectKind, ObjectVariant, SyncParams},
    graphics::Graphics,
    partition::{Bounds, Partition},
    screen_map::ScreenMap,
    synchronization::{ScreenSync, WorldSync},
    timespan::Timespan,
};

mod blend_modes;
pub use blend_modes::BlendMode;

pub fn tileset_index_to_pixels(i: u8) -> (u32, u32) {
    (
        (i as u32 % 16) * 24,
        (i as u32 / 16) * 24,
    )
}

pub fn screen_index_to_pixels(i: u8) -> (i64, i64) {
    (
        (i as i64 % 25) * 24,
        (i as i64 / 25) * 24,
    )
}

struct DrawContext<'a> {
    image: RgbaImage,
    tileset_a: Option<&'a RgbaImage>,
    tileset_b: Option<&'a RgbaImage>,
    gfx: &'a Graphics<'a>,
    defs: &'a ObjectDefs,
    ini_section: Option<VirtualSection<'a>>,
    sync: ScreenSync,
    opts: &'a DrawOptions,
}

pub struct DrawOptions {
    pub editor_only: bool,
}

#[derive(Debug, Clone)]
struct Cursor<'a> {
    i: usize,
    actual_id: ObjectId,
    proxy_id: ObjectId,
    object_def: Option<&'a ObjectDef>,
}

pub fn draw_partitions(
    screens: &ScreenMap,
    partitions: &[Partition],
    gfx: &Graphics,
    defs: &ObjectDefs,
    ini: &Ini,
    output_dir: impl AsRef<Path>,
    options: &DrawOptions,
    world_sync: &WorldSync,
) -> Result<()> {
    for partition in partitions {        
        let bounds = partition.bounds();
        println!("{bounds}");
        let Ok(mut canvas) = make_canvas(&bounds) else { continue };

        let mut span_draw = Timespan::begin();
        print!("    Drawing screens");
        let _ = std::io::stdout().flush();
        for pos in partition {
            let Some(screen) = screens.get(pos) else { continue };
            match draw_screen(screen, gfx, defs, ini, options, world_sync) {
                Ok(screen_image) => {
                    let canvas_x: u32 = ((screen.position.0 as i64 - bounds.x.start) * 600).try_into().unwrap();
                    let canvas_y: u32 = ((screen.position.1 as i64 - bounds.y.start) * 240).try_into().unwrap();
                    canvas.copy_from(&screen_image, canvas_x, canvas_y)?;
                },
                Err(err) => {
                    eprintln!("    Error on x{}y{}: {err}", screen.position.0, screen.position.1);
                },
            }
        }
        span_draw.end();
        println!(" [{span_draw}]");

        let mut span_export = Timespan::begin();
        print!("    Saving canvas to disk");
        let _ = std::io::stdout().flush();
        let file_name = format!("{bounds}.png");
        let path = output_dir.as_ref().join(file_name);
        export_canvas_multithreaded(canvas, &path)?;
        span_export.end();
        println!(" [{span_export}]\n");
    }

    Ok(())
}

fn make_canvas(bounds: &Bounds) -> Result<RgbaImage> {
    let (width, height) = bounds.size();

    let Ok(Some(width)) = u32::try_from(width)
        .map(|width| width.checked_mul(600))
    else {
        return Err(anyhow!("Partition is too large: {bounds}"));
    };

    let Ok(Some(height)) = u32::try_from(height)
        .map(|height| height.checked_mul(240))
    else {
        return Err(anyhow!("Partition {bounds} is too large"));
    };
    
    Ok(RgbaImage::new(width, height))
}

fn export_canvas(canvas: RgbaImage, path: &Path) -> Result<()> {
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;

    let encoder = PngEncoder::new_with_quality(
        file,
        image::codecs::png::CompressionType::Best,
        Default::default(),
    );

    let width = canvas.width();
    let height = canvas.height();
    let buf = canvas.into_vec();

    encoder.write_image(&buf, width, height, image::ExtendedColorType::Rgba8)?;

    Ok(())
}

fn export_canvas_multithreaded(canvas: RgbaImage, path: &Path) -> Result<()> {
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    let writer = std::io::BufWriter::new(file);
    
    let width = canvas.width();
    let height = canvas.height();
    let data = canvas.into_vec();
    
    let mut header = mtpng::Header::new();
    header.set_size(width, height)?;
    header.set_color(mtpng::ColorType::TruecolorAlpha, 8)?;
    
    let mut options = mtpng::encoder::Options::new();
    options.set_compression_level(mtpng::CompressionLevel::High)?;

    let mut encoder = mtpng::encoder::Encoder::new(writer, &options);
    encoder.write_header(&header)?;
    encoder.write_image_rows(&data)?;
    encoder.finish()?;

    Ok(())
}

pub fn draw_screen(
    screen: &ScreenData,
    gfx: &Graphics,
    defs: &ObjectDefs,
    ini: &Ini,
    options: &DrawOptions,
    world_sync: &WorldSync,
) -> Result<RgbaImage> {
    let ini_section = ini.section(&format!("x{}y{}", screen.position.0, screen.position.1));
    let is_overlay = ini_section
        .as_ref()
        .is_some_and(|section| {
            section.get("Overlay")
                .unwrap_or("")
                .eq_ignore_ascii_case("True")
        });

    // Create context
    let group_anim_t = world_sync.group_anim_ts.get(&screen.position).cloned();
    let sync = ScreenSync::new(screen, defs, group_anim_t);
    let mut ctx = DrawContext {
        image: RgbaImage::new(600, 240),
        tileset_a: gfx.tileset(screen.assets.tileset_a),
        tileset_b: gfx.tileset(screen.assets.tileset_b),
        gfx,
        defs,
        ini_section,
        sync,
        opts: options,
    };
    
    // Draw gradient
    if let Some(gradient) = ctx.gfx.gradient(screen.assets.gradient) {
        imageops::tile(&mut ctx.image, gradient);
    }
    
    // Draw tile layers
    draw_tile_layer(&mut ctx, &screen.layers[0]);
    draw_tile_layer(&mut ctx, &screen.layers[1]);
    if !is_overlay {
        draw_tile_layer(&mut ctx, &screen.layers[2]);
    }
    draw_tile_layer(&mut ctx, &screen.layers[3]);

    // Draw object layers
    draw_object_layer(&mut ctx, &screen.layers[4]);
    draw_object_layer(&mut ctx, &screen.layers[5]);
    draw_object_layer(&mut ctx, &screen.layers[6]);
    if is_overlay {
        draw_tile_layer(&mut ctx, &screen.layers[2]);
    }
    draw_object_layer(&mut ctx, &screen.layers[7]);

    Ok(ctx.image)
}

fn draw_tile_layer(ctx: &mut DrawContext, layer: &LayerData) {
    for (i, tile) in layer.0.iter().enumerate() {
        if tile.1 == 0 {
            continue;
        }

        let Some(tileset) = (match tile.0 {
            0 => ctx.tileset_a,
            1 => ctx.tileset_b,
            _ => None,
        }) else {
            continue;
        };

        let (tile_x, tile_y) = tileset_index_to_pixels(tile.1);        
        let (screen_x, screen_y) = screen_index_to_pixels(i as u8);
        
        let tile_img = imageops::crop_imm(tileset, tile_x, tile_y, 24, 24);
        imageops::overlay(&mut ctx.image, &*tile_img, screen_x, screen_y);
    }
}

fn draw_object_layer(ctx: &mut DrawContext, layer: &LayerData) {
    for (i, tile) in layer.0.iter().enumerate() {
        if tile.1 == 0 { continue }

        let actual_id = ObjectId::from(tile);
        let object_def = ctx.defs.get(&actual_id);
        let proxy_id = match object_def.map(|def| &def.kind) {
            Some(ObjectKind::OverrideObject(tile)) => ObjectId::from(tile),
            _ => ObjectId::from(tile),
        };
        let curs = Cursor {
            i,
            actual_id,
            proxy_id,
            object_def,
        };

        if ctx.sync.limiters.get_mut(&curs.actual_id)
            .is_some_and(|limiter| !limiter.increment())
        {
            continue;
        }
        if !ctx.opts.editor_only
            && object_def.is_some_and(|object| object.is_editor_only)
        {
            continue;
        }

        match curs.proxy_id.0 {
            Tile(0, 14) => draw_shift(ctx, curs, "ShiftVisible(A)", "ShiftType(A)"),
            Tile(0, 15) => draw_shift(ctx, curs, "ShiftVisible(B)", "ShiftType(B)"),
            Tile(0, 16) => draw_shift(ctx, curs, "ShiftVisible(C)", "ShiftType(C)"),
            Tile(0, 32) => draw_shift(ctx, curs, "TrigVisible(A)", "TrigType(A)"),
            Tile(0, 33) => draw_shift(ctx, curs, "TrigVisible(B)", "TrigType(B)"),
            Tile(0, 34) => draw_shift(ctx, curs, "TrigVisible(C)", "TrigType(C)"),
            Tile(1, 5 | 10 | 12 | 22) => draw_with_glow(ctx, curs),
            Tile(2, 18 | 19) => draw_elemental(ctx, curs),
            Tile(8, 10) => draw_with_random_offset(ctx, curs, -6..=6),
            Tile(8, 15) => draw_with_random_offset(ctx, curs, -12..=12),
            _ => draw_object(ctx, curs.i, curs.actual_id),
        }
    }
}

#[inline]
fn draw_object(ctx: &mut DrawContext, at_index: usize, object: ObjectId) {
    let (draw_params, sync_params) = match ctx.defs.get(&object) {
        Some(def) => (&def.draw_params, &def.sync_params),
        None => (&DrawParams::default(), &SyncParams::default()),
    };
    draw_object_with_params(ctx, at_index, object, draw_params, sync_params);
}

#[inline]
fn draw_object_with_params(
    ctx: &mut DrawContext,
    at_index: usize,
    object: ObjectId,
    draw_params: &DrawParams,
    sync_params: &SyncParams,
) {
    let Some(obj_image) = ctx.gfx.object(&object) else { return };
    let anim_t = match sync_params.sync_to {
        AnimSync::None => None,
        AnimSync::Screen => Some(ctx.sync.anim_t),
        AnimSync::Group => ctx.sync.group_anim_t.or(Some(ctx.sync.anim_t)),
    };
    draw_spritesheet(ctx, at_index as u8, draw_params, anim_t, obj_image);
}

fn draw_spritesheet(ctx: &mut DrawContext, at_index: u8, params: &DrawParams, anim_t: Option<u32>, obj_img: &RgbaImage) {
    let frame = pick_frame(&obj_img, params, anim_t);
    let (screen_x, screen_y) = screen_index_to_pixels(at_index);
    let (offset_x, offset_y) = params.offset.unwrap_or_default();

    let (final_x, final_y) = match params.frame_size {
        Some((frame_width, frame_height)) => (
            screen_x + 12 - (frame_width as i64 / 2) + offset_x,
            screen_y + 12 - (frame_height as i64 / 2) + offset_y,
        ),
        None => (
            screen_x + offset_x,
            screen_y + offset_y,
        ),
    };

    if let Some(alpha_range) = params.alpha_range.as_ref() {
        let alpha = rng().random_range(alpha_range.clone()) as f32 / 255.0;
        blend_modes::overlay_with_alpha(&mut ctx.image, &*frame, final_x, final_y, params.blend_mode, alpha);
    }
    else {
        blend_modes::overlay(&mut ctx.image, &*frame, final_x, final_y, params.blend_mode);
    }
}

fn pick_frame<'a>(object_img: &'a RgbaImage, params: &DrawParams, anim_t: Option<u32>) -> SubImage<&'a RgbaImage> {
    let size = object_img.dimensions();
    let (frame_width, frame_height) = params.frame_size.unwrap_or((24, 24));
    let frames_per_row = u32::max(1, size.0 / frame_width);
    let n_rows = u32::max(1, size.1 / frame_height);
    let n_frames_max = n_rows * frames_per_row;
    let mut frame_range = params.frame_range.clone().unwrap_or_else(|| {
        let n_frames = n_rows * frames_per_row;
        0..n_frames
    });
    frame_range.end = u32::min(n_frames_max, frame_range.end);

    let frame = 
        if frame_range.is_empty() {
            0
        }
        else if let Some(anim_t) = anim_t {
            let n_frames = frame_range.end - frame_range.start;
            (anim_t % n_frames) + frame_range.start
        }
        else {
            rng().random_range(frame_range)
        };

    let frame_x = (frame % frames_per_row) * frame_width;
    let frame_y = (frame / frames_per_row) * frame_height;

    imageops::crop_imm(object_img, frame_x, frame_y, frame_width, frame_height)
}

fn draw_shift(ctx: &mut DrawContext, curs: Cursor, vis_prop: &str, type_prop: &str) {
    let shift_visible = !ctx.ini_section
        .as_ref()
        .and_then(|section| section.get(vis_prop))
        .unwrap_or("True")
        .eq_ignore_ascii_case("False");

    if !shift_visible {
        return;
    }

    let shift_type = match ctx.ini_section
        .as_ref()
        .and_then(|section| section.get(type_prop))
        .unwrap_or("0")
    {
        "0" => ObjectVariant::Spot,
        "1" => ObjectVariant::Floor,
        "2" => ObjectVariant::Circle,
        "3" => ObjectVariant::Square,
        _ => ObjectVariant::Spot,
    };

    draw_object(ctx, curs.i, curs.proxy_id.into_variant(shift_type));
}

fn draw_with_glow(ctx: &mut DrawContext, curs: Cursor) {
    draw_object(ctx, curs.i, curs.proxy_id.to_variant(ObjectVariant::Glow));
    draw_object(ctx, curs.i, curs.actual_id);
}

fn draw_elemental(ctx: &mut DrawContext, curs: Cursor) {
    let mut rng = rng();
    let variant = [ObjectVariant::A, ObjectVariant::B, ObjectVariant::C, ObjectVariant::D]
        .choose(&mut rng)
        .unwrap();

    draw_object(ctx, curs.i, curs.proxy_id.into_variant(*variant));
}

fn draw_with_random_offset(ctx: &mut DrawContext, curs: Cursor, range: RangeInclusive<i64>) {
    let mut rng = rng();
    let offset_x = rng.random_range(range.clone());
    let offset_y = rng.random_range(range);

    let (mut draw_params, sync_params) = match curs.object_def {
        Some(def) => (def.draw_params.clone(), &def.sync_params),
        None => (DrawParams::default(), &SyncParams::default()),
    };
    draw_params.offset = Some((offset_x, offset_y));

    draw_object_with_params(ctx, curs.i, curs.actual_id, &draw_params, sync_params);
}
