use crate::model::border::Border;
use crate::model::position::{CenterPlacement, Position};
use crate::model::rectangle::Rectangle;
use crate::model::vec2::Vec2;
use petgraph::algo::connected_components;
use petgraph::graphmap::UnGraphMap;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet, LinkedList};
use std::f64::consts::PI;
use std::fmt::{Display, Formatter};
use std::ops::Sub;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Galaxy {
    positions: HashSet<Position>,
}

/// A galaxy is a set of positions. A valid galaxy needs to satisfy the following conditions:
/// - It must not be empty
/// - It must be connected
/// - It must contain its center
/// - It must be rotationally symmetric
impl Galaxy {
    pub fn new() -> Galaxy {
        Galaxy {
            positions: HashSet::new(),
        }
    }

    pub fn get_borders(&self) -> impl IntoIterator<Item = Border> {
        let mut borders = HashSet::new();
        for p1 in self.get_positions() {
            for p2 in &p1.adjacent() {
                if !self.contains_position(p2) {
                    borders.insert(Border::new(*p1, *p2));
                }
            }
        }
        borders
    }

    /// Returns the number of positions in this galaxy
    pub fn size(&self) -> usize {
        self.positions.len()
    }

    /// Returns the center of this galaxy is half-steps.
    /// For example, the center of a galaxy containing [(0, 0)] is (0, 0),
    /// the center of a galaxy containing [(0, 0), (0, 1)] is (0, 1),
    /// and the center of a galaxy containing [(0, 1)] is (0, 2).
    ///
    /// If the galaxy is empty, (0, 0) is returned.
    pub fn center(&self) -> Position {
        let rect = self.bounding_rectangle();
        let center_half_row = rect.min_row + rect.max_row;
        let center_half_column = rect.min_column + rect.max_column;
        Position::new(center_half_row, center_half_column)
    }

    /// Returns the smallest rectangle that contains the galaxy.
    pub fn bounding_rectangle(&self) -> Rectangle {
        self.positions
            .iter()
            .fold(Option::<Rectangle>::default(), |acc, p| match acc {
                None => Some(Rectangle {
                    min_row: p.row,
                    max_row: p.row,
                    min_column: p.column,
                    max_column: p.column,
                }),
                Some(rect) => Some(Rectangle::new(
                    min(p.row, rect.min_row),
                    max(p.row, rect.max_row),
                    min(p.column, rect.min_column),
                    max(p.column, rect.max_column),
                )),
            })
            .unwrap_or(Rectangle::default())
    }

    pub fn mirror_position(&self, p: &Position) -> Position {
        let center = self.center();
        let mirrored_row = center.row - p.row;
        let mirrored_column = center.column - p.column;
        Position::new(mirrored_row, mirrored_column)
    }

    pub fn contains_position(&self, p: &Position) -> bool {
        self.positions.contains(p)
    }

    pub fn is_symmetric(&self) -> bool {
        self.positions
            .iter()
            .all(|p| self.contains_position(&self.mirror_position(p)))
    }

