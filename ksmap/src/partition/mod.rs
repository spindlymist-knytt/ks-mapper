mod bounds;
mod grid;
mod islands;

use libks::ScreenCoord;

use crate::screen_map::ScreenMap;

pub use bounds::Bounds;
pub use grid::GridPartitioner;
pub use islands::IslandsPartitioner;

pub trait Partitioner {
    fn partitions(&self, screens: &ScreenMap) -> Vec<Partition>;
}

#[derive(Debug, Clone)]
pub struct Partition {
    positions: Vec<ScreenCoord>,
    bounds: Bounds,
}

impl Partition {
    pub fn new(positions: Vec<ScreenCoord>) -> Self {
        let bounds = Bounds::from(positions.as_slice());
        Self {
            positions,
            bounds,
        }
    }

    pub fn positions(&self) -> &[ScreenCoord] {
        &self.positions
    }
    
    pub fn bounds(&self) -> Bounds {
        self.bounds.clone()
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }
}

impl IntoIterator for Partition {
    type Item = ScreenCoord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.positions.into_iter()
    }
}

impl<'a> IntoIterator for &'a Partition {
    type Item = &'a ScreenCoord;
    type IntoIter = std::slice::Iter<'a, ScreenCoord>;

    fn into_iter(self) -> Self::IntoIter {
        self.positions.iter()
    }
}
