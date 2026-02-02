use libks::ScreenCoord;

use crate::screen_map::ScreenMap;
use super::{Bounds, Partition, Partitioner};

pub struct GridPartitioner {
    pub max_size: (u64, u64),
    pub rows: Option<u64>,
    pub cols: Option<u64>,
}

impl Default for GridPartitioner {
    fn default() -> Self {
        Self {
            max_size: (48000, 48000),
            rows: None,
            cols: None,
        }
    }
}

impl Partitioner for GridPartitioner {
    fn partitions(&self, screens: &ScreenMap) -> Vec<Partition> {
        let bounds = Bounds::from_iter(screens.iter_positions());
        
        if bounds.width() <= self.max_size.0
            && bounds.height() <= self.max_size.1
        {
            let positions: Vec<_> = screens.iter()
                .map(|screen| screen.position)
                .collect();
            return vec![Partition::new(positions)];
        }
        
        let rows = self.rows.unwrap_or_else(|| calc_grid_rows(&bounds, self.max_size.1));
        let cols = self.cols.unwrap_or_else(|| calc_grid_cols(&bounds, self.max_size.0));
        let positions = screens.iter().map(|screen| &screen.position);
        
        partitions_from_grid(positions, &bounds, rows, cols)
    }
}

#[inline]
pub fn calc_grid_rows(bounds: &Bounds, max_height: u64) -> u64 {
    (bounds.height() as f64 / max_height as f64).ceil() as u64
}

#[inline]
pub fn calc_grid_cols(bounds: &Bounds, max_width: u64) -> u64 {
    (bounds.width() as f64 / max_width as f64).ceil() as u64
}

#[inline]
pub fn calc_grid_dimensions(bounds: &Bounds, max_size: (u64, u64)) -> (u64, u64) {
    let rows = calc_grid_rows(bounds, max_size.1);
    let cols = calc_grid_cols(bounds, max_size.0);
    (rows, cols)
}

pub fn partitions_from_grid<'a, 'b, I>(
    positions: I,
    bounds: &'b Bounds,
    rows: u64,
    cols: u64,
) -> Vec<Partition>
where
    I: std::iter::Iterator<Item = &'a ScreenCoord>
{
    let cell_width = (bounds.width() as f64 / cols as f64).ceil() as u64;
    let cell_height = (bounds.height() as f64 / rows as f64).ceil() as u64;
    let n_cells = (rows * cols) as usize;

    let mut partitions: Vec<Vec<ScreenCoord>> = (0..n_cells)
        .map(|_| Vec::new())
        .collect();

    for position in positions {
        let x = position.0 as i64;
        let y = position.1 as i64;
        let dx = x.abs_diff(bounds.x.start);
        let dy = y.abs_diff(bounds.y.start);

        let cell_x = u64::min(dx / cell_width, cols - 1);
        let cell_y = u64::min(dy / cell_height, rows - 1);
        let cell_i = (cell_x + cell_y * cols) as usize;

        partitions[cell_i].push(*position);
    }

    partitions.into_iter()
        .filter(|p| !p.is_empty())
        .map(Partition::new)
        .collect()
}