    pub fn is_connected(&self) -> bool {
        if self.positions.is_empty() {
            return true;
        }
        let mut graph = UnGraphMap::new();
        for p in self.positions.iter() {
            graph.add_node(*p);
            for adjacent in p.adjacent() {
                if self.contains_position(&adjacent) {
                    graph.add_edge(adjacent, *p, ());
                }
            }
        }
        connected_components(&graph) == 1
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    pub fn contains_center(&self) -> bool {
        let center = self.center();
        let rows = if center.row % 2 == 0 {
            vec![center.row / 2]
        } else {
            vec![center.row / 2, center.row / 2 + 1]
        };
        let columns = if center.column % 2 == 0 {
            vec![center.column / 2]
        } else {
            vec![center.column / 2, center.column / 2 + 1]
        };
        for &row in &rows {
            for &col in &columns {
                let p = Position::new(row, col);
                if !self.contains_position(&p) {
                    return false;
                }
            }
        }
        true
    }

    pub fn is_valid(&self) -> bool {
        !self.is_empty() && self.contains_center() && self.is_connected() && self.is_symmetric()
    }

    pub fn is_empty_or_valid(&self) -> bool {
        self.is_empty() || self.is_valid()
    }

    pub fn with_position(&self, p: &Position) -> Galaxy {
        let mut g = self.clone();
        g.positions.insert(*p);
        g
    }

    pub fn without_position(&self, p: &Position) -> Galaxy {
        let mut g = self.clone();
        g.positions.remove(p);
        g
    }

    /// Removes the given position from the galaxy, leaving it in a potentially invalid state
    pub fn remove_position(&mut self, p: &Position) {
        self.positions.remove(p);
    }

    /// Adds the given position from the galaxy, leaving it in a potentially invalid state
    pub fn add_position(&mut self, p: Position) {
        self.positions.insert(p);
    }

    /// Get adjacent positions that belong to the galaxy
    pub fn get_neighbours(&self, p: &Position) -> Vec<Position> {
        p.adjacent()
            .into_iter()
            .filter(|neighbour| self.contains_position(neighbour))
            .collect()
    }

    pub fn get_positions(&self) -> impl Iterator<Item = &Position> {
        self.positions.iter()
    }

    pub fn get_swirl(&self) -> f64 {
        let hamming_distances = self.get_hamming_distances();
        let center = Vec2::from(&self.center()) / 2.0;
        let vectors: HashMap<Position, Vec2> = self
            .positions
            .iter()
            .map(|p| (*p, Vec2::from(p) - center))
            .collect();

        let mut swirl = 0.0;
        for p in &self.positions {
            let v = vectors[&p];
            let hamming_distance = hamming_distances[&p];
            if hamming_distance != 0 {
                self.get_neighbours(&p)
                    .iter()
                    .filter(|n| hamming_distances[&n] < hamming_distance)
                    .map(|parent_position| vectors[&parent_position])
                    .filter(|parent_vector| !parent_vector.is_zero())
                    .map(|parent_vector| parent_vector.angle_to(&v))
                    .for_each(|angle_difference| swirl += angle_difference);
            }
        }

        swirl
    }

    pub fn get_curl(&self) -> f64 {
        let distances = self.get_hamming_distances();
        let children_map: HashMap<Position, Vec<Position>> = self
            .positions
            .iter()
            .copied()
            .map(|p| {
                let distance = distances[&p];
                let children: Vec<Position> = self
                    .get_neighbours(&p)
                    .into_iter()
                    .filter(|n| distances[n] == distance + 1)
                    .collect();
                (p, children)
            })
            .collect();
        let mut flows: HashMap<Position, Vec2> = HashMap::new();
        {
            let center = self.center();
            let parent = Vec2::from(&center) / 2.0;
            for child in center.get_center_placement().get_positions() {
                let v = (Vec2::from(&child) - parent).normalized();
                *flows.entry(child).or_insert(Vec2::ZERO) += v;
            }
        }
        for (parent, children) in &children_map {
            let parent = Vec2::from(parent);
            for child in children {
                let v = (Vec2::from(child) - parent).normalized();
                *flows.entry(*child).or_insert(Vec2::ZERO) += v
            }
        }
        for flow in flows.values_mut() {
            flow.normalize();
        }

        let curl: f64 = children_map
            .iter()
            .map(|(parent, children)| {
                let parent_flow = flows[&parent];
                children
                    .iter()
                    .map(|child| &flows[child])
                    .map(|child_flow| parent_flow.angle_to(child_flow))
                    .sum::<f64>()
            })
            .sum();

        curl
    }

    fn get_hamming_distances(&self) -> HashMap<Position, usize> {
        let mut queue: LinkedList<Position> = LinkedList::new();
        let mut hamming_distances: HashMap<Position, usize> = HashMap::new();
        for p in self.center().get_center_placement().get_positions() {
            hamming_distances.insert(p, 0);
            for n in self.get_neighbours(&p) {
                queue.push_back(n);
            }
        }
        while let Some(p) = queue.pop_front() {
            if hamming_distances.contains_key(&p) {
                continue;
            }
            let neighbours = self.get_neighbours(&p);
            let min_neighbour_distance = neighbours
                .iter()
                .filter_map(|n| hamming_distances.get(n))
                .min()
                .copied()
                .unwrap();
            hamming_distances.insert(p, min_neighbour_distance + 1);
            for n in neighbours {
                if !hamming_distances.contains_key(&n) {
                    queue.push_back(n);
                }
            }
        }
        hamming_distances
    }

    /// Returns the rectangles that make up the galaxy, by finding the largest rectangle, subtracting
    /// it from the galaxy, finding the next largest rectangle, and so forth.
    pub fn rectangles(&self) -> Vec<Rectangle> {
        Self::rectangles_internal(self.positions.clone())
    }

    fn rectangles_internal(mut positions: HashSet<Position>) -> Vec<Rectangle> {
        if positions.is_empty() {
            return vec![];
        }

        let min_col = positions.iter().map(|p| p.column).min().unwrap();
        let min_row = positions.iter().map(|p| p.row).min().unwrap();
        let max_col = positions.iter().map(|p| p.column).max().unwrap() + 1;
        let max_row = positions.iter().map(|p| p.row).max().unwrap() + 1;

        let width = max_col.abs_diff(min_col) as usize;
        let mut height = vec![0; width];
        let mut left = vec![min_col; width];
        let mut right = vec![max_col; width];

        let mut max_rectangle = Rectangle::default();

        for row in min_row..max_row {
            for col in min_col..max_col {
                let index = (col - min_col) as usize;
                let p = Position::new(row, col);
                if positions.contains(&p) {
                    height[index] += 1;
                } else {
                    height[index] = 0;
                }
            }
            let mut current_left = min_col;
            for col in min_col..max_col {
                let index = (col - min_col) as usize;
                let p = Position::new(row, col);
                if positions.contains(&p) {
                    left[index] = max(left[index], current_left);
                } else {
                    left[index] = 0;
                    current_left = col + 1;
                }
            }
            let mut current_right = max_col;
            for col in (min_col..max_col).rev() {
                let index = (col - min_col) as usize;
                let p = Position::new(row, col);
                if positions.contains(&p) {
                    right[index] = min(right[index], current_right);
                } else {
                    right[index] = max_col;
                    current_right = col;
                }
            }
            for col in min_col..max_col {
                let index = (col - min_col) as usize;
                let rect = Rectangle {
                    min_row: row - height[index] + 1,
                    max_row: row + 1,
                    min_column: left[index],
                    max_column: right[index],
                };
                if rect.area() > max_rectangle.area() {
                    max_rectangle = rect;
                }
            }
        }

        for p in &max_rectangle.positions() {
            positions.remove(p);
        }
        let mut rectangles = Self::rectangles_internal(positions);
        rectangles.push(max_rectangle);

        rectangles
    }
}

impl Display for Galaxy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bounds = self.bounding_rectangle();
        let positions: HashSet<Position> = self
            .get_positions()
            .map(|p| Position::new(p.row - bounds.min_row, p.column - bounds.min_column))
            .collect();
        for row in 0..=bounds.height() + 1 {
            for column in 0..=bounds.width() + 1 {
                let bottom_right = Position::from((row, column));
                let bottom_left = bottom_right.left();
                let top_left = bottom_left.up();
                let top_right = bottom_right.up();
                let has_top_left = positions.contains(&top_left);
                let has_top_right = positions.contains(&top_right);
                let has_bottom_left = positions.contains(&bottom_left);
                let has_bottom_right = positions.contains(&bottom_right);

                let bar_top = has_top_left != has_top_right;
                let bar_right = has_top_right != has_bottom_right;
                let bar_bottom = has_bottom_left != has_bottom_right;
                let bar_left = has_top_left != has_bottom_left;
                match (bar_top, bar_right, bar_bottom, bar_left) {
                    (false, false, false, false) => write!(f, "  ")?,
                    (false, false, false, true) => write!(f, "╴ ")?,
                    (false, false, true, false) => write!(f, "╷ ")?,
                    (false, false, true, true) => write!(f, "┐ ")?,
                    (false, true, false, false) => write!(f, "╶─")?,
                    (false, true, false, true) => write!(f, "──")?,
                    (false, true, true, false) => write!(f, "┌─")?,
                    (false, true, true, true) => write!(f, "┬─")?,
                    (true, false, false, false) => write!(f, "╵ ")?,
                    (true, false, false, true) => write!(f, "┘ ")?,
                    (true, false, true, false) => write!(f, "│ ")?,
                    (true, false, true, true) => write!(f, "┤ ")?,
                    (true, true, false, false) => write!(f, "└─")?,
                    (true, true, false, true) => write!(f, "┴─")?,
                    (true, true, true, false) => write!(f, "├─")?,
                    (true, true, true, true) => write!(f, "┼─")?,
                }
            }
            if row != bounds.height() + 1 {
                write!(f, "\n")?;
            }
        }

