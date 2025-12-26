use crate::model::border::Border;
use crate::model::position::CenterPlacement::{
    Center, HorizontalBorder, Intersection, VerticalBorder,
};
use crate::model::rectangle::Rectangle;
use rand::Rng;
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::ops::Sub;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, Hash, Serialize)]
pub struct Position {
    pub row: i32,
    pub column: i32,
}

#[derive(Eq, PartialEq, Debug)]
pub enum CenterPlacement {
    Center(Position),
    VerticalBorder(Border),
    HorizontalBorder(Border),
    Intersection(Rectangle),
}

impl CenterPlacement {
    pub fn get_positions(&self) -> Vec<Position> {
        match self {
            Center(p) => vec![p.clone()],
            VerticalBorder(b) => vec![b.p1(), b.p2()],
            HorizontalBorder(b) => vec![b.p1(), b.p2()],
            Intersection(r) => r.corners(),
        }
    }
}

impl Position {
    pub const ZERO: Position = Position { row: 0, column: 0 };

    pub fn new(row: i32, column: i32) -> Position {
        Position { row, column }
    }

    pub fn random(width: usize, height: usize, rng: &mut impl Rng) -> Position {
        let row = rng.gen_range(0..height) as i32;
        let column = rng.gen_range(0..width) as i32;
        Position { row, column }
    }

    pub fn up(&self) -> Position {
        Position {
            row: self.row - 1,
            ..*self
        }
    }

    pub fn right(&self) -> Position {
        Position {
            column: self.column + 1,
            ..*self
        }
    }

    pub fn down(&self) -> Position {
        Position {
            row: self.row + 1,
            ..*self
        }
    }

    pub fn left(&self) -> Position {
        Position {
            column: self.column - 1,
            ..*self
        }
    }

    /// Get positions above, below, and to the left and right of this position
    pub fn adjacent(&self) -> Vec<Position> {
        vec![self.up(), self.right(), self.down(), self.left()]
    }

    /// Returns true iff other is directly above, below, to the left, or to the right of this position.
    pub fn is_adjacent_to(&self, other: &Position) -> bool {
        let delta_row = self.row.abs_diff(other.row);
        let delta_column = self.column.abs_diff(other.column);
        delta_row
            .checked_add(delta_column)
            .map(|it| it == 1)
            .unwrap_or(false)
    }

    /// Interpreting this position as a center, i.e. a position that could lie on borders,
    /// returns the positions of the cells surrounding this center
    pub fn get_center_placement(&self) -> CenterPlacement {
        let r1 = self.row / 2;
        let c1 = self.column / 2;
        if self.row % 2 == 0 {
            if self.column % 2 == 0 {
                Center(Position::new(r1, c1))
            } else {
                let c2 = c1 + 1;
                VerticalBorder(Border::new(Position::new(r1, c1), Position::new(r1, c2)))
            }
        } else {
            let r2 = r1 + 1;
            if self.column % 2 == 0 {
                HorizontalBorder(Border::new(Position::new(r1, c1), Position::new(r2, c1)))
            } else {
                let c2 = c1 + 1;
                Intersection(Rectangle::new(r1, r2, c1, c2))
            }
        }
    }

    /// Mirrors the position horizontally and vertically with respect
    /// to this position, interpreted as half-steps
    pub fn mirror_position(&self, p: &Position) -> Position {
        let mirrored_row = self.row - p.row;
        let mirrored_column = self.column - p.column;
        Position::new(mirrored_row, mirrored_column)
    }

    /// Returns the index of this position in a grid with the given width
    pub fn to_index(&self, width: usize) -> usize {
        self.row as usize * width + self.column as usize
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.column)
    }
}

impl From<(usize, usize)> for Position {
    fn from((row, column): (usize, usize)) -> Self {
        Position::new(row as i32, column as i32)
    }
}

impl From<(i32, i32)> for Position {
    fn from((row, column): (i32, i32)) -> Self {
        Position::new(row, column)
    }
}

impl<'a, 'b> Sub<&'b Position> for &'a Position {
    type Output = Position;
    fn sub(self, rhs: &'b Position) -> Position {
        Position::new(self.row - rhs.row, self.column - rhs.column)
    }
}

