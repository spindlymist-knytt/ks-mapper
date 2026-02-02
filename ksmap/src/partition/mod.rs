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

pub fn merge_redundant_partitions(partitions: &mut Vec<Partition>) {
    // This algorithm is not great, but it's fine for small numbers of partitions
    // For each partition, consider each partition that comes after it in the list
    let mut i = 0;
    while i < partitions.len() {
        let current = partitions[i].bounds();
        let mut j = i + 1;
        while j < partitions.len() {
            let other = partitions[j].bounds();
            // If we fully contain the other partition, remove it from the list
            if current.contains(&other) {
                let removed = partitions.swap_remove(j);
                partitions[i].merge(removed);
            }
            // If the other partition fully contains us, replace the current partition with that one
            // and remove the other one from the list
            else if other.contains(&current) {
                let removed = partitions.swap_remove(j);
                partitions[i].merge(removed);
                j = i + 1; // Start over
                continue;
            }
            else {
                j += 1;
            }
        }
        i += 1;
    }
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
    
    pub fn merge(&mut self, other: Partition) {
        self.positions.extend(other.positions.into_iter());
        self.bounds = Bounds::union(&self.bounds, &other.bounds);
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
