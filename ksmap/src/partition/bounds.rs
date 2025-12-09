use std::ops::Range;

use libks::ScreenCoord;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bounds {
    pub x: Range<i64>,
    pub y: Range<i64>,
}

impl Bounds {
    pub fn is_empty(&self) -> bool {
        self.x.is_empty() || self.y.is_empty()
    }

    pub fn size(&self) -> (u64, u64) {
        (self.width(), self.height())
    }

    pub fn width(&self) -> u64 {
        self.x.end.abs_diff(self.x.start)
    }
    
    pub fn height(&self) -> u64 {
        self.y.end.abs_diff(self.y.start)
    }
}

impl From<&[ScreenCoord]> for Bounds {
    fn from(positions: &[ScreenCoord]) -> Self {
        Self::from_iter(positions)
    }
}

impl<'a> FromIterator<&'a ScreenCoord> for Bounds {
    fn from_iter<I>(positions: I) -> Self
    where
        I: IntoIterator<Item = &'a ScreenCoord>,
    {
        let mut positions = positions.into_iter();
        
        if let Some(first) = positions.next() {
            let mut min_x = first.0;
            let mut min_y = first.1;
            let mut max_x = first.0;
            let mut max_y = first.1;
            
            for pos in positions {
                min_x = i32::min(min_x, pos.0);
                min_y = i32::min(min_y, pos.1);
                max_x = i32::max(max_x, pos.0);
                max_y = i32::max(max_y, pos.1);
            }
            
            Self {
                x: (min_x as i64)..(max_x as i64 + 1),
                y: (min_y as i64)..(max_y as i64 + 1),
            }
        }
        else {
            Self {
                x: 0..0,
                y: 0..0,
            }
        }
    }
}

impl std::fmt::Display for Bounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "empty")
        }
        else if self.size() == (1, 1) {
            write!(f, "x{}y{}", self.x.start, self.y.start)
        }
        else {
            write!(f, "x{}y{} to x{}y{}", self.x.start, self.y.start, self.x.end - 1, self.y.end - 1)
        }
    }
}
