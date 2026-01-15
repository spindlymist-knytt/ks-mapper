use std::collections::HashSet;

use libks::map_bin::{AssetId, ScreenData};

use crate::definitions::{ObjectDefs, ObjectId};

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
        
        for layer in &screen.layers[4..] {
            for tile in &layer.0 {
                if tile.1 > 0 {
                    objects.insert(ObjectId(*tile, None));
                    for variant in defs.variants_of(*tile) {
                        objects.insert(ObjectId(*tile, Some(variant.clone())));
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
    let objects = objects.into_iter().collect();
    
    AssetsUsed {
        tilesets,
        gradients,
        objects,
    }
}
