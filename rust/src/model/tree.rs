use crate::model::position::Position;
use crate::model::rectangle::Rectangle;
use std::collections::hash_map::Iter;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tree {
    parents: HashMap<Position, Option<Position>>,
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            parents: HashMap::new(),
        }
    }

    pub fn from_parents(parents: impl IntoIterator<Item = (Position, Option<Position>)>) -> Self {
        Tree {
            parents: parents.into_iter().collect(),
        }
    }

    fn from_string(string: &str) -> Self {
        let tree = Tree::from_parents(string.lines().enumerate().flat_map(|(row, line)| {
            line.chars()
                .enumerate()
                .filter_map(move |(column, c)| match c {
                    '>' => Some((
                        Position::from((row, column)),
                        Some(Position::from((row, column + 1))),
                    )),
                    'v' => Some((
                        Position::from((row, column)),
                        Some(Position::from((row + 1, column))),
                    )),
                    '<' => Some((
                        Position::from((row, column)),
                        Some(Position::from((row, column - 1))),
                    )),
                    '^' => Some((
                        Position::from((row, column)),
                        Some(Position::from((row - 1, column))),
                    )),
                    ' ' => None,
                    _ => Some((Position::from((row, column)), None)),
                })
        }));
        tree
    }

    pub fn insert(&mut self, position: Position, parent: Option<Position>) {
        self.parents.insert(position, parent);
    }

    /// Returns whether this tree is valid.
    /// A tree is valid if all its parents are also present in the tree.
    pub fn is_valid(&self) -> bool {
        self.parents
            .values()
            .flatten()
            .all(|parent| self.parents.contains_key(parent))
    }

    pub fn contains(&self, position: &Position) -> bool {
        self.parents.contains_key(position)
    }

    /// Returns the parent of the given position, if any.
    /// Returns none if the position is not part of the tree.
    pub fn get_parent(&self, position: &Position) -> Option<Position> {
        self.parents.get(position).copied().unwrap_or(None)
    }

    /// Returns the nodes in the tree
    pub fn get_positions(&self) -> impl IntoIterator<Item = Position> {
        self.parents.keys().copied().collect::<Vec<Position>>()
    }

    pub fn iter(&self) -> Iter<Position, Option<Position>> {
        self.parents.iter()
    }

    pub fn to_string(&self) -> String {
        let bounds = Rectangle::bounding_rectangle(self.get_positions());
        let mut result = String::new();
        for row in bounds.min_row..=bounds.max_row + 1 {
            let mut result_line = String::new();
            for column in bounds.min_column..=bounds.max_column + 1 {
                let bottom_right = Position::new(row, column);
                let bottom_left = bottom_right.left();
                let top_left = bottom_left.up();
                let top_right = bottom_right.up();
                let contains_top_left = self.contains(&top_left);
                let contains_top_right = self.contains(&top_right);
                let contains_bottom_left = self.contains(&bottom_left);
                let contains_bottom_right = self.contains(&bottom_right);
                let top_border = contains_top_left != contains_top_right;
                let left_border = contains_top_left != contains_bottom_left;
                let right_border = contains_top_right != contains_bottom_right;
                let bottom_border = contains_bottom_left != contains_bottom_right;
                let top_left_parent = self.get_parent(&top_left);
                let top_right_parent = self.get_parent(&top_right);
                let bottom_left_parent = self.get_parent(&bottom_left);
                let bottom_right_parent = self.get_parent(&bottom_right);
                let top_parent = (contains_top_left && top_left_parent != Some(top_right))
                    && (contains_top_right && top_right_parent != Some(top_left));
                let left_parent = (contains_top_left && top_left_parent != Some(bottom_left))
                    && (contains_bottom_left && bottom_left_parent != Some(top_left));
                let right_parent = (contains_top_right && top_right_parent != Some(bottom_right))
                    && (contains_bottom_right && bottom_right_parent != Some(top_right));
                let bottom_parent = (contains_bottom_left
                    && bottom_left_parent != Some(bottom_right))
                    && (contains_bottom_right && bottom_right_parent != Some(bottom_left));

                let bar_top = top_border || top_parent;
                let bar_left = left_border || left_parent;
                let bar_right = right_border || right_parent;
                let bar_bottom = bottom_border || bottom_parent;
                let bars = match (bar_top, bar_right, bar_bottom, bar_left) {
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
            if row != bounds.max_row + 1 {
                result.push_str("\n");
            }
        }
        result.trim_end().to_string()
    }
}

impl FromIterator<(Position, Option<Position>)> for Tree {
    fn from_iter<I: IntoIterator<Item = (Position, Option<Position>)>>(iter: I) -> Self {
        Tree::from_parents(iter)
    }
}

#[cfg(test)]
mod tests {
    mod to_string {
        use crate::model::tree::Tree;
        use indoc::indoc;

        #[test]
        fn empty_tree() {
            assert_eq!(Tree::new().to_string(), "");
        }

        #[test]
        fn singleton() {
            assert_eq!(
                Tree::from_string(".").to_string(),
                indoc! {"
                    ┌─┐
                    └─┘"
                }
            );
        }

        #[test]
        fn cross() {
            assert_eq!(
                Tree::from_string(
                    "
                     v
                    >.<
                     ^
                    "
                )
                .to_string(),
                indoc! {"
                      ┌─┐
                    ┌─┘ └─┐
                    └─┐ ┌─┘
                      └─┘"
                }
            );
        }

        #[test]
        fn galaxy_1() {
            assert_eq!(
                Tree::from_string(
                    "
                    v<v<<
                    v v ^
                    >>.<<
                    v ^ ^
                    >>^>^
                    "
                )
                .to_string(),
                indoc! {"
                    ┌───┬─────┐
                    │ ┌─┤ ┌─┐ │
                    │ └─┘ └─┴─┤
                    ├─┬─┐ ┌─┐ │
                    │ └─┘ ├─┘ │
                    └─────┴───┘"
                }
            );
        }

        #[test]
        fn galaxy_2() {
            assert_eq!(
                Tree::from_string(
                    "
                    v<v<<
                    v^<<^
                    >>.<<
                    v>>v^
                    >>^>^
                    "
                )
                .to_string(),
                indoc! {"
                    ┌───┬─────┐
                    │ ╷ ╵ ╶─┐ │
                    │ └─────┴─┤
                    ├─┬─────┐ │
                    │ └─╴ ╷ ╵ │
                    └─────┴───┘"
                }
            );
        }

        #[test]
        fn cinnamon_bun() {
            assert_eq!(
                Tree::from_string(
                    "
                     v<<<<
                    vvv<<^
                    vv>.^^
                    v>>>^^
                    >>>>>^
                    "
                )
                .to_string(),
                indoc! {"
                      ┌─────────┐
                    ┌─┤ ┌─────┐ │
                    │ │ │ ╶─┐ │ │
                    │ │ └───┘ │ │
                    │ └───────┘ │
                    └───────────┘"
                }
            );
        }

        #[test]
        fn hashtag() {
            assert_eq!(
                Tree::from_string(
                    "
                     v  v
                    >v<<<<
                     v  ^
                     v  ^
                    >>>>^<
                     ^  ^
                    "
                )
                .to_string(),
                indoc! {"
                      ┌─┐   ┌─┐
                    ┌─┘ └───┘ └─┐
                    └─┐ ┌───┐ ┌─┘
                      │ │   │ │
                    ┌─┘ └───┘ └─┐
                    └─┐ ┌───┐ ┌─┘
                      └─┘   └─┘"
                }
            );
        }
    }
}
