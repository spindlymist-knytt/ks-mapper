use std::{cmp::{max, min}, ops::Range};

use crate::Position;

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

    pub fn top(&self) -> i64 {
        self.y.start
    }

    pub fn right(&self) -> i64 {
        self.x.end - 1
    }

    pub fn bottom(&self) -> i64 {
        self.y.end - 1
    }

    pub fn left(&self) -> i64 {
        self.x.start
    }
}

impl From<&[Position]> for Bounds {
    fn from(positions: &[Position]) -> Self {
        Self::from_iter(positions)
    }
}

impl<'a> FromIterator<&'a Position> for Bounds {
    fn from_iter<I>(positions: I) -> Self
    where
        I: IntoIterator<Item = &'a Position>,
    {
        let mut min_x = i64::MAX;
        let mut min_y = i64::MAX;
        let mut max_x = i64::MIN;
        let mut max_y = i64::MIN;

        for pos in positions {
            min_x = min(min_x, pos.0);
            min_y = min(min_y, pos.1);
            max_x = max(max_x, pos.0);
            max_y = max(max_y, pos.1);
        }

        Self {
            x: min_x..max_x + 1,
            y: min_y..max_y + 1,
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
