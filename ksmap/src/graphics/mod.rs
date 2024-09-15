use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{self, BufReader},
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{Context, Result};
use image::{DynamicImage, Rgba, RgbaImage};
use libks::map_bin::AssetId;

use crate::definitions::{ObjectDef, ObjectId, ObjectKind};

mod png_decoder;

pub struct GraphicsLoader {
    paths: Paths,
    object_defs: HashMap<ObjectId, ObjectDef>,
    tilesets: HashMap<AssetId, Option<Rc<RgbaImage>>>,
    gradients: HashMap<AssetId, Option<Rc<RgbaImage>>>,
    objects: HashMap<ObjectId, Option<Rc<RgbaImage>>>,
}

pub struct Paths {
    data_tilesets: PathBuf,
    data_gradients: PathBuf,
    editor_objects: PathBuf,
    level_tilesets: PathBuf,
    level_gradients: PathBuf,
    custom_objects: PathBuf,
    templates: PathBuf,
}

impl Paths {
    pub fn new(data_dir: impl AsRef<Path>, level_dir: impl AsRef<Path>, templates_dir: PathBuf) -> Self {
        Self {
            data_tilesets: data_dir.as_ref().join("Tilesets"),
            data_gradients: data_dir.as_ref().join("Gradients"),
            editor_objects: data_dir.as_ref().join("Objects"),
            level_tilesets: level_dir.as_ref().join("Tilesets"),
            level_gradients: level_dir.as_ref().join("Gradients"),
            custom_objects: level_dir.as_ref().join("Custom Objects"),
            templates: templates_dir,
        }
    }
}

impl GraphicsLoader {
    pub fn new(
        data_dir: impl AsRef<Path>,
        level_dir: impl AsRef<Path>,
        templates_dir: impl AsRef<Path>,
        object_defs: HashMap<ObjectId, ObjectDef>,
    ) -> Self {
        let paths = Paths::new(
            data_dir.as_ref().to_owned(),
            level_dir.as_ref().to_owned(),
            templates_dir.as_ref().to_owned(),
        );

        Self {
            paths,
            object_defs,
            tilesets: HashMap::new(),
            gradients: HashMap::new(),
            objects: HashMap::new(),
        }
    }

    pub fn object_def(&self, id: &ObjectId) -> Option<&ObjectDef> {
        self.object_defs.get(id)
    }

    pub fn object_defs(&self) -> &HashMap<ObjectId, ObjectDef> {
        &self.object_defs
    }

    pub fn tileset(&mut self, id: AssetId) -> Result<Option<Rc<RgbaImage>>> {
        let image = match self.tilesets.get(&id) {
            Some(cached) => cached.as_ref().map(Rc::clone),
            None => {
                let cached = load_tileset(&self.paths, id)?
                    .map(Rc::new);
                let image = cached.as_ref().map(Rc::clone);
                self.tilesets.insert(id, cached);

                image
            }
        };

        Ok(image)
    }

    pub fn gradient(&mut self, id: AssetId) -> Result<Option<Rc<RgbaImage>>> {
        let image = match self.gradients.get(&id) {
            Some(cached) => cached.as_ref().map(Rc::clone),
            None => {
                let cached = load_gradient(&self.paths, id)?
                    .map(Rc::new);
                let image = cached.as_ref().map(Rc::clone);
                self.gradients.insert(id, cached);

                image
            }
        };

        Ok(image)
    }

    pub fn object(&mut self, id: &ObjectId) -> Result<Option<Rc<RgbaImage>>> {
        let image = match self.objects.get(id) {
            Some(cached) => cached.as_ref().map(Rc::clone),
            None => {
                let def = self.object_defs.get(&id);
                let cached = match def.map(|def| &def.kind) {
                        Some(ObjectKind::Object) | None => load_stock_object(&self.paths, id, def)?,
                        Some(ObjectKind::CustomObject) => load_custom_object(&self.paths, def.unwrap())?,
                        Some(ObjectKind::OverrideObject(_)) =>
                            load_override_object(&self.paths, def.unwrap(), &self.object_defs)?
                    }
                    .map(Rc::new);
                let image = cached.as_ref().map(Rc::clone);
                self.objects.insert(id.clone(), cached);

                image
            }
        };

        Ok(image)
    }
}

