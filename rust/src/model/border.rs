use crate::model::position::Position;
use std::cmp::{max, min};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct Border {
    p1: Position,
    p2: Position,
}

impl Border {
    pub fn new(p1: Position, p2: Position) -> Self {
        debug_assert!(p1.is_adjacent_to(&p2));
        Self {
            p1: min(p1, p2),
            p2: max(p1, p2),
        }
    }

    pub fn p1(&self) -> Position {
        self.p1
    }

    pub fn p2(&self) -> Position {
        self.p2
    }

    pub fn is_vertical(&self) -> bool {
        self.p1.row == self.p2.row
    }
}

impl From<(Position, Position)> for Border {
    fn from((p1, p2): (Position, Position)) -> Self {
        Border::new(p1, p2)
    }
}

impl From<Border> for (Position, Position) {
    fn from(border: Border) -> Self {
        (border.p1, border.p2)
    }
}
