use std::cmp::min;

use crate::screen_map::ScreenMap;
use super::{Bounds, Partition, PartitionStrategy};

pub struct GridStrategy {
    pub max_size: (u64, u64),
    pub rows: Option<u64>,
    pub cols: Option<u64>,
}

impl PartitionStrategy for GridStrategy {
    fn partitions(&self, screens: &ScreenMap) -> Result<Vec<Partition>, anyhow::Error> {
        let bounds = Bounds::from_iter(screens.iter_positions());
        let (level_width, level_height) = bounds.size();
        
        let cols = self.cols.unwrap_or_else(|| {
            (level_width as f64 / self.max_size.0 as f64).ceil() as u64
        });

        let rows = self.rows.unwrap_or_else(|| {
            (level_height as f64 / self.max_size.1 as f64).ceil() as u64
        });

        let cell_width = level_width / cols;
        let cell_height = level_height / rows;
        let n_cells = rows * cols;

        let mut partitions: Vec<_> = (0..n_cells)
            .map(|_| Vec::new())
            .collect();

        for screen in screens {
            let dx = screen.position.0.abs_diff(bounds.left());
            let dy = screen.position.1.abs_diff(bounds.top());

            let cell_x = min(dx / cell_width, cols - 1);
            let cell_y = min(dy / cell_height, rows - 1);
            let cell_i = (cell_x + cell_y * cols) as usize;

            partitions[cell_i].push(screen.position);
        }

        let partitions = partitions.into_iter()
            .filter(|p| !p.is_empty())
            .map(Partition::new)
            .collect();
        Ok(partitions)
    }
}
