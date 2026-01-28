use std::{collections::HashMap, ops::Deref};

use libks::{ScreenCoord, map_bin::ScreenData};

pub struct ScreenMap {
    screens: Vec<ScreenData>,
    indices: HashMap<ScreenCoord, usize>,
}

impl ScreenMap {
    pub fn new(screens: Vec<ScreenData>) -> Self {
        let mut indices = HashMap::new();

        for (i, screen) in screens.iter().enumerate() {
            indices.insert(screen.position, i);
        }

        Self {
            screens,
            indices,
        }
    }

    pub fn pos(&self, position: &ScreenCoord) -> Option<&ScreenData> {
        self.indices.get(position)
            .map(|i| &self.screens[*i])
    }
    
    pub fn index_of(&self, position: &ScreenCoord) -> Option<usize> {
        self.indices.get(position)
            .cloned()
    }

    pub fn iter_positions(&self) -> impl Iterator<Item = &ScreenCoord> {
        self.iter().map(|screen| &screen.position)
    }
}

impl Deref for ScreenMap {
    type Target = Vec<ScreenData>;

    fn deref(&self) -> &Self::Target {
        &self.screens
    }
}
