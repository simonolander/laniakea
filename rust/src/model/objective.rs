use crate::model::border::Border;
use crate::model::position::Position;
use crate::model::universe::Universe;

#[derive(Debug, Copy, Clone)]
pub struct GalaxyCenter {
    pub position: Position,
    pub size: Option<usize>,
}

pub struct Objective {
    pub centers: Vec<GalaxyCenter>,
    pub walls: Vec<Border>,
}

impl Objective {
    pub fn generate(universe: &Universe) -> Self {
        let walls = Vec::new();
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