const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
const MAGENTA: Rgba<u8> = Rgba([255, 0, 255, 255]);

fn try_load_image(path: &Path, magic_color: Rgba<u8>, force_magic_color: bool) -> Result<Option<RgbaImage>> {
    let decoder = {
        let file = match OpenOptions::new().read(true).open(path) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Ok(None);
            },
            Err(err) => Err(err)?,
        };
        png_decoder::PngDecoder::new(BufReader::new(file))
            .with_context(|| format!("Error while decoding {path:?}"))?
    };

    let image = DynamicImage::from_decoder(decoder)
        .with_context(|| format!("Error while decoding {path:?}"))?;

    let is_24_bpp = matches!(image, DynamicImage::ImageRgb8(_));
    let mut image = image.to_rgba8();

    if is_24_bpp || force_magic_color {
        for pixel in image.pixels_mut() {
            if *pixel == magic_color {
                pixel.0[0] = 0;
                pixel.0[1] = 0;
                pixel.0[2] = 0;
                pixel.0[3] = 0;
            }
        }
    }

    Ok(Some(image))
}

fn try_load_image_from_paths(paths: &[&Path], magic_color: Rgba<u8>, force_magic_color: bool) -> Result<Option<RgbaImage>> {
    for path in paths {
        if let Some(image) = try_load_image(path, magic_color, force_magic_color)? {
            return Ok(Some(image))
        }
    }

    Ok(None)
}

fn load_tileset(paths: &Paths, id: AssetId) -> Result<Option<RgbaImage>> {
    let suffix = format!("Tileset{id}.png");

    try_load_image_from_paths(&[
        &paths.level_tilesets.join(&suffix),
        &paths.data_tilesets.join(&suffix),
    ], MAGENTA, false)
}

fn load_gradient(paths: &Paths, id: AssetId) -> Result<Option<RgbaImage>> {
    let suffix = format!("Gradient{id}.png");

    try_load_image_from_paths(&[
        &paths.level_gradients.join(&suffix),
        &paths.data_gradients.join(&suffix),
    ], MAGENTA, false)
}

fn load_stock_object(
    paths: &Paths,
    ObjectId(tile, variant): &ObjectId,
    def: Option<&ObjectDef>,
) -> Result<Option<RgbaImage>> {
    let suffix = match def.and_then(|def| def.path.as_ref()) {
        Some(path) => path,
        None => match variant {
            Some(variant) => &format!("Bank{}/Object{}_{}.png", tile.0, tile.1, variant),
            None => &format!("Bank{}/Object{}.png", tile.0, tile.1),
        },
    };

    try_load_image_from_paths(&[
        &paths.templates.join(suffix),
        &paths.editor_objects.join(suffix),
    ], MAGENTA, true)
}

fn load_custom_object(paths: &Paths, def: &ObjectDef) -> Result<Option<RgbaImage>> {
    let Some(object_path) = def.path.as_ref() else {
        return Ok(None);
    };

    let image_path = paths.custom_objects.join(object_path);
    try_load_image(&image_path, BLACK, false)
}

fn load_override_object(
    paths: &Paths,
    def: &ObjectDef,
    object_defs: &HashMap<ObjectId, ObjectDef>
) -> Result<Option<RgbaImage>> {
    let mut image = 
        if def.ignore_oco_path {
            let ObjectKind::OverrideObject(original_tile) = def.kind else {
                return Ok(None);
            };
            let original_id = ObjectId(original_tile, None);
            let Some(original_def) = object_defs.get(&original_id) else {
                return Ok(None);
            };
            load_stock_object(paths, &original_id, Some(original_def))?
        }
        else {
            load_custom_object(paths, def)?
        };

    if def.replace_colors.is_empty() {
        return Ok(image);
    }

    if let Some(image) = image.as_mut() {
        for pixel in image.pixels_mut() {
            for (old, new) in &def.replace_colors {
                if pixel.0[0] == old[0]
                    && pixel.0[1] == old[1]
                    && pixel.0[2] == old[2]
                {
                    pixel.0[0] = new[0];
                    pixel.0[1] = new[1];
                    pixel.0[2] = new[2];
                    // Alpha channel is preserved
                }
            }
        }
    }

    Ok(image)
}
