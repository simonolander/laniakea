use crate::model::board_error::BoardError;
use crate::model::border::Border;
use crate::model::galaxy::Galaxy;
use crate::model::objective::Objective;
use crate::model::position::{CenterPlacement, Position};
use crate::model::rectangle::Rectangle;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    width: usize,
    height: usize,
    /// The borders of the board, including the outer frame and any internal borders.
    borders: BTreeSet<Border>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        // Initialize the borders to the large frame around the board
        let mut borders = BTreeSet::new();
        for row in 0..height {
            borders.insert(Border::left(Position::from((row, 0))));
            borders.insert(Border::left(Position::from((row, width))));
        }
        for column in 0..width {
            borders.insert(Border::up(Position::from((0, column))));
            borders.insert(Border::up(Position::from((height, column))));
        }
        Board {
            width,
            height,
            borders,
        }
    }

    pub fn from_iter<I: IntoIterator<Item = Border>>(iter: I) -> Self {
        let borders = BTreeSet::from_iter(iter);
        let width = borders
            .iter()
            .map(|border| border.p2().column)
            .max()
            .unwrap_or(0) as usize;
        let height = borders
            .iter()
            .map(|border| border.p2().row)
            .max()
            .unwrap_or(0) as usize;
        println!("{:?}", borders);
        Board {
            width,
            height,
            borders,
        }
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.width
    }

    pub fn contains(&self, position: &Position) -> bool {
        position.row >= 0
            && position.row < self.height as i32
            && position.column >= 0
            && position.column < self.width as i32
    }

    fn get_positions(&self) -> impl Iterator<Item = Position> {
        Rectangle::from_dimensions(self.width, self.height)
            .positions()
            .into_iter()
    }

    pub fn is_active(&self, border: &Border) -> bool {
        self.borders.contains(border)
    }

    pub fn is_border_within_bounds(&self, border: &Border) -> bool {
        self.contains(&border.p1()) && self.contains(&border.p2())
    }

    pub fn add_border(&mut self, border: Border) {
        self.is_border_within_bounds(&border) && self.borders.insert(border);
    }

    pub fn remove_border(&mut self, border: &Border) {
        self.is_border_within_bounds(border) && self.borders.remove(border);
    }

    /// Adds a wall between [p1] and [p2], returns true if the wall did not previously exist
    pub fn add_wall(&mut self, p1: Position, p2: Position) -> bool {
        debug_assert!(p1.is_adjacent_to(&p2));
        debug_assert!(self.contains(&p1));
        debug_assert!(self.contains(&p2));
        self.borders.insert(Border::new(p1, p2))
    }

    /// Removes the wall between [p1] and [p2], if it exists. Returns true if the wall existed
    pub fn remove_wall(&mut self, p1: Position, p2: Position) -> bool {
        debug_assert!(p1.is_adjacent_to(&p2));
        debug_assert!(self.contains(&p1));
        debug_assert!(self.contains(&p2));
        self.borders.remove(&Border::new(p1, p2))
    }

    /// Returns whether there is a wall between p1 and p2
    pub fn is_wall(&self, p1: Position, p2: Position) -> bool {
        self.borders.contains(&Border::new(p1, p2))
    }

    /// Toggles the wall between [p1] and [p2], returns true if there's a wall after the toggle
    pub fn toggle_wall(&mut self, p1: Position, p2: Position) -> bool {
        if self.is_wall(p1, p2) {
            self.remove_wall(p1, p2);
            false
        } else {
            self.add_wall(p1, p2);
            true
        }
    }

    pub fn get_borders(&self) -> impl Iterator<Item = Border> + use<'_> {
        self.borders.iter().copied()
    }

    pub fn get_interior_borders(&self) -> impl Iterator<Item = Border> {
        self.borders
            .iter()
            .copied()
            .filter(|border| self.is_border_within_bounds(border))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn get_galaxies(&self) -> Vec<Galaxy> {
        let mut galaxies = Vec::new();
        let mut remaining_positions: BTreeSet<Position> = self.get_positions().collect();
        while let Some(p) = remaining_positions.pop_first() {
            let mut component = HashSet::new();
            let mut queue = BTreeSet::new();
            queue.insert(p);
            while let Some(p) = queue.pop_first() {
                component.insert(p);
                remaining_positions.remove(&p);
                for neighbour in p.adjacent() {
                    if component.contains(&neighbour) {
                        continue;
                    }
                    if queue.contains(&neighbour) {
                        continue;
                    }
                    if !self.contains(&neighbour) {
                        continue;
                    }
                    if self.is_wall(p, neighbour) {
                        continue;
                    }
                    queue.insert(neighbour);
                }
            }
            galaxies.push(Galaxy::from(component));
        }

        galaxies
    }

    pub fn compute_error(&self, objective: &Objective) -> BoardError {
        let dangling_borders = self.get_dangling_borders().collect();

        let galaxies = self.get_galaxies();
        let galaxy_by_position: HashMap<Position, &Galaxy> = galaxies
            .iter()
            .flat_map(|galaxy| galaxy.get_positions().copied().map(move |p| (p, galaxy)))
            .collect();
        let galaxy_by_objective_center: HashMap<Position, &Galaxy> = objective
            .centers
            .iter()
            .map(|gc| {
                let some_position_around_center = match gc.position.get_center_placement() {
                    CenterPlacement::Center(p) => p,
                    CenterPlacement::VerticalBorder(b) => b.p1(),
                    CenterPlacement::HorizontalBorder(b) => b.p1(),
                    CenterPlacement::Intersection(r) => r.top_left(),
                };
                let &galaxy = galaxy_by_position
                    .get(&some_position_around_center)
                    .unwrap();
                (gc.position, galaxy)
            })
            .collect();

        let cut_centers: HashSet<Position> = objective
            .centers
            .iter()
            .map(|gc| gc.position)
            .filter(|center| self.is_center_cut(center))
            .collect();

        let incorrect_galaxy_sizes = objective
            .centers
            .iter()
            .filter_map(|gc| {
                if let Some(size) = gc.size {
                    let galaxy = galaxy_by_objective_center.get(&gc.position).unwrap();
                    if galaxy.size() != size {
                        Some(gc.position)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let asymmetric_centers = objective
            .centers
            .iter()
            .filter_map(|gc| {
                let galaxy = galaxy_by_objective_center.get(&gc.position).unwrap();
                if galaxy.center() != gc.position || !galaxy.is_valid() {
                    Some(gc.position)
                } else {
                    None
                }
            })
            .collect();

        let centerfull_cells: HashSet<Position> = galaxy_by_objective_center
            .values()
            .flat_map(|galaxy| galaxy.get_positions())
            .copied()
            .collect();
        let centerless_cells = self
            .get_positions()
            .filter(|p| !centerfull_cells.contains(p))
            .collect();

        BoardError {
            dangling_borders,
            incorrect_galaxy_sizes,
            centerless_cells,
            cut_centers,
            asymmetric_centers,
        }
    }

    fn is_center_cut(&self, center: &Position) -> bool {
        match center.get_center_placement() {
            CenterPlacement::Center(_) => false,
            CenterPlacement::VerticalBorder(border) => self.is_active(&border),
            CenterPlacement::HorizontalBorder(border) => self.is_active(&border),
            CenterPlacement::Intersection(rect) => {
                let top_left = Position::new(rect.min_row, rect.min_column);
                let top_right = Position::new(rect.min_row, rect.max_column);
                let bottom_left = Position::new(rect.max_row, rect.min_column);
                let bottom_right = Position::new(rect.max_row, rect.max_column);
                self.is_wall(top_left, top_right)
                    || self.is_wall(top_right, bottom_right)
                    || self.is_wall(bottom_right, bottom_left)
                    || self.is_wall(top_left, bottom_left)
            }
        }
    }

    fn get_dangling_borders(&self) -> impl Iterator<Item = Border> + use<'_> {
        self.get_borders().filter(|border| self.is_dangling(border))
    }

    fn is_dangling(&self, border: &Border) -> bool {
        let p1 = border.p1();
        let p2 = border.p2();
        if border.is_vertical() {
            // Check that the border connects to something above
            if p1.row != 0 {
                let p1_up = p1.up();
                let p2_up = p2.up();
                if !self.is_wall(p1, p1_up)
                    && !self.is_wall(p1_up, p2_up)
                    && !self.is_wall(p2_up, p2)
                {
                    return true;
                }
            }

            // Check that the border connects to something below
            if p1.row != (self.height - 1) as i32 {
                let p1_down = p1.down();
                let p2_down = p2.down();
                if !self.is_wall(p1, p1_down)
                    && !self.is_wall(p1_down, p2_down)
                    && !self.is_wall(p2_down, p2)
                {
                    return true;
                }
            }
        } else {
            // The border is horizontal

            // Check that the border connects to something to the left
            if p1.column != 0 {
                let p1_left = p1.left();
                let p2_left = p2.left();
                if !self.is_wall(p1, p1_left)
                    && !self.is_wall(p1_left, p2_left)
                    && !self.is_wall(p2_left, p2)
                {
                    return true;
                }
            }

            // Check that the border connects to something below
            if p1.column != (self.width - 1) as i32 {
                let p1_right = p1.right();
                let p2_right = p2.right();
                if !self.is_wall(p1, p1_right)
                    && !self.is_wall(p1_right, p2_right)
                    && !self.is_wall(p2_right, p2)
                {
                    return true;
                }
            }
        }

        false
    }

    /// Returns a matrix where m[row][column] is true if there is a border
    /// to the right of the cell (row, column).
    pub fn get_vertical_borders(&self) -> Vec<Vec<bool>> {
        let mut matrix = vec![vec![false; self.width - 1]; self.height];
        for border in self.get_interior_borders() {
            if border.is_vertical() {
                matrix[border.p1().row as usize][border.p1().column as usize] = true;
            }
        }
        matrix
    }

    /// Returns a matrix where m[row][column] is true if there is a border
    /// below the cell (row, column).
    pub fn get_horizontal_borders(&self) -> Vec<Vec<bool>> {
        let mut matrix = vec![vec![false; self.width]; self.height - 1];
        for border in self.get_interior_borders() {
            if border.is_horizontal() {
                matrix[border.p1().row as usize][border.p1().column as usize] = true;
            }
        }
        matrix
    }

    pub fn from_string(string: &str) -> Self {
        /*
        ┌───┬─┬───┬─┬─┬───┬─┐
        ├─┐ └─┼─┐ └─┴─┤   ├─┤
        ├─┤   └─┼───┐ └─┬─┘ │
        ├─┘   ┌─┘ ┌─┴─┬─┘   │
        ├─┐   ├───┤   │   ┌─┤
        │ └─┐ └─┬─┘ ╷ │ ┌─┘ │
        │   ├─┬─┘ ╶─┘ └─┤ ┌─┤
        ├─┐ ├─┤   ┌─╴ ┌─┴─┘ │
        │ └─┘ └─┐ ╵ ┌─┤     │
        ├─┐ ┌─┐ ├─┐ ├─┤ ┌─┬─┤
        └─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘
         */

        let width = string
            .lines()
            .next()
            .map(|line| (line.chars().count() - 1) / 2)
            .unwrap_or(1);
        let height = string.lines().count() - 1;
        let mut borders = BTreeSet::<Border>::new();
        for (row, line) in string.lines().enumerate() {
            for (column, char) in line.chars().step_by(2).enumerate() {
                let (top, right, bottom, left) = match char {
                    '┼' => (true, true, true, true),
                    '├' => (true, true, true, false),
                    '┴' => (true, true, false, true),
                    '└' => (true, true, false, false),
                    '┤' => (true, false, true, true),
                    '│' => (true, false, true, false),
                    '┘' => (true, false, false, true),
                    '╵' => (true, false, false, false),
                    '┬' => (false, true, true, true),
                    '┌' => (false, true, true, false),
                    '─' => (false, true, false, true),
                    '╶' => (false, true, false, false),
                    '┐' => (false, false, true, true),
                    '╷' => (false, false, true, false),
                    '╴' => (false, false, false, true),
                    ' ' => (false, false, false, false),
                    _ => (false, false, false, false),
                };
                let bottom_right = Position::from((row, column));
                if right {
                    borders.insert(Border::up(bottom_right));
                }
                if bottom {
                    borders.insert(Border::left(bottom_right));
                }
            }
        }

        Self {
            width,
            height,
            borders,
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity((self.width + 1) * (self.height + 1) * 2);
        for row in 0..=self.height {
            let mut result_line = String::new();
            for column in 0..=self.width {
                let bottom_right = Position::from((row, column));
                let top_left = bottom_right.left().up();
                let top = self.is_active(&Border::right(top_left));
                let left = self.is_active(&Border::down(top_left));
                let right = self.is_active(&Border::up(bottom_right));
                let bottom = self.is_active(&Border::left(bottom_right));
                let bars = match (top, right, bottom, left) {
                    (false, false, false, false) => "  ",
                    (false, false, false, true) => "╴ ",
                    (false, false, true, false) => "╷ ",
                    (false, false, true, true) => "┐ ",
                    (false, true, false, false) => "╶─",
                    (false, true, false, true) => "──",
                    (false, true, true, false) => "┌─",
                    (false, true, true, true) => "┬─",
                    (true, false, false, false) => "╵ ",
                    (true, false, false, true) => "┘ ",
                    (true, false, true, false) => "│ ",
                    (true, false, true, true) => "┤ ",
                    (true, true, false, false) => "└─",
                    (true, true, false, true) => "┴─",
                    (true, true, true, false) => "├─",
                    (true, true, true, true) => "┼─",
                };
                result_line.push_str(bars);
            }
            result.push_str(result_line.trim_end());
            if row != self.height {
                result.push('\n');
            }
        }
        result
    }
}

impl Display for &Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&Board::to_string(&self))
    }
}

#[cfg(test)]
mod tests {
    mod get_galaxies {
        use crate::model::board::Board;

        #[test]
        fn empty_board_should_return_one_galaxy() {
            let board = Board::new(1, 1);
            let galaxies = board.get_galaxies();
            assert_eq!(galaxies.len(), 1);
            assert_eq!(galaxies[0].size(), 1);
        }
    }

    mod to_string {
        use crate::model::board::Board;
        use indoc::indoc;

        #[test]
        fn empty_board_should_be_empty_string() {
            let board = Board::new(0, 0);
            assert_eq!(board.to_string(), "");
        }

        #[test]
        fn varous_boards_with_only_frames() {
            assert_eq!(
                Board::new(1, 1).to_string(),
                indoc! {"
                    ┌─┐
                    └─┘"
                }
            );
            assert_eq!(
                Board::new(2, 1).to_string(),
                indoc! {"
                    ┌───┐
                    └───┘"
                }
            );
            assert_eq!(
                Board::new(1, 2).to_string(),
                indoc! {"
                    ┌─┐
                    │ │
                    └─┘"
                }
            );
            assert_eq!(
                Board::new(2, 2).to_string(),
                indoc! {"
                    ┌───┐
                    │   │
                    └───┘"
                }
            );
            assert_eq!(
                Board::new(3, 3).to_string(),
                indoc! {"
                    ┌─────┐
                    │     │
                    │     │
                    └─────┘"
                }
            );
        }
    }

    mod from_string {
        use crate::model::board::Board;
        use indoc::indoc;

        #[test]
        fn from_and_to_string_should_return_same_board() {
            let string = indoc! {"
                ┌───┬─┬───┬─┬─┬───┬─┐
                ├─┐ └─┼─┐ └─┴─┤   ├─┤
                ├─┤   └─┼───┐ └─┬─┘ │
                ├─┘   ┌─┘ ┌─┴─┬─┘   │
                ├─┐   ├───┤   │   ┌─┤
                │ └─┐ └─┬─┘ ╷ │ ┌─┘ │
                │   ├─┬─┘ ╶─┘ └─┤ ┌─┤
                ├─┐ ├─┤   ┌─╴ ┌─┴─┘ │
                │ └─┘ └─┐ ╵ ┌─┤     │
                ├─┐ ┌─┐ ├─┐ ├─┤ ┌─┬─┤
                └─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘"
            };
            assert_eq!(Board::from_string(string).to_string(), string);
        }
    }
}
