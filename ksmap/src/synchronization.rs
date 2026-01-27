use std::collections::HashMap;

use petgraph::unionfind::UnionFind;
use rand::{prelude::*, rng};
use libks::{ScreenCoord, constants::{SCREEN_WIDTH, TILES_PER_LAYER}, map_bin::{LayerData, ScreenData}};

use crate::{
    definitions::{Limit, ObjectDefs},
    id::ObjectId,
    screen_map::ScreenMap,
};

pub struct WorldSync {
    pub group_anim_ts: HashMap<ScreenCoord, u32>,
}

pub struct ScreenSync {
    pub anim_t: u32,
    pub group_anim_t: Option<u32>,
    pub limiters: HashMap<ObjectId, Limiter>,
}

pub struct Limiter {
    count: usize,
    chosen: Vec<usize>,
}

impl WorldSync {
    pub fn new(screens: &ScreenMap, object_defs: &ObjectDefs) -> Self {
        let mut groups = UnionFind::<usize>::new(screens.len());
        let mut has_group = vec![false; screens.len()];
        
        const TOP_LEFT: usize = 0;
        const TOP_RIGHT: usize = SCREEN_WIDTH - 1;
        const BOTTOM_LEFT: usize = TILES_PER_LAYER - SCREEN_WIDTH;
        const BOTTOM_RIGHT: usize = TILES_PER_LAYER - 1;
        const OFFSET_NORTH_TO_SOUTH: usize = BOTTOM_LEFT - TOP_LEFT;
        const OFFSET_WEST_TO_EAST: usize = TOP_RIGHT - TOP_LEFT;
        
        for (index_current, screen) in screens.iter().enumerate() {
            // Northern border
            if let Some(index_north) = screens.index(&(screen.position.0, screen.position.1 - 1))
                && groups.find_mut(index_current) != groups.find_mut(index_north)
            {
                let screen_north = &screens[index_north];
                'north: for LayerData(layer) in &screen.layers {
                    for i in TOP_LEFT..=TOP_RIGHT {
                        let id = ObjectId::from(layer[i]);
                        let Some(def) = object_defs.get(&id) else { continue };
                        let j = i + OFFSET_NORTH_TO_SOUTH;
                        for ObjectId(sync_tile, _) in &def.sync_params.sync_north {
                            if screen_north.layers[4].0[j] == *sync_tile
                                || screen_north.layers[5].0[j] == *sync_tile
                                || screen_north.layers[6].0[j] == *sync_tile
                                || screen_north.layers[7].0[j] == *sync_tile
                            {
                                groups.union(index_current, index_north);
                                has_group[index_current] = true;
                                has_group[index_north] = true;
                                break 'north;
                            }
                        }
                    }
                }
            }
            
            // Western border
            if let Some(index_west) = screens.index(&(screen.position.0 - 1, screen.position.1))
                && groups.find_mut(index_current) != groups.find_mut(index_west)
            {
                let screen_west = &screens[index_west];
                'west: for LayerData(layer) in &screen.layers {
                    for i in (TOP_LEFT..=BOTTOM_LEFT).step_by(SCREEN_WIDTH) {
                        let id = ObjectId::from(layer[i]);
                        let Some(def) = object_defs.get(&id) else { continue };
                        let j = i + OFFSET_WEST_TO_EAST;
                        for ObjectId(sync_tile, _) in &def.sync_params.sync_west {
                            if screen_west.layers[4].0[j] == *sync_tile
                                || screen_west.layers[5].0[j] == *sync_tile
                                || screen_west.layers[6].0[j] == *sync_tile
                                || screen_west.layers[7].0[j] == *sync_tile
                            {
                                groups.union(index_current, index_west);
                                has_group[index_current] = true;
                                has_group[index_west] = true;
                                break 'west;
                            }
                        }
                    }
                }
            }
            