impl Sub<&Position> for Position {
    type Output = Position;
    fn sub(self, rhs: &Position) -> Position {
        &self - rhs
    }
}

impl Sub<Position> for &Position {
    type Output = Position;
    fn sub(self, rhs: Position) -> Position {
        self - &rhs
    }
}

impl Sub<Position> for Position {
    type Output = Position;
    fn sub(self, rhs: Position) -> Position {
        &self - &rhs
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use crate::model::position::Position;
    use proptest::prelude::*;
    use rand::thread_rng;

    fn prop_assert_eq_vec_orderless<T: Eq + Debug>(
        left: Vec<T>,
        right: Vec<T>,
    ) -> Result<(), TestCaseError> {
        prop_assert_eq!(
            left.len(),
            right.len(),
            "assertion failed: `(left.len() == right.len())`\n  left: `{:?}`,\n right: `{:?}`",
            left,
            right
        );
        for item in &left {
            let left_count = left.iter().filter(|&it| it == item).count();
            let right_count = right.iter().filter(|&it| it == item).count();
            prop_assert_eq!(left_count, right_count, "assertion failed: `(left.count({:?}) == right.count({:?}))`\n  left: `{:?}`,\n right: `{:?}`", item, item, left, right);
        }
        Ok(())
    }

    mod get_center_placement {
        use crate::model::position::{CenterPlacement, Position};

        #[test]
        fn center_within_cell_should_return_that_cell() {
            assert_eq!(
                Position::new(0, 0).get_center_placement(),
                CenterPlacement::Center(Position::new(0, 0))
            );
            assert_eq!(
                Position::new(0, 2).get_center_placement(),
                CenterPlacement::Center(Position::new(0, 1))
            );
            assert_eq!(
                Position::new(2, 0).get_center_placement(),
                CenterPlacement::Center(Position::new(1, 0))
            );
            assert_eq!(
                Position::new(2, 2).get_center_placement(),
                CenterPlacement::Center(Position::new(1, 1))
            );
            assert_eq!(
                Position::new(10, 10).get_center_placement(),
                CenterPlacement::Center(Position::new(5, 5))
            );
            assert_eq!(
                Position::new(10, 20).get_center_placement(),
                CenterPlacement::Center(Position::new(5, 10))
            );
            assert_eq!(
                Position::new(20, 10).get_center_placement(),
                CenterPlacement::Center(Position::new(10, 5))
            );
            assert_eq!(
                Position::new(20, 20).get_center_placement(),
                CenterPlacement::Center(Position::new(10, 10))
            );
        }
    }

    proptest! {
        #[test]
        fn test_fmt(row: i32, col: i32) {
            let p = Position::new(row, col);
            let expected = format!("({}, {})", row, col);
            prop_assert_eq!(expected, p.to_string());
        }

        #[test]
        fn test_up(row: i32, col: i32) {
            prop_assume!(row != i32::MIN);
            let p = Position::new(row, col);
            let expected = Position::new(row - 1, col);
            prop_assert_eq!(expected, p.up());
        }

        #[test]
        fn test_right(row: i32, col: i32) {
            prop_assume!(col != i32::MAX);
            let p = Position::new(row, col);
            let expected = Position::new(row, col + 1);
            prop_assert_eq!(expected, p.right());
        }

        #[test]
        fn test_down(row: i32, col: i32) {
            prop_assume!(row != i32::MAX);
            let p = Position::new(row, col);
            let expected = Position::new(row + 1, col);
            prop_assert_eq!(expected, p.down());
        }

        #[test]
        fn test_left(row: i32, col: i32) {
            prop_assume!(col != i32::MIN);
            let p = Position::new(row, col);
            let expected = Position::new(row, col - 1);
            prop_assert_eq!(expected, p.left());
        }

        #[test]
        fn test_random(width in 1..i32::MAX, height in 1..i32::MAX) {
            let p = Position::random(width as usize, height as usize, &mut thread_rng());
            prop_assert!(p.column >= 0);
            prop_assert!(p.column < width);
            prop_assert!(p.row >= 0);
            prop_assert!(p.row < height);
        }

        #[test]
        fn test_adjacent(row: i32, col: i32) {
            prop_assume!(row != i32::MIN && row != i32::MAX);
            prop_assume!(col != i32::MIN && col != i32::MAX);
            let p = Position::new(row, col);
            let adjacent = p.adjacent();
            prop_assert_eq_vec_orderless(adjacent, vec![p.left(), p.up(), p.down(), p.right()])?;
        }

        #[test]
        fn test_is_adjacent_to_should_not_be_adjacent_to_self(row: i32, col: i32) {
            let p = Position::new(row, col);
            prop_assert!(!p.is_adjacent_to(&p));
        }

        #[test]
        fn test_is_adjacent_to_should_be_adjacent_to_left_up_down_right(row: i32, col: i32) {
            prop_assume!(row != i32::MIN && row != i32::MAX);
            prop_assume!(col != i32::MIN && col != i32::MAX);
            let p = Position::new(row, col);
            prop_assert!(p.is_adjacent_to(&p.left()));
            prop_assert!(p.is_adjacent_to(&p.up()));
            prop_assert!(p.is_adjacent_to(&p.down()));
            prop_assert!(p.is_adjacent_to(&p.right()));
        }

        #[test]
        fn test_is_adjacent_to_should_not_be_adjacent_to_anything_but_left_up_down_right(r1: i32, c1: i32, r2: i32, c2: i32) {
            prop_assume!(r1 != i32::MIN && r1 != i32::MAX);
            prop_assume!(c1 != i32::MIN && c1 != i32::MAX);
            prop_assume!(r2 != i32::MIN && r2 != i32::MAX);
            prop_assume!(c2 != i32::MIN && c2 != i32::MAX);
            let p1 = Position::new(r1, c1);
            let p2 = Position::new(r2, c2);
            prop_assume!(p2 != p1.left());
            prop_assume!(p2 != p1.up());
            prop_assume!(p2 != p1.right());
            prop_assume!(p2 != p1.down());
            prop_assert!(!p1.is_adjacent_to(&p2));
        }

        #[test]
        fn test_is_adjacent_is_symmetric(r1: i32, c1: i32, r2: i32, c2: i32) {
            let p1 = Position::new(r1, c1);
            let p2 = Position::new(r2, c2);
            prop_assert_eq!(p1.is_adjacent_to(&p2), p2.is_adjacent_to(&p1));
        }

        #[test]
        fn test_from_usize_usize(row: i32, col: i32) {
            let tuple = (row as usize, col as usize);
            prop_assert_eq!(Position::from(tuple), Position::new(row, col));
        }
    }

    mod ord {
        use crate::model::position::Position;
        use proptest::{prop_assert, prop_assume, proptest};

        proptest! {
            #[test]
            fn order_by_row(r1: i32, c1: i32, r2: i32, c2: i32) {
                prop_assume!(r1 < r2);
                let p1 = Position::new(r1, c1);
                let p2 = Position::new(r2, c2);

                prop_assert!(p1 < p2);
            }

            #[test]
            fn if_same_row_then_order_by_column(row: i32, c1: i32, c2: i32) {
                prop_assume!(c1 < c2);
                let p1 = Position::new(row, c1);
                let p2 = Position::new(row, c2);

                prop_assert!(p1 < p2);
            }
        }
    }

    mod eq {
        use crate::model::position::Position;
        use proptest::{prop_assert_eq, prop_assert_ne, prop_assume, proptest};

        proptest! {
            #[test]
            fn should_be_equal_if_row_and_column_is_equal(row: i32, col: i32) {
                let p1 = Position::new(row, col);
                let p2 = Position::new(row, col);

                prop_assert_eq!(p1, p2);
            }

            #[test]
            fn should_not_be_equal_if_rows_are_different(r1: i32, r2: i32, col: i32) {
                prop_assume!(r1 != r2);
                let p1 = Position::new(r1, col);
                let p2 = Position::new(r2, col);

                prop_assert_ne!(p1, p2);
            }

            #[test]
            fn should_not_be_equal_if_columns_are_different(row: i32, c1: i32, c2: i32) {
                prop_assume!(c1 != c2);
                let p1 = Position::new(row, c1);
                let p2 = Position::new(row, c2);

                prop_assert_ne!(p1, p2);
            }
        }
    }
}
