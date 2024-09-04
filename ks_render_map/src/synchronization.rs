use std::collections::HashMap;

use rand::{seq::SliceRandom, thread_rng, Rng};
use libks::map_bin::ScreenData;

use crate::definitions::{Limit, ObjectDef, ObjectId};

pub struct ScreenSync {
    pub anim_t: u32,
    pub limiters: HashMap<ObjectId, Limiter>,
}

pub struct Limiter {
    count: usize,
    chosen: Vec<usize>,
}

// let mut n_8_10 = 0;
// let mut n_8_15 = 0;
// let mut n_17_3 = 0;
// let mut n_18_6 = 0;
// let mut n_19_50 = 0;

impl ScreenSync {
    pub fn new(screen: &ScreenData, object_defs: &HashMap<ObjectId, ObjectDef>) -> Self {
        let anim_t = thread_rng().gen();
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
            let id = ObjectId(tile, None);

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
        let (shuffled, _) = all.partial_shuffle(&mut thread_rng(), n);

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
