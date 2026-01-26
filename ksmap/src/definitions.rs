use std::{collections::HashMap, fs, ops::{Deref, DerefMut, Range, RangeInclusive}, path::Path};

use anyhow::Result;
use libks::map_bin::Tile;
use libks_ini::Ini;
use serde::Deserialize;

use crate::drawing::BlendMode;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ObjectDef {
    #[serde(skip)]
    pub kind: ObjectKind,
    pub path: Option<String>,
    #[serde(default)]
    pub is_editor_only: bool,
    #[serde(flatten)]
    pub draw_params: DrawParams,
    #[serde(default)]
    pub offset_combine: OffsetCombine,
    #[serde(default)]
    pub ignore_oco_path: bool,
    #[serde(default)]
    pub limit: Limit,
    pub color_base: Option<i64>,
    #[serde(default)]
    pub color_offsets: Vec<i64>,
    #[serde(skip)]
    pub replace_colors: Vec<([u8; 3], [u8; 3])>,
}

#[derive(Debug, Clone, Default)]
pub enum ObjectKind {
    #[default]
    Object,
    CustomObject,
    OverrideObject(Tile),
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DrawParams {
    #[serde(default)]
    pub is_anim_synced: bool,
    #[serde(default)]
    pub sync_offset: u32,
    #[serde(default)]
    pub blend_mode: BlendMode,
    pub alpha_range: Option<RangeInclusive<u8>>,
    pub frame_size: Option<(u32, u32)>,
    pub frame_range: Option<Range<u32>>,
    pub offset: Option<(i64, i64)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectId(pub Tile, pub Option<String>);

impl ObjectId {
    pub fn into_variant(mut self, variant: &str) -> Self {
        self.1 = Some(variant.to_owned());
        self
    }

    pub fn with_variant(&self, variant: &str) -> Self {
        Self(self.0, Some(variant.to_owned()))
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub enum OffsetCombine {
    #[default]
    Add,
    Replace,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(tag = "pick")]
pub enum Limit {
    #[default]
    None,
    First { n: usize },
    Random { n: usize },
    LogNPlusOne,
}

pub struct ObjectDefs {
    pub defs: HashMap<ObjectId, ObjectDef>,
    pub variants: HashMap<Tile, Vec<String>>,
}

impl ObjectDefs {
    pub fn variants_of(&self, object: Tile) -> impl Iterator<Item = &String> {
        match self.variants.get(&object) {
            Some(variants) => variants.iter(),
            None => [].iter(),
        }
    }
}

impl Deref for ObjectDefs {
    type Target = HashMap<ObjectId, ObjectDef>;

    fn deref(&self) -> &Self::Target {
        &self.defs
    }
}

impl DerefMut for ObjectDefs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.defs
    }
}

pub fn load_object_defs(path: impl AsRef<Path>) -> Result<ObjectDefs> {
    let mut defs = HashMap::<ObjectId, ObjectDef>::new();
    let mut variants = HashMap::<Tile, Vec<String>>::new();

    let raw = fs::read_to_string(path)?;
    let table: toml::Table = raw.parse()?;

    for (key, value) in table.into_iter() {
        if let toml::Value::Table(table) = value {
            let id = ObjectId::try_from(key)?;
            let def = table.try_into()?;
            
            if let Some(variant) = id.1.as_ref() {
                variants.entry(id.0)
                    .or_insert(Vec::new())
                    .push(variant.clone());
            }
            
            defs.insert(id, def);
        }
    }

    Ok(ObjectDefs {
        defs,
        variants,
    })
}

pub fn insert_custom_obj_defs(defs: &mut ObjectDefs, ini: &Ini) {
    for section in ini.iter_sections() {
        let key_lower = section.key().to_ascii_lowercase();

        let Some(suffix) = key_lower.strip_prefix("custom object ") else {
            continue;
        };

        let tile = match suffix.strip_prefix('b') {
            Some(index) => {
                let Ok(index) = str::parse::<u8>(index) else { continue };
                Tile(254, index)
            },
            None => {
                let Ok(index) = str::parse::<u8>(suffix) else { continue };
                Tile(255, index)
            },
        };

        let bank = section.get("Bank")
            .and_then(|v| str::parse(v).ok())
            .unwrap_or(0);
        let object = section.get("Object")
            .and_then(|v| str::parse(v).ok());

        let path = section.get("Image").map(|v| v.to_owned());
        if path.is_none() && bank != 7 {
            continue;
        }

        let frame_width: u32 = section.get("Tile Width")
            .and_then(|v| str::parse(v).ok())
            .unwrap_or(24);
        let frame_height: u32 = section.get("Tile Height")
            .and_then(|v| str::parse(v).ok())
            .unwrap_or(24);
        let mut offset_x: i64 = section.get("Offset X")
            .and_then(|v| str::parse(v).ok())
            .unwrap_or(0);
        let mut offset_y: i64 = section.get("Offset Y")
            .and_then(|v| str::parse(v).ok())
            .unwrap_or(0);
        let anim_to: Option<u32> = section.get("Init AnimTo")
            .and_then(|v| str::parse(v).ok());
        let anim_from: u32 = section.get("Init AnimFrom")
            .and_then(|v| str::parse(v).ok())
            .map(|v: u32| v.min(anim_to.unwrap_or(0)))
            .unwrap_or(0);
        let anim_loop_back: u32 = section.get("Init AnimLoopback")
            .and_then(|v| str::parse(v).ok())
            .map(|v: u32| v.min(anim_to.unwrap_or(0)))
            .unwrap_or(0);
        let anim_repeat: u32 = section.get("Init AnimRepeat")
            .and_then(|v| str::parse(v).ok())
            .unwrap_or(0);

        // OCOs
        
        let kind;
        let frame_range;
        let is_anim_synced;
        let mut sync_offset = 0;
        let limit;
        let ignore_oco_path;
        let color_base = None;
        let color_offsets = Vec::new();
        let mut replace_colors = Vec::new();

        if let Some(object) = object {
            kind = ObjectKind::OverrideObject(Tile(bank, object));
            let oco_id = ObjectId(Tile(bank, object), None);

            if let Some(oco_def) = defs.get(&oco_id) {
                is_anim_synced = oco_def.draw_params.is_anim_synced;
                sync_offset = 0;
                frame_range = oco_def.draw_params.frame_range.clone();
                limit = oco_def.limit;
                ignore_oco_path = oco_def.ignore_oco_path;

                if let Some(offset) = oco_def.draw_params.offset {
                    match oco_def.offset_combine {
                        OffsetCombine::Add => {
                            offset_x += offset.0;
                            offset_y += offset.1;
                        },
                        OffsetCombine::Replace => {},
                    }
                }

                if let Some(color_base) = oco_def.color_base {
                    let color: i64 = section.get("Color")
                        .and_then(|v| str::parse(v).ok())
                        .unwrap_or(0);
                    for offset in [0].iter().chain(oco_def.color_offsets.iter()) {
                        let old_color = unpack_color(color_base + offset);
                        let new_color = unpack_color(color + offset);
                        replace_colors.push((old_color, new_color));
                    }
                }
            }
            else {
                is_anim_synced = false;
                sync_offset = 0;
                frame_range = None;
                offset_x = 0;
                offset_y = 0;
                limit = Limit::None;
                ignore_oco_path = false;
            }
        }
        else {
            kind = ObjectKind::CustomObject;
            frame_range = match (anim_repeat, anim_to) {
                (0, Some(anim_to)) => Some(anim_loop_back..anim_to + 1),
                (_, Some(_)) => Some(anim_from..anim_from + 1),
                _ => Some(0..1),
            };
            is_anim_synced = true;
            limit = Limit::None;
            ignore_oco_path = false;
        }

        let draw = DrawParams {
            is_anim_synced,
            sync_offset,
            blend_mode: BlendMode::Over,
            alpha_range: None,
            frame_size: Some((frame_width, frame_height)),
            frame_range,
            offset: Some((offset_x, offset_y)),
        };

        let def = ObjectDef {
            kind, 
            path,
            is_editor_only: false,
            draw_params: draw,
            offset_combine: OffsetCombine::Replace,
            ignore_oco_path,
            limit,
            color_base,
            color_offsets,
            replace_colors,
        };

        defs.insert(ObjectId(tile, None), def);
    }
}

fn unpack_color(mut color: i64) -> [u8; 3] {
    color %= 256 * 256 * 256;

    let r = color & 0x0000FF;
    let g = (color & 0x00FF00) >> 8;
    let b = (color & 0xFF0000) >> 16;

    [r as u8, g as u8, b as u8]
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.1.as_ref() {
            Some(variant) => write!(f, "{}-{} {}", self.0.0, self.0.1, variant),
            None => write!(f, "{}-{}", self.0.0, self.0.1),
        }
    }
}

impl TryFrom<&str> for ObjectId {
    type Error = ObjectIdParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (bank_and_index, variant) = match value.split_once(' ') {
            Some((id, variant)) => (id, Some(variant.to_owned())),
            None => (value, None),
        };

        let Some((bank, index)) = bank_and_index.split_once('-') else {
            return Err(ObjectIdParseError::MissingSeparator(bank_and_index.to_owned()));
        };

        let bank = match str::parse::<u8>(bank) {
            Ok(bank) => bank,
            Err(_) => return Err(ObjectIdParseError::InvalidIndex(bank.to_owned())),
        };

        let index = match str::parse::<u8>(index) {
            Ok(index) => index,
            Err(_) => return Err(ObjectIdParseError::InvalidIndex(index.to_owned())),
        };

        Ok(ObjectId(Tile(bank, index), variant))
    }
}

impl TryFrom<String> for ObjectId {
    type Error = ObjectIdParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ObjectIdParseError {
    #[error("Invalid ObjectId: missing bank/object separator")]
    MissingSeparator(String),
    #[error("Invalid ObjectId: failed to parse bank or object index")]
    InvalidIndex(String),
}
