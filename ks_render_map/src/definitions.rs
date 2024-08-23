use std::{collections::HashMap, fs, ops::{Range, RangeInclusive}, path::Path};

use anyhow::Result;
use libks::map_bin::Tile;
use libks_ini::Ini;
use serde::Deserialize;

use crate::drawing::BlendMode;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ObjectDef {
    #[serde(default)]
    pub is_editor_only: bool,
    // #[serde(default)]
    // pub is_custom_object: bool,
    pub path: Option<String>,
    #[serde(flatten)]
    pub draw_params: DrawParams,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DrawParams {
    #[serde(default)]
    pub is_anim_synced: bool,
    #[serde(default)]
    pub blend_mode: BlendMode,
    pub alpha_range: Option<RangeInclusive<u8>>,
    pub frame_size: Option<(u32, u32)>,
    pub frame_range: Option<Range<u32>>,
    pub offset: Option<(i64, i64)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectId(pub Tile, pub Option<String>);

pub fn load_object_defs(path: impl AsRef<Path>) -> Result<HashMap<ObjectId, ObjectDef>> {
    let mut objects = HashMap::<ObjectId, ObjectDef>::new();

    let raw = fs::read_to_string(path)?;
    let table: toml::Table = raw.parse()?;

    for (key, value) in table.into_iter() {
        if let toml::Value::Table(table) = value {
            let id = ObjectId::try_from(key)?;
            let def = table.try_into()?;
            objects.insert(id, def);
        }
    }

    Ok(objects)
}

pub fn insert_custom_obj_defs(defs: &mut HashMap<ObjectId, ObjectDef>, ini: &Ini) {
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

        let Some(path) = section.get("Image") else { continue };

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

        let frame_range = match (anim_repeat, anim_to) {
            (0, Some(anim_to)) => anim_loop_back..anim_to + 1,
            (_, Some(_)) => anim_from..anim_from + 1,
            _ => 0..1,
        };
        let mut frame_range = Some(frame_range);

        let mut is_anim_synced = true;

        // OCOs

        let bank = section.get("Bank")
            .and_then(|v| str::parse(v).ok());
        let object = section.get("Object")
            .and_then(|v| str::parse(v).ok());

        if let (Some(bank), Some(obj)) = (bank, object) {
            let oco_id = ObjectId(Tile(bank, obj), None);
            if let Some(oco_def) = defs.get(&oco_id) {
                is_anim_synced = oco_def.draw_params.is_anim_synced;
                frame_range = oco_def.draw_params.frame_range.clone();
                if let Some(offset) = oco_def.draw_params.offset {
                    offset_x += offset.0;
                    offset_y += offset.1;
                }
            }
            else {
                is_anim_synced = false;
                frame_range = None;
                offset_x = 0;
                offset_y = 0;
            }
        }

        let draw = DrawParams {
            is_anim_synced,
            blend_mode: BlendMode::Over,
            alpha_range: None,
            frame_size: Some((frame_width, frame_height)),
            frame_range,
            offset: Some((offset_x, offset_y)),
        };

        let def = ObjectDef {
            is_editor_only: false,
            // is_custom_object: true,
            path: Some(path.to_owned()),
            draw_params: draw,
        };

        defs.insert(ObjectId(tile, None), def);
    }
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