        Ok(())
    }
}

impl<I, T> From<I> for Galaxy
where
    I: IntoIterator<Item = T>,
    T: Into<Position>,
{
    fn from(positions: I) -> Self {
        Galaxy {
            positions: positions.into_iter().map(|p| p.into()).collect(),
        }
    }
}

impl From<&Rectangle> for Galaxy {
    fn from(rect: &Rectangle) -> Self {
        Self::from(rect.positions())
    }
}

#[cfg(test)]
mod tests {
    use crate::model::galaxy::Galaxy;
    use crate::model::position::Position;

    fn galaxy(positions: &[(i32, i32)]) -> Galaxy {
        Galaxy::from(positions.iter().map(|&p| Position::from(p)))
    }

    #[test]
    fn test_center() {
        assert_eq!(Position::new(0, 0), galaxy(&[(0, 0)]).center());
        assert_eq!(Position::new(0, 1), galaxy(&[(0, 0), (0, 1)]).center());
        assert_eq!(Position::new(0, 2), galaxy(&[(0, 1)]).center());
        assert_eq!(Position::new(1, 0), galaxy(&[(0, 0), (1, 0)]).center());
        assert_eq!(Position::new(2, 0), galaxy(&[(1, 0)]).center());
        assert_eq!(
            Position::new(0, 2),
            galaxy(&[(0, 0), (0, 1), (0, 2)]).center()
        );
        assert_eq!(
            Position::new(14, 6),
            galaxy(&[(6, 3), (7, 3), (8, 3)]).center()
        );
        assert_eq!(
            Position::new(14, 7),
            galaxy(&[(6, 3), (7, 3), (7, 4), (8, 4)]).center()
        );
        assert_eq!(
            Position::new(1, 1),
            galaxy(&[(0, 0), (0, 1), (1, 0), (1, 1)]).center()
        );
    }

