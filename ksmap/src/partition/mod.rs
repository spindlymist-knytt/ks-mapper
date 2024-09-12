use crate::{Position, screen_map::ScreenMap};

mod bounds;
pub use bounds::Bounds;

mod grid;
pub use grid::GridStrategy;

mod islands;
pub use islands::IslandsStrategy;

pub trait PartitionStrategy {
    fn partitions(&self, screens: &ScreenMap) -> Result<Vec<Partition>, anyhow::Error>;
}

#[derive(Debug, Clone)]
pub struct Partition {
    positions: Vec<Position>,
    bounds: Bounds,
}

impl Partition {
    pub fn new(positions: Vec<Position>) -> Self {
        let bounds = Bounds::from(positions.as_slice());
        Self {
            positions,
            bounds,
        }
    }

    pub fn positions(&self) -> &[Position] {
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
    type Item = Position;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.positions.into_iter()
    }
}

impl<'a> IntoIterator for &'a Partition {
    type Item = &'a Position;
    type IntoIter = std::slice::Iter<'a, Position>;

    fn into_iter(self) -> Self::IntoIter {
        self.positions.iter()
    }
}
