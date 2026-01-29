use std::collections::HashMap;

use petgraph::unionfind::UnionFind;
use rand::{prelude::*, rng};
use libks::{constants::{SCREEN_WIDTH, TILES_PER_LAYER}, map_bin::{LayerData, ScreenData}};

use crate::{
    analysis::count_laser_phases, definitions::{LaserPhase, Limit, ObjectDefs, ObjectKind}, id::ObjectId, screen_map::ScreenMap
};

pub struct WorldSync {
    pub groups: Vec<GroupSync>,
}

#[derive(Clone, Copy, Default)]
pub struct GroupSync {
    pub anim_t: u32,
    pub laser_phase: LaserPhase,
}

pub struct ScreenSync {
    pub group: GroupSync,
    pub anim_t: u32,
    pub limiters: HashMap<ObjectId, Limiter>,
}

pub struct Limiter {
    count: usize,
    chosen: Vec<usize>,
}

pub struct SyncOptions {
    pub maximize_visible_lasers: bool,
}

impl WorldSync {
    pub fn new(screens: &ScreenMap, object_defs: &ObjectDefs, options: &SyncOptions) -> Self {
        const TOP_LEFT: usize = 0;
        const TOP_RIGHT: usize = SCREEN_WIDTH - 1;
        const BOTTOM_LEFT: usize = TILES_PER_LAYER - SCREEN_WIDTH;
        const _BOTTOM_RIGHT: usize = TILES_PER_LAYER - 1;
        const OFFSET_NORTH_TO_SOUTH: usize = BOTTOM_LEFT - TOP_LEFT;
        const OFFSET_WEST_TO_EAST: usize = TOP_RIGHT - TOP_LEFT;
        
        let mut uf = UnionFind::<usize>::new(screens.len());
        
        for (index_current, screen) in screens.iter().enumerate() {
            // Northern border
            if let Some(index_north) = screens.index_of(&(screen.position.0, screen.position.1 - 1)) {
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
                                uf.union(index_current, index_north);
                                break 'north;
                            }
                        }
                    }
                }
            }
            
            // Western border
            if let Some(index_west) = screens.index_of(&(screen.position.0 - 1, screen.position.1)) {
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
                                uf.union(index_current, index_west);
                                break 'west;
                            }
                        }
                    }
                }
            }
        }
        
        let mut groups_by_rep = HashMap::<usize, Vec<usize>>::new();
        for (index_member, index_rep) in uf.into_labeling().into_iter().enumerate() {
            let members = groups_by_rep.entry(index_rep)
                .or_insert_with(|| Vec::new());
            members.push(index_member);
        }
        
        let mut groups = vec![GroupSync::default(); screens.len()];
        let laser_phases = count_laser_phases(screens, object_defs);
        let mut rng = rng();
        for (_index_rep, members) in groups_by_rep {
            let anim_t = rng.next_u32();
            let laser_phase = pick_laser_phase(&mut rng, &laser_phases, &members, options.maximize_visible_lasers);           
            let group_sync = GroupSync {
                anim_t,
                laser_phase,
            };
            for index_member in members {
                groups[index_member] = group_sync;
            }
        }
        
        Self {
            groups,
        }
    }
}

fn pick_laser_phase(
    rng: &mut impl Rng,
    phase_counts: &[[usize; 2]],
    members: &[usize],
    maximize: bool
) -> LaserPhase {
    let mut total_red = 0;
    let mut total_green = 0;
    
    for index_member in members {
        total_red += phase_counts[*index_member][LaserPhase::Red as usize];
        total_green += phase_counts[*index_member][LaserPhase::Green as usize];
    }
    
    if total_green == 0 || (maximize && total_red > total_green) {
        LaserPhase::Red
    }
    else if total_red == 0 || (maximize && total_green > total_red) {
        LaserPhase::Green
    }
    else {
        *[LaserPhase::Red, LaserPhase::Green].choose(rng).unwrap()
    }
}

impl ScreenSync {
    pub fn new(screen: &ScreenData, object_defs: &ObjectDefs, group: GroupSync) -> Self {
        let anim_t = rng().next_u32();
        let mut limiters = HashMap::new();
        let mut counts = HashMap::new();

        for LayerData(layer) in &screen.layers[4..] {
            for tile in layer {
                if tile.1 > 0
                    && let Some(def) = object_defs.get(&ObjectId::from(tile))
                    && def.limit != Limit::None
                {
                    let id = match &def.kind {
                        ObjectKind::OverrideObject(tile_original) => ObjectId::from(tile_original),
                        _ => ObjectId::from(tile),
                    };
                    counts.entry(id)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
            }
        }

        for (id, count) in counts {
            let Some(def) = object_defs.get(&id) else { continue };
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
            group,
            anim_t,
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
