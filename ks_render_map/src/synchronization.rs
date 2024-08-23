use std::{cmp::min, collections::HashMap};

use rand::{seq::SliceRandom, thread_rng, Rng};
use libks::map_bin::{ScreenData, Tile};

pub struct ScreenSync {
    pub anim_t: u32,
    pub limiters: HashMap<Tile, Limiter>,
}

pub struct Limiter {
    count: usize,
    chosen: Vec<usize>,
}

impl ScreenSync {
    pub fn new(screen: &ScreenData) -> Self {
        let anim_t = thread_rng().gen();
        let mut limiters = HashMap::new();
        
        // Count limited objects
        let mut n_8_10 = 0;
        let mut n_8_15 = 0;
        let mut n_17_3 = 0;
        let mut n_18_6 = 0;
        let mut n_19_50 = 0;

        for layer in &screen.layers[4..] {
            for tile in &layer.0 {
                match tile {
                    Tile(8, 10) => n_8_10 += 1,
                    Tile(8, 15) => n_8_15 += 1,
                    Tile(17, 3) => n_17_3 += 1,
                    Tile(18, 6) => n_18_6 += 1,
                    Tile(19, 50) => n_19_50 += 1,
                    Tile(19, 1..=49 | 51..) => {
                        // All collectibles except 19-50 are limited to the first one
                        limiters.entry(*tile)
                            .or_insert(Limiter::first());
                    },
                    _ => (),
                }
            }
        }

        if n_8_10 > 0 {
            let n_active_8_10 = min(2, n_8_10);
            let limiter = Limiter::choose_n(n_8_10, n_active_8_10);
            limiters.insert(Tile(8, 10), limiter);
        }

        if n_8_15 > 0 {
            let n_active_8_15 = (1.0 + (n_8_15 as f32).log2())
                .round()
                .clamp(0.0, n_8_15 as f32)
                as usize;
            let limiter = Limiter::choose_n(n_8_15, n_active_8_15);
            limiters.insert(Tile(8, 15), limiter);
        }
        
        if n_17_3 > 0 {
            limiters.insert(Tile(17, 3), Limiter::choose_one(n_17_3));
        }

        if n_18_6 > 0 {
            limiters.insert(Tile(18, 6), Limiter::choose_one(n_18_6));
        }

        if n_19_50 > 0 {
            limiters.insert(Tile(19, 50), Limiter::choose_one(n_19_50));
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

    pub fn first() -> Self {
        Self {
            count: 0,
            chosen: vec![0],
        }
    }

    pub fn choose_one(total: usize) -> Self {
        let mut chosen = Vec::new();

        if total > 0 {
            let sample = thread_rng().gen_range(0..total);
            chosen.push(sample);
        }

        Self { count: 0, chosen }
    }

    pub fn choose_n(total: usize, n: usize) -> Self {
        if total == 0 || n == 0 {
            return Self { count: 0, chosen: Vec::new() };
        }

        let mut all: Vec<usize> = (0..total).collect();
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
