use std::{collections::HashMap, ops::Index};

use libks::map_bin::ScreenData;

use crate::Position;

pub struct ScreenMap {
    screens: Vec<ScreenData>,
    map: HashMap<Position, usize>,
}

impl ScreenMap {
    pub fn new(screens: Vec<ScreenData>) -> Self {
        let mut map = HashMap::new();

        for (i, screen) in screens.iter().enumerate() {
            map.insert(screen.position, i);
        }

        Self {
            screens,
            map,
        }
    }

    pub fn get(&self, position: &Position) -> Option<&ScreenData> {
        self.map.get(position)
            .map(|i| &self.screens[*i])
    }

    pub fn len(&self) -> usize {
        self.screens.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ScreenData> {
        self.into_iter()
    }

    pub fn iter_positions(&self) -> impl Iterator<Item = &Position> {
        self.into_iter()
            .map(|screen| &screen.position)
    }
}

impl Index<usize> for ScreenMap {
    type Output = ScreenData;

    fn index(&self, index: usize) -> &Self::Output {
        &self.screens[index]
    }
}

impl IntoIterator for ScreenMap {
    type Item = ScreenData;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.screens.into_iter()
    }
}

impl<'a> IntoIterator for &'a ScreenMap {
    type Item = &'a ScreenData;
    type IntoIter = std::slice::Iter<'a, ScreenData>;

    fn into_iter(self) -> Self::IntoIter {
        self.screens.iter()
    }
}