    #[test]
    fn test_mirror_position() {
        assert_eq!(Position::new(0, 0), galaxy(&[(0, 0)]).center());
        assert_eq!(Position::new(0, 1), galaxy(&[(0, 0), (0, 1)]).center());
        assert_eq!(Position::new(0, 2), galaxy(&[(0, 1)]).center());
        assert_eq!(Position::new(1, 0), galaxy(&[(0, 0), (1, 0)]).center());
        assert_eq!(Position::new(2, 0), galaxy(&[(1, 0)]).center());
        assert_eq!(
            Position::new(0, 2),
            galaxy(&[(0, 0), (0, 1), (0, 2)]).center()
        );
        assert_eq!(
            Position::new(14, 6),
            galaxy(&[(6, 3), (7, 3), (8, 3)]).center()
        );
        assert_eq!(
            Position::new(14, 7),
            galaxy(&[(6, 3), (7, 3), (7, 4), (8, 4)]).center()
        );
        assert_eq!(
            Position::new(1, 1),
            galaxy(&[(0, 0), (0, 1), (1, 0), (1, 1)]).center()
        );
    }

    mod rectangles {
        use crate::model::galaxy::Galaxy;
        use crate::model::position::Position;
        use crate::model::rectangle::Rectangle;
        use itertools::Itertools;
        use proptest::proptest;

