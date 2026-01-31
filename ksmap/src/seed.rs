use std::{fmt::Display, hash::Hash};

use rand::prelude::*;
use rand_seeder::Seeder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MapSeed {
    pub seed: u64,
}

/// DO NOT change the discriminants of this enum.
/// It will break the backwards compatibility of seeds.
#[derive(Debug, Hash)]
pub enum RngStep {
    Default,
    ScreenAnimationTime,
    GroupAnimationTime,
    LaserPhases,
    Limiters,
    Frame,
    Offset,
    Flip,
    Alpha,
    ElementalVariant,
}

pub struct SeedStep {
    seed: u64,
    step: RngStep,
}

impl MapSeed {
    pub fn random() -> Self {
        let seed = rand::rng().next_u64();
        Self { seed }
    }
    
    pub fn salted<H: Hash>(&self, salt: H) -> SmallRng {
        Seeder::from((self.seed, RngStep::Default, salt)).into_rng()
    }
    
    pub fn step(&self, step: RngStep) -> SeedStep {
        SeedStep {
            seed: self.seed,
            step
        }
    }
}

impl Display for MapSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>16X}", self.seed)
    }
}

impl SeedStep {
    pub fn into_rng(self) -> SmallRng {
        Seeder::from((self.seed, self.step)).into_rng()
    }
    
    pub fn into_u32(self) -> u32 {
        self.into_rng().next_u32()
    }
    
    pub fn into_u64(self) -> u64 {
        self.into_rng().next_u64()
    }
    
    pub fn salted(self, salt: impl Hash) -> SmallRng {
        Seeder::from((self.seed, self.step, salt)).into_rng()
    }
}
