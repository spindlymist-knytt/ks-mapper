use std::{fs, path::Path, rc::Rc};

use anyhow::{anyhow, Result};
use image::{codecs::png::PngEncoder, imageops, GenericImage, ImageEncoder, RgbaImage, SubImage};
use rand::{thread_rng, Rng};
use libks::map_bin::{LayerData, ScreenData, Tile};
use libks_ini::{Ini, VirtualSection};

use crate::{
    definitions::{DrawParams, ObjectId, ObjectKind},
    graphics::GraphicsLoader,
    partition::{Bounds, Partition},
    screen_map::ScreenMap,
    synchronization::ScreenSync,
};

mod blend_modes;
pub use blend_modes::BlendMode;

mod bank0;
mod bank1;
mod bank2;
mod bank8;

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
    tileset_a: Option<Rc<RgbaImage>>,
    tileset_b: Option<Rc<RgbaImage>>,
    gfx: &'a mut GraphicsLoader,
    ini_section: Option<VirtualSection<'a>>,
    sync: ScreenSync,
    opts: &'a DrawOptions,
}

pub struct DrawOptions {
    pub editor_only: bool,
}

#[derive(Debug, Clone)]
struct Cursor {
    i: usize,
    actual_id: ObjectId,
    proxy_id: ObjectId,
    // tile: Tile,
    // variant: Option<String>,
}