        #[test]
        fn empty_galaxy_should_have_no_rectangles() {
            let galaxy = Galaxy::new();
            assert_eq!(galaxy.rectangles(), vec![]);
        }

        proptest! {
            #[test]
            fn rectangle_galaxy_should_have_single_rectangle(rect: Rectangle) {
                if !rect.positions().is_empty() {
                    let galaxy = Galaxy::from(&rect);
                    let rects = galaxy.rectangles();
                    assert_eq!(rects.len(), 1);
                    assert_eq!(rects[0], rect);
                }
            }
        }

        #[test]
        fn s_galaxy() {
            /*
             *   ┌───┐    ┌─┬─┐
             *   │ ┌─┘ -> │ ├─┘
             * ┌─┘ │    ┌─┤ │
             * └───┘    └─┴─┘
             */
            let mut galaxy = Galaxy::new();
            galaxy.positions.insert(Position::new(0, 2));
            galaxy.positions.insert(Position::new(0, 1));
            galaxy.positions.insert(Position::new(1, 1));
            galaxy.positions.insert(Position::new(2, 1));
            galaxy.positions.insert(Position::new(2, 0));

            let actual: Vec<Rectangle> = galaxy.rectangles().into_iter().sorted().collect();
            let expected: Vec<Rectangle> = vec![
                Rectangle {
                    min_row: 2,
                    max_row: 3,
                    min_column: 0,
                    max_column: 1,
                },
                Rectangle {
                    min_row: 0,
                    max_row: 3,
                    min_column: 1,
                    max_column: 2,
                },
                Rectangle {
                    min_row: 0,
                    max_row: 1,
                    min_column: 2,
                    max_column: 3,
                },
            ]
            .into_iter()
            .sorted()
            .collect();
            assert_eq!(expected, actual);
        }
    }

    mod swirl {
        use crate::model::galaxy::Galaxy;
        use crate::model::position::Position;
        use crate::model::rectangle::Rectangle;
        use approx::assert_abs_diff_eq;
        use more_asserts::assert_gt;

        #[test]
        fn single_cell_should_have_zero_swirl() {
            let mut galaxy = Galaxy::new();
            galaxy.add_position(Position::ZERO);
            assert_eq!(galaxy.get_swirl(), 0.0);
        }

        #[test]
        fn rectangular_galaxy_should_have_zero_swirl() {
            for width in 1..10 {
                for height in 1..10 {
                    let galaxy = Galaxy::from(&Rectangle::from(&(width, height)));
                    let actual_swirl = galaxy.get_swirl();
                    assert_abs_diff_eq!(actual_swirl, 0.0, epsilon = 1e-8);
                }
            }
        }

