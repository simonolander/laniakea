use crate::model::border::Border;
use crate::model::position::Position;
use crate::model::universe::Universe;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash, Serialize)]
pub struct GalaxyCenter {
    pub position: Position,
    pub size: Option<usize>,
}

#[derive(Serialize, Clone)]
pub struct Objective {
    pub centers: HashSet<GalaxyCenter>,
    pub walls: HashSet<Border>,
}

impl Objective {
    pub fn generate(universe: &Universe) -> Self {
        let walls = HashSet::new();
        let centers = universe
            .get_galaxies()
            .iter()
            .map(|galaxy| GalaxyCenter {
                position: galaxy.center(),
                size: None,
                // size: Some(galaxy.size()),
            })
            .collect();

        Objective { centers, walls }
    }
}