pub fn draw_partitions(
    screens: &ScreenMap,
    partitions: &[Partition],
    gfx: &mut GraphicsLoader,
    ini: &Ini,
    output_dir: impl AsRef<Path>,
    options: &DrawOptions,
) -> Result<()> {
    for partition in partitions {
        let bounds = partition.bounds();

        println!("{bounds}");
        println!("    Allocating canvas");
        
        let Ok(mut canvas) = make_canvas(&bounds) else { continue };

        println!("    Drawing screens");

        for pos in partition {
            let Some(screen) = screens.get(pos) else { continue };
            match draw_screen(screen, gfx, ini, options) {
                Ok(screen_image) => {
                    let canvas_x: u32 = ((screen.position.0 - bounds.left()) * 600).try_into().unwrap();
                    let canvas_y: u32 = ((screen.position.1 - bounds.top()) * 240).try_into().unwrap();
                    canvas.copy_from(&screen_image, canvas_x, canvas_y)?;
                },
                Err(err) => {
                    eprintln!("    Error on x{}y{}: {err}", screen.position.0, screen.position.1);
                },
            }
        }

        println!("    Saving canvas to disk");

        let file_name = format!("{bounds}.png");
        let path = output_dir.as_ref().join(file_name);
        export_canvas(canvas, &path)?;

        println!();
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

pub fn draw_screen(screen: &ScreenData, gfx: &mut GraphicsLoader, ini: &Ini, options: &DrawOptions) -> Result<RgbaImage> {
    let ini_section = ini.section(&format!("x{}y{}", screen.position.0, screen.position.1));
    let is_overlay = ini_section
        .as_ref()
        .is_some_and(|section| {
            section.get("Overlay")
                .unwrap_or("")
                .eq_ignore_ascii_case("True")
        });

    // Create context
    let sync = ScreenSync::new(screen, gfx.object_defs());
    let mut ctx = DrawContext {
        image: RgbaImage::new(600, 240),
        tileset_a: gfx.tileset(screen.assets.tileset_a)?,
        tileset_b: gfx.tileset(screen.assets.tileset_b)?,
        gfx,
        ini_section,
        sync,
        opts: options,
    };
    
    // Draw gradient
    if let Some(gradient) = ctx.gfx.gradient(screen.assets.gradient)? {
        imageops::tile(&mut ctx.image, gradient.as_ref());
    }
    
    // Draw tile layers
    draw_tile_layer(&mut ctx, &screen.layers[0]);
    draw_tile_layer(&mut ctx, &screen.layers[1]);
    if !is_overlay {
        draw_tile_layer(&mut ctx, &screen.layers[2]);
    }
    draw_tile_layer(&mut ctx, &screen.layers[3]);

    // Draw object layers
    draw_object_layer(&mut ctx, &screen.layers[4])?;
    draw_object_layer(&mut ctx, &screen.layers[5])?;
    draw_object_layer(&mut ctx, &screen.layers[6])?;
    if is_overlay {
        draw_tile_layer(&mut ctx, &screen.layers[2]);
    }
    draw_object_layer(&mut ctx, &screen.layers[7])?;

    Ok(ctx.image)
}

fn draw_tile_layer(ctx: &mut DrawContext, layer: &LayerData) {
    for (i, tile) in layer.0.iter().enumerate() {
        if tile.1 == 0 {
            continue;
        }

        let Some(tileset) = (match tile.0 {
            0 => ctx.tileset_a.as_ref(),
            1 => ctx.tileset_b.as_ref(),
            _ => None,
        }) else {
            continue;
        };

        let (tile_x, tile_y) = tileset_index_to_pixels(tile.1);        
        let (screen_x, screen_y) = screen_index_to_pixels(i as u8);
        
        let tile_img = imageops::crop_imm(tileset.as_ref(), tile_x, tile_y, 24, 24);
        imageops::overlay(&mut ctx.image, &*tile_img, screen_x, screen_y);
    }
}

fn draw_object_layer(ctx: &mut DrawContext, layer: &LayerData) -> Result<()> {
    for (i, tile) in layer.0.iter().enumerate() {
        if tile.1 == 0 {
            continue;
        }

        let actual_id = ObjectId(*tile, None);

        if ctx.sync.limiters.get_mut(&actual_id)
            .is_some_and(|limiter| !limiter.increment())
        {
            continue;
        }

        let object_def = ctx.gfx.object_def(&actual_id);

        if !ctx.opts.editor_only
            && object_def.is_some_and(|object| object.is_editor_only)
        {
            continue;
        }

        let proxy_id = {
            let tile = match object_def.map(|def| &def.kind) {
                Some(ObjectKind::OverrideObject(tile)) => *tile,
                _ => *tile,
            };
            ObjectId(tile, None)
        };

        let curs = Cursor {
            i,
            actual_id,
            proxy_id,
        };

        match curs.proxy_id.0 {
            Tile(0, _) => bank0::draw_bank_0_object(ctx, curs)?,
            Tile(1, _) => bank1::draw_bank_1_object(ctx, curs)?,
            Tile(2, _) => bank2::draw_bank_2_object(ctx, curs)?,
            Tile(8, _) => bank8::draw_bank_8_object(ctx, curs)?,
            _ => draw_object(ctx, curs.i, curs.actual_id)?,
        }
    }

    Ok(())
}

#[inline]
fn draw_object(ctx: &mut DrawContext, at_index: usize, object: ObjectId) -> Result<()> {
    let draw_params = ctx.gfx.object_def(&object)
        .map_or_else(Default::default, |def| def.draw_params.clone());

    draw_object_with_params(ctx, at_index, object, &draw_params)
}

#[inline]
fn draw_object_with_params(ctx: &mut DrawContext, at_index: usize, object: ObjectId, params: &DrawParams) -> Result<()> {
    if let Some(obj_image) = ctx.gfx.object(&object)? {
        draw_spritesheet(ctx, at_index as u8, params, ctx.sync.anim_t, obj_image);
    }

    Ok(())
}

fn draw_spritesheet(ctx: &mut DrawContext, at_index: u8, params: &DrawParams, anim_t: u32, obj_img: Rc<RgbaImage>) {
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
        let alpha = thread_rng().gen_range(alpha_range.clone()) as f32 / 255.0;
        blend_modes::overlay_with_alpha(&mut ctx.image, &*frame, final_x, final_y, params.blend_mode, alpha);
    }
    else {
        blend_modes::overlay(&mut ctx.image, &*frame, final_x, final_y, params.blend_mode);
    }
}

fn pick_frame<'a>(object_img: &'a RgbaImage, params: &DrawParams, anim_t: u32) -> SubImage<&'a RgbaImage> {
    let size = object_img.dimensions();
    let (frame_width, frame_height) = params.frame_size.unwrap_or((24, 24));
    let frames_per_row = (size.0 / frame_width).max(1);

    let frame_range = params.frame_range.clone().unwrap_or_else(|| {
        let n_rows = size.1 / frame_height;
        let n_frames = n_rows * frames_per_row;
        0..n_frames
    });

    let frame = 
        if frame_range.is_empty() {
            0
        }
        else if params.is_anim_synced {
            let n_frames = frame_range.end - frame_range.start;
            (anim_t % n_frames) + frame_range.start
        }
        else {
            thread_rng().gen_range(frame_range)
        };

    let frame_x = (frame % frames_per_row) * frame_width;
    let frame_y = (frame / frames_per_row) * frame_height;

    imageops::crop_imm(object_img, frame_x, frame_y, frame_width, frame_height)
}