        #[test]
        fn mirror_symmetrical_galaxy_should_have_zero_swirl() {
            #[rustfmt::skip]
            let galaxy = Galaxy::from(vec![
                (0, 0),         (0, 2),
                (1, 0), (1, 1), (1, 2),
                (2, 0),         (2, 2),
            ]);
            assert_abs_diff_eq!(galaxy.get_swirl(), 0.0, epsilon = 1e-8);

            #[rustfmt::skip]
            let galaxy = Galaxy::from(vec![
                (0, 0), (0, 1), (0, 2),
                        (1, 1),
                (2, 0), (2, 1), (2, 2),
            ]);
            assert_abs_diff_eq!(galaxy.get_swirl(), 0.0, epsilon = 1e-8);

            #[rustfmt::skip]
            let galaxy = Galaxy::from(vec![
                (0, 0), (0, 1), (0, 2),         (0, 4), (0, 5), (0, 6),
                (1, 0),         (1, 2),         (1, 4),         (1, 6),
                                (2, 2), (2, 3), (2, 4),
                                (3, 2), (3, 3), (3, 4),
                (4, 0),         (4, 2),         (4, 4),         (4, 6),
                (5, 0), (5, 1), (5, 2),         (5, 4), (5, 5), (5, 6),
            ]);
            assert_abs_diff_eq!(galaxy.get_swirl(), 0.0, epsilon = 1e-8);

            // #[rustfmt::skip]
            // let galaxy = Galaxy::from(vec![
            //     (0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7), (0, 8), (0, 9),
            //     (1, 0), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5), (1, 6), (1, 7), (1, 8), (1, 9),
            //     (2, 0), (2, 1), (2, 2), (2, 3), (2, 4), (2, 5), (2, 6), (2, 7), (2, 8), (2, 9),
            //     (3, 0), (3, 1), (3, 2), (3, 3), (3, 4), (3, 5), (3, 6), (3, 7), (3, 8), (3, 9),
            //     (4, 0), (4, 1), (4, 2), (4, 3), (4, 4), (4, 5), (4, 6), (4, 7), (4, 8), (4, 9),
            //     (5, 0), (5, 1), (5, 2), (5, 3), (5, 4), (5, 5), (5, 6), (5, 7), (5, 8), (5, 9),
            //     (6, 0), (6, 1), (6, 2), (6, 3), (6, 4), (6, 5), (6, 6), (6, 7), (6, 8), (6, 9),
            //     (7, 0), (7, 1), (7, 2), (7, 3), (7, 4), (7, 5), (7, 6), (7, 7), (7, 8), (7, 9),
            //     (8, 0), (8, 1), (8, 2), (8, 3), (8, 4), (8, 5), (8, 6), (8, 7), (8, 8), (8, 9),
            //     (9, 0), (9, 1), (9, 2), (9, 3), (9, 4), (9, 5), (9, 6), (9, 7), (9, 8), (9, 9),
            // ]);
        }

        #[test]
        fn s_shaped_galaxy_should_have_positive_swirl() {
            #[rustfmt::skip]
            let g1 = Galaxy::from(vec![
                (0, 0),
                (1, 0), (1, 1),
                        (2, 1),
            ]);
            assert_gt!(g1.get_swirl(), 0.0);

            #[rustfmt::skip]
            let g2 = Galaxy::from(vec![
                (0, 0),
                (1, 0),
                (2, 0), (2, 1), (2, 2),
                                (3, 2),
                                (4, 2),
            ]);
            assert_eq!(g2.get_swirl(), g1.get_swirl());

            #[rustfmt::skip]
            let g3 = Galaxy::from(vec![
                (0, 0), (0, 1),
                (1, 0),
                (2, 0), (2, 1), (2, 2),
                                (3, 2),
                        (4, 1), (4, 2),
            ]);
            assert_gt!(g3.get_swirl(), g2.get_swirl());

            #[rustfmt::skip]
            let g4 = Galaxy::from(vec![
                (0, 0), (0, 1), (0, 2),
                (1, 0),
                (2, 0), (2, 1), (2, 2),
                                (3, 2),
                (4, 0), (4, 1), (4, 2),
            ]);
            assert_gt!(g4.get_swirl(), g3.get_swirl());
        }
    }

    mod curl {
        use crate::model::galaxy::Galaxy;
        use crate::model::position::Position;
        use crate::model::rectangle::Rectangle;
        use approx::assert_abs_diff_eq;
        use more_asserts::assert_gt;

        #[test]
        fn single_cell_should_have_zero_curl() {
            let mut galaxy = Galaxy::new();
            galaxy.add_position(Position::ZERO);
            assert_eq!(galaxy.get_curl(), 0.0);
        }

        #[test]
        fn rectangular_galaxy_should_have_zero_curl() {
            for width in 1..10 {
                for height in 1..10 {
                    let galaxy = Galaxy::from(&Rectangle::from(&(width, height)));
                    let actual_curl = galaxy.get_curl();
                    assert_abs_diff_eq!(actual_curl, 0.0, epsilon = 1e-8);
                }
            }
        }

