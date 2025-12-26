use crate::model::border::Border;
use crate::model::position::Position;
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct BoardError {
    pub dangling_borders: HashSet<Border>,
    pub incorrect_galaxy_sizes: HashSet<Position>,
    pub centerless_cells: HashSet<Position>,
    pub cut_centers: HashSet<Position>,
    pub asymmetric_centers: HashSet<Position>,
}

impl BoardError {
    pub fn none() -> BoardError {
        BoardError::default()
    }

    pub fn is_error_free(&self) -> bool {
        self.dangling_borders.is_empty()
            && self.incorrect_galaxy_sizes.is_empty()
            && self.centerless_cells.is_empty()
            && self.asymmetric_centers.is_empty()
            && self.cut_centers.is_empty()
    }
}
