use std::collections::HashSet;

use libks::map_bin::{AssetId, LayerData, ScreenData};

use crate::definitions::{ObjectDefs, ObjectKind};
use crate::id::ObjectId;

pub struct AssetsUsed {
    pub tilesets: Vec<AssetId>,
    pub gradients: Vec<AssetId>,
    pub objects: Vec<ObjectId>,
}

pub fn list_assets(screens: &[ScreenData], defs: &ObjectDefs) -> AssetsUsed {
    let mut tilesets = [false; 256];
    let mut gradients = [false; 256];
    let mut objects = HashSet::new();
    
    for screen in screens {
        let mut uses_tileset_a = false;
        let mut uses_tileset_b = false;
        
        for layer in &screen.layers[..4] {
            for tile in &layer.0 {
                uses_tileset_a |= tile.0 == 0 && tile.1 > 0;
                uses_tileset_b |= tile.0 == 1 && tile.1 > 0;
            }
        }
        
        
        for LayerData(layer) in &screen.layers[4..] {
            for tile in layer {
                if tile.1 > 0 {
                    let id = ObjectId::from(tile);
                    objects.insert(id.clone());
                    if let Some(def) = defs.get(&id)
                        && let ObjectKind::OverrideObject(orig_tile) = &def.kind
                    {
                        objects.insert(ObjectId::from(orig_tile));
                    }
                }
            }
        }
        
        tilesets[screen.assets.tileset_a as usize] |= uses_tileset_a;
        tilesets[screen.assets.tileset_b as usize] |= uses_tileset_b;
        gradients[screen.assets.gradient as usize] = true;
    }
    
    let tilesets = tilesets.into_iter()
        .enumerate()
        .filter(|(_, used)| *used)
        .map(|(i, _)| i as AssetId)
        .collect();
    let gradients = gradients.into_iter()
        .enumerate()
        .filter(|(_, used)| *used)
        .map(|(i, _)| i as AssetId)
        .collect();
    let mut objects: Vec<_> = objects.into_iter().collect();
    
    for i in 0..objects.len() {
        for variant in defs.variants_of(objects[i].0) {
            objects.push(objects[i].to_variant(*variant));
        }
    }
    
    AssetsUsed {
        tilesets,
        gradients,
        objects,
    }
}