        #[test]
        fn mirror_symmetrical_galaxy_should_have_zero_curl() {
            #[rustfmt::skip]
            let galaxy = Galaxy::from(vec![
                (0, 0),         (0, 2),
                (1, 0), (1, 1), (1, 2),
                (2, 0),         (2, 2),
            ]);
            assert_abs_diff_eq!(galaxy.get_curl(), 0.0, epsilon = 1e-8);

            #[rustfmt::skip]
            let galaxy = Galaxy::from(vec![
                (0, 0), (0, 1), (0, 2),
                        (1, 1),
                (2, 0), (2, 1), (2, 2),
            ]);
            assert_abs_diff_eq!(galaxy.get_curl(), 0.0, epsilon = 1e-8);

            #[rustfmt::skip]
            let galaxy = Galaxy::from(vec![
                (0, 0), (0, 1), (0, 2),         (0, 4), (0, 5), (0, 6),
                (1, 0),         (1, 2),         (1, 4),         (1, 6),
                                (2, 2), (2, 3), (2, 4),
                                (3, 2), (3, 3), (3, 4),
                (4, 0),         (4, 2),         (4, 4),         (4, 6),
                (5, 0), (5, 1), (5, 2),         (5, 4), (5, 5), (5, 6),
            ]);
            assert_abs_diff_eq!(galaxy.get_curl(), 0.0, epsilon = 1e-8);

            // #[rustfmt::skip]
            // let galaxy = Galaxy::from(vec![
            //     (0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7), (0, 8), (0, 9),
            //     (1, 0), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5), (1, 6), (1, 7), (1, 8), (1, 9),
            //     (2, 0), (2, 1), (2, 2), (2, 3), (2, 4), (2, 5), (2, 6), (2, 7), (2, 8), (2, 9),
            //     (3, 0), (3, 1), (3, 2), (3, 3), (3, 4), (3, 5), (3, 6), (3, 7), (3, 8), (3, 9),
            //     (4, 0), (4, 1), (4, 2), (4, 3), (4, 4), (4, 5), (4, 6), (4, 7), (4, 8), (4, 9),
            //     (5, 0), (5, 1), (5, 2), (5, 3), (5, 4), (5, 5), (5, 6), (5, 7), (5, 8), (5, 9),
            //     (6, 0), (6, 1), (6, 2), (6, 3), (6, 4), (6, 5), (6, 6), (6, 7), (6, 8), (6, 9),
            //     (7, 0), (7, 1), (7, 2), (7, 3), (7, 4), (7, 5), (7, 6), (7, 7), (7, 8), (7, 9),
            //     (8, 0), (8, 1), (8, 2), (8, 3), (8, 4), (8, 5), (8, 6), (8, 7), (8, 8), (8, 9),
            //     (9, 0), (9, 1), (9, 2), (9, 3), (9, 4), (9, 5), (9, 6), (9, 7), (9, 8), (9, 9),
            // ]);
        }

        #[test]
        fn s_shaped_galaxy_should_have_positive_curl() {
            #[rustfmt::skip]
            let g1 = Galaxy::from(vec![
                (0, 0),
                (1, 0), (1, 1),
                        (2, 1),
            ]);
            assert_gt!(g1.get_curl(), 0.0);

            #[rustfmt::skip]
            let g2 = Galaxy::from(vec![
                (0, 0),
                (1, 0),
                (2, 0), (2, 1), (2, 2),
                                (3, 2),
                                (4, 2),
            ]);
            assert_eq!(g2.get_curl(), g1.get_curl());

            #[rustfmt::skip]
            let g3 = Galaxy::from(vec![
                (0, 0), (0, 1),
                (1, 0),
                (2, 0), (2, 1), (2, 2),
                                (3, 2),
                        (4, 1), (4, 2),
            ]);
            assert_gt!(g3.get_curl(), g2.get_curl());

            #[rustfmt::skip]
            let g4 = Galaxy::from(vec![
                (0, 0), (0, 1), (0, 2),
                (1, 0),
                (2, 0), (2, 1), (2, 2),
                                (3, 2),
                (4, 0), (4, 1), (4, 2),
            ]);
            assert_eq!(g4.get_curl(), g3.get_curl());
        }
    }
}
