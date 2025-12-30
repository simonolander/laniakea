use crate::model::border::Border;
use crate::model::position::Position;
use crate::model::universe::Universe;
use itertools::Itertools;
use serde::Serialize;
use std::collections::{BTreeSet, HashSet};
use std::ops::Div;
use ts_rs::TS;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash, Serialize, TS)]
pub struct GalaxyCenter {
    pub position: Position,
    pub size: Option<usize>,
}

#[derive(Serialize, Clone, TS)]
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

    pub fn from_string(string: &str) -> Self {
        let centers = string
            .lines()
            .skip(1)
            .enumerate()
            .flat_map(|(half_row, line)| {
                line.chars()
                    .skip(1)
                    .enumerate()
                    .filter_map(move |(index, char)| {
                        if char == '●' {
                            let half_column = (index - 1) / 2;
                            Some(GalaxyCenter::from(Position::from((half_row, half_column))))
                        } else {
                            None
                        }
                    })
            })
            .collect();
        // let borders =
        Objective {
            centers,
            walls: HashSet::new(),
        }
    }

    pub fn to_string(&self) -> String {
        let width = 10;
        let height = 10;
        let center_positions: BTreeSet<Position> =
            self.centers.iter().map(|center| center.position).collect();
        let mut result = String::new();
        result.push_str("┌───");
        result.push_str(&"┬───".repeat(width - 1));
        result.push_str("┐\n");
        for row in 0..(height * 2) - 1 {
            result.push(if row % 2 == 0 { '│' } else { '├' });
            for column in 0..(width * 2) - 1 {
                result.push(' ');
                result.push(
                    if center_positions.contains(&Position::from((row, column))) {
                        '●'
                    } else if row % 2 == 1 && column % 2 == 1 {
                        '·'
                    } else {
                        ' '
                    },
                );
            }
            result.push(' ');
            result.push(if row % 2 == 0 { '│' } else { '┤' });
            result.push('\n');
        }
        result.push_str("└───");
        result.push_str(&"┴───".repeat(width - 1));
        result.push_str("┘");
        result
    }
}

impl From<Position> for GalaxyCenter {
    fn from(value: Position) -> Self {
        GalaxyCenter {
            position: value,
            size: None,
        }
    }
}

#[cfg(test)]
mod tests {
    mod from_string {
        use crate::model::objective::{GalaxyCenter, Objective};
        use crate::model::position::Position;
        use indoc::indoc;
        use std::collections::HashSet;

        #[test]
        pub fn should_parse_objective() {
            let objective = Objective::from_string(indoc! {
                "
                ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
                │         ●               ●       ●     │
                ├   ●   ·   ·   ·   ·   ·   ·   ·   · ● ┤
                │           ●                           │
                ├   ·   ·   ·   ·   · ● ·   ·   ·   ·   ┤
                │                                       │
                ├ ● ·   ·   ·   ·   ·   ·   ·   ·   ●   ┤
                │                                       │
                ├   ·   ·   ·   ·   ·   ·   ·   ·   ·   ┤
                │                       ●               │
                ├   ·   ·   ·   ·   ·   ·   ·   ·   ·   ┤
                │             ●                 ●       │
                ├   ·   ·   ·   ·   ·   ·   ·   ·   ·   ┤
                │   ●                   ●             ● │
                ├   ·   ·   ·   ·   ·   ·   ·   ·   ·   ┤
                │ ●                                     │
                ├   ·   ·   ·   ·   ·   ·   ·   ·   ·   ┤
                │             ●       ●     ●           │
                ├   ·   ·   ·   ·   ·   ·   ·   ·   · ● ┤
                │       ●           ●                   │
                └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘
                "
            });
            assert_eq!(
                objective.centers,
                HashSet::from_iter(
                    vec![
                        (0, 4),
                        (0, 12),
                        (0, 16),
                        (1, 1),
                        (1, 18),
                        (2, 5),
                        (3, 10),
                        (5, 0),
                        (5, 17),
                        (8, 11),
                        (10, 6),
                        (10, 15),
                        (12, 1),
                        (12, 11),
                        (12, 18),
                        (14, 0),
                        (16, 6),
                        (16, 10),
                        (16, 13),
                        (17, 18),
                        (18, 3),
                        (18, 9),
                    ]
                    .into_iter()
                    .map(Position::from)
                    .map(GalaxyCenter::from)
                )
            )
        }
    }
}