            // Eastern border
            if let Some(index_east) = screens.index(&(screen.position.0 + 1, screen.position.1))
                && groups.find_mut(index_current) != groups.find_mut(index_east)
            {
                let screen_east = &screens[index_east];
                'east: for LayerData(layer) in &screen.layers {
                    for i in (TOP_RIGHT..=BOTTOM_RIGHT).step_by(SCREEN_WIDTH) {
                        let id = ObjectId::from(layer[i]);
                        let Some(def) = object_defs.get(&id) else { continue };
                        let j = i - OFFSET_WEST_TO_EAST;
                        for ObjectId(sync_tile, _) in &def.sync_params.sync_east {
                            if screen_east.layers[4].0[j] == *sync_tile
                                || screen_east.layers[5].0[j] == *sync_tile
                                || screen_east.layers[6].0[j] == *sync_tile
                                || screen_east.layers[7].0[j] == *sync_tile
                            {
                                groups.union(index_current, index_east);
                                has_group[index_current] = true;
                                has_group[index_east] = true;
                                break 'east;
                            }
                        }
                    }
                }
            }
            
            // Southern border
            if let Some(index_south) = screens.index(&(screen.position.0, screen.position.1 + 1))
                && groups.find_mut(index_current) != groups.find_mut(index_south)
            {
                let screen_south = &screens[index_south];
                'south: for LayerData(layer) in &screen.layers {
                    for i in BOTTOM_LEFT..=BOTTOM_RIGHT {
                        let id = ObjectId::from(layer[i]);
                        let Some(def) = object_defs.get(&id) else { continue };
                        let j = i - OFFSET_NORTH_TO_SOUTH;
                        for ObjectId(sync_tile, _) in &def.sync_params.sync_south {
                            if screen_south.layers[4].0[j] == *sync_tile
                                || screen_south.layers[5].0[j] == *sync_tile
                                || screen_south.layers[6].0[j] == *sync_tile
                                || screen_south.layers[7].0[j] == *sync_tile
                            {
                                groups.union(index_current, index_south);
                                has_group[index_current] = true;
                                has_group[index_south] = true;
                                break 'south;
                            }
                        }
                    }
                }
            }
        }
        
        let mut anim_ts: HashMap<ScreenCoord, u32> = HashMap::new();
        let mut rng = rng();
        let labeling = groups.into_labeling();
        
        for (index_screen, index_rep) in labeling.into_iter().enumerate() {
            if !has_group[index_screen] { continue }
            
            let screen_rep = &screens[index_rep];
            let anim_t = *anim_ts.entry(screen_rep.position)
                .or_insert_with(|| rng.random());
            
            if index_screen != index_rep {
                let screen = &screens[index_screen];
                anim_ts.insert(screen.position, anim_t);
            }
        }
        
        Self {
            group_anim_ts: anim_ts,
        }
    }
}

impl ScreenSync {
    pub fn new(screen: &ScreenData, object_defs: &ObjectDefs, group_anim_t: Option<u32>) -> Self {
        let anim_t = rng().next_u32();
        let mut limiters = HashMap::new();
        let mut counts = HashMap::new();

        for layer in &screen.layers[4..] {
            for tile in &layer.0 {
                counts.entry(*tile)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }

        for (tile, count) in counts {
            let id = ObjectId::from(tile);

            let Some(def) = object_defs.get(&id) else {
                continue
            };

            match def.limit {
                Limit::None => {},
                Limit::First { n } => {
                    let limiter = Limiter::take(n);
                    limiters.insert(id, limiter);
                },
                Limit::Random { n } => {
                    let limiter = Limiter::choose_n(count, n);
                    limiters.insert(id, limiter);
                },
                Limit::LogNPlusOne => {
                    let n = (1.0 + (count as f32).log2())
                        .round()
                        .clamp(0.0, count as f32)
                        as usize;
                    let limiter = Limiter::choose_n(count, n);
                    limiters.insert(id, limiter);
                },
            }
        }
    
        Self {
            anim_t,
            group_anim_t,
            limiters,
        }
    }
}

impl Limiter {
    pub fn new(mut chosen: Vec<usize>) -> Self {
        chosen.sort_unstable_by(|a, b| b.cmp(a));
        Self {
            count: 0,
            chosen,
        }
    }

    pub fn take(n: usize) -> Self {
        Self {
            count: 0,
            chosen: Vec::from_iter(0..n),
        }
    }

    pub fn choose_n(total: usize, n: usize) -> Self {
        if total == 0 || n == 0 {
            return Self { count: 0, chosen: Vec::new() };
        }

        let mut all = Vec::from_iter(0..total);
        let (shuffled, _) = all.partial_shuffle(&mut rng(), n);

        Self::new(shuffled.to_owned())
    }

    pub fn increment(&mut self) -> bool {
        let Some(next) = self.chosen.last() else {
            return false;
        };

        let is_chosen = self.count == *next;
        self.count += 1;

        if is_chosen {
            self.chosen.pop();
        }

        is_chosen
    }
}
