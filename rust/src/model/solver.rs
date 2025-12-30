use crate::model::border::Border;
use crate::model::objective::{GalaxyCenter, Objective};
use crate::model::position::Position;
use crate::model::rectangle::Rectangle;
use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

type GalaxyId = usize;

#[derive(Debug)]
pub struct Contradiction;

#[derive(Clone)]
pub struct Solver {
    width: usize,
    height: usize,
    galaxy_centers: Vec<GalaxyCenter>,
    borders: BTreeMap<Border, bool>,
    possible_galaxy_ids: BTreeMap<Position, BTreeSet<GalaxyId>>,
}

pub struct Solution {
    pub(crate) borders: BTreeSet<Border>,
}

impl Solver {
    pub fn new(width: usize, height: usize, objective: &Objective) -> Self {
        let galaxy_centers: Vec<GalaxyCenter> = objective.centers.iter().copied().collect();

        // We initialize all borders to unknown
        let mut borders = BTreeMap::new();

        // We know all the borders in the objective are active
        for &border in &objective.walls {
            borders.insert(border, true);
        }

        // We know that all the borders in the frame are active
        for column in 0..width {
            borders.insert(Border::up(Position::from((0, column))), true);
            borders.insert(Border::up(Position::from((height, column))), true);
        }
        for row in 0..height {
            borders.insert(Border::left(Position::from((row, 0))), true);
            borders.insert(Border::left(Position::from((row, width))), true);
        }

        // We initialize all the possible galaxy IDs to every galaxy id
        let mut possible_galaxy_ids = Rectangle::from_dimensions(width, height)
            .positions()
            .into_iter()
            .map(|p| (p, BTreeSet::from_iter(0..galaxy_centers.len())))
            .collect::<BTreeMap<_, _>>();

        // We know that all cells around the galaxy centers belong to that specific galaxy
        for (id, center) in galaxy_centers.iter().enumerate() {
            for position in center.position.get_center_placement().get_positions() {
                possible_galaxy_ids
                    .get_mut(&position)
                    .unwrap()
                    .retain(|&galaxy_id| galaxy_id == id);
            }
        }

        Solver {
            width,
            height,
            galaxy_centers,
            borders,
            possible_galaxy_ids,
        }
    }

    pub fn solve(&mut self) -> Result<Solution, Contradiction> {
        loop {
            if self.add_borders_between_known_galaxies()? {
                continue;
            };
            if self.mirror_borders()? {
                continue;
            };
            if self.exclude_unreachable_galaxies()? {
                continue;
            };
            if self.remove_impossible_galaxy_mirrors()? {
                continue;
            };
            if self.assume_galaxy()? {
                continue;
            };
            break;
        }
        let borders = self.get_borders();
        Ok(Solution { borders })
    }

    fn get_borders(&self) -> BTreeSet<Border> {
        self.borders
            .iter()
            .filter_map(
                |(&border, &active)| {
                    if active {
                        Some(border)
                    } else {
                        None
                    }
                },
            )
            .collect()
    }

    fn get_cells_with_certain_galaxy_id(&self) -> impl IntoIterator<Item = (Position, GalaxyId)> {
        self.possible_galaxy_ids
            .iter()
            .filter_map(|(&position, galaxy_ids)| {
                galaxy_ids
                    .iter()
                    .exactly_one()
                    .ok()
                    .map(|&id| (position, id))
            })
            .collect::<Vec<_>>()
    }

    /// For cells that certainly belong to a galaxy, we can mirror all the borders along the galaxy center.
    fn mirror_borders(&mut self) -> Result<bool, Contradiction> {
        /*
         * For each cell for which we're certain of the galaxy membership,
         * we can mirror all the borders along the center of that galaxy.
         * This also works if the mirror position is the same as the original position.
         * In the case that the mirrored border disagrees with the original,
         * an error is returned, indicating that some assumption previously taken is incorrect.
         */
        let mut changed = false;
        for (position, galaxy_id) in self.get_cells_with_certain_galaxy_id() {
            let center_position = self.galaxy_centers[galaxy_id].position;
            let mirrored_position = center_position.mirror_position(&position);
            for (border, mirrored_border) in [
                (Border::up(position), Border::down(mirrored_position)),
                (Border::left(position), Border::right(mirrored_position)),
                (Border::right(position), Border::left(mirrored_position)),
                (Border::down(position), Border::up(mirrored_position)),
            ] {
                if let Some(&has_border) = self.borders.get(&border) {
                    if let Some(&has_mirrored_border) = self.borders.get(&mirrored_border) {
                        if has_border != has_mirrored_border {
                            return Err(Contradiction);
                        }
                    } else {
                        self.borders.insert(mirrored_border, has_border);
                        changed = true;
                    }
                }
            }
        }
        Ok(changed)
    }

    /// Cells that belong to different galaxies should have a border between them,
    /// and cells that belong to the same galaxy should not.
    fn add_borders_between_known_galaxies(&mut self) -> Result<bool, Contradiction> {
        let mut changed = false;
        for (position, galaxy_id) in self.get_cells_with_certain_galaxy_id() {
            for neighbour in position.adjacent() {
                if let Some(&neighbour_galaxy_id) = self
                    .possible_galaxy_ids
                    .get(&neighbour)
                    .map(|galaxy_ids| galaxy_ids.iter().exactly_one().ok())
                    .flatten()
                {
                    let border = Border::new(position, neighbour);
                    let should_have_border = galaxy_id != neighbour_galaxy_id;
                    if let Some(&has_border) = self.borders.get(&border) {
                        if has_border != should_have_border {
                            return Err(Contradiction);
                        }
                    } else {
                        self.borders.insert(border, should_have_border);
                        changed = true;
                    }
                }
            }
        }
        Ok(changed)
    }

    fn exclude_unreachable_galaxies(&mut self) -> Result<bool, Contradiction> {
        let mut changed = false;
        let all_cells =
            BTreeSet::from_iter(Rectangle::from_dimensions(self.width, self.height).positions());
        for (galaxy_id, galaxy_center) in self.galaxy_centers.iter().enumerate() {
            let mut queue = VecDeque::from_iter(
                galaxy_center
                    .position
                    .get_center_placement()
                    .get_positions(),
            );
            let mut visited = BTreeSet::from_iter(queue.clone());
            while let Some(position) = queue.pop_front() {
                for neighbour in position.adjacent() {
                    let border = Border::new(position, neighbour);
                    if self.borders.get(&border).copied().unwrap_or(false) {
                        continue;
                    }
                    if !self
                        .possible_galaxy_ids
                        .get(&neighbour)
                        .unwrap()
                        .contains(&galaxy_id)
                    {
                        continue;
                    }
                    if visited.insert(neighbour) {
                        queue.push_back(neighbour);
                    }
                }
            }
            for position in all_cells.difference(&visited) {
                let galaxy_ids = self.possible_galaxy_ids.get_mut(&position).unwrap();
                changed |= galaxy_ids.remove(&galaxy_id);
                if galaxy_ids.is_empty() {
                    return Err(Contradiction);
                }
            }
        }
        Ok(changed)
    }

    fn remove_impossible_galaxy_mirrors(&mut self) -> Result<bool, Contradiction> {
        let mut changed = false;
        for (position, galaxy_ids) in self.possible_galaxy_ids.clone() {
            for galaxy_id in galaxy_ids {
                let center = self.galaxy_centers[galaxy_id].position;
                let mirrored_position = center.mirror_position(&position);
                /*
                 * If the mirrored position does not contain the galaxy_id,
                 * or if the mirrored position is outside the board, remove the galaxy id.
                 */
                if self
                    .possible_galaxy_ids
                    .get(&mirrored_position)
                    .map(|mirrored_galaxy_ids| !mirrored_galaxy_ids.contains(&galaxy_id))
                    .unwrap_or(true)
                {
                    let galaxy_ids = self.possible_galaxy_ids.get_mut(&position).unwrap();
                    changed |= galaxy_ids.remove(&galaxy_id);
                    if galaxy_ids.is_empty() {
                        return Err(Contradiction);
                    }
                }
            }
        }
        Ok(changed)
    }

    fn assume_galaxy(&mut self) -> Result<bool, Contradiction> {
        let positions_with_multiple_possible_galaxies = self
            .possible_galaxy_ids
            .iter()
            .filter(|(_, galaxy_ids)| galaxy_ids.len() > 1)
            .sorted_by_key(|(_, galaxy_ids)| galaxy_ids.len());
        for (&position, galaxy_ids) in positions_with_multiple_possible_galaxies {
            for &galaxy_id in galaxy_ids {
                let mut solver = self.clone();
                solver
                    .possible_galaxy_ids
                    .get_mut(&position)
                    .unwrap()
                    .retain(|&id| id == galaxy_id);
                if let Err(Contradiction) = solver.solve() {
                    self.possible_galaxy_ids
                        .get_mut(&position)
                        .unwrap()
                        .remove(&galaxy_id);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    mod solve {
        use crate::model::board::Board;
        use crate::model::objective::Objective;
        use crate::model::solver::Solver;
        use indoc::indoc;

        fn test_should_solve_example(objective: &str, expected_solution_str: &str) {
            let objective = Objective::from_string(objective);
            let expected_solution = Board::from_string(expected_solution_str);
            let actual_solution = Solver::new(
                expected_solution.get_width(),
                expected_solution.get_height(),
                &objective,
            )
            .solve()
            .map(|it| Board::from_iter(it.borders))
            .unwrap();
            assert_eq!(
                actual_solution, expected_solution,
                "expected \n{}\nbut was \n{}",
                &actual_solution, &expected_solution
            );
        }

        #[test]
        fn should_successfully_solve_examples() {
            test_should_solve_example(
                indoc! {"
                    ┌───┬───┬───┬───┐
                    │             ● │
                    ├   ·   · ● ·   ┤
                    │               │
                    ├ ● ·   ·   ·   ┤
                    │     ●         │
                    ├   ·   ·   ●   ┤
                    │               │
                    └───┴───┴───┴───┘"
                },
                indoc! {"
                    ┌─┬───┬─┐
                    │ ├─┐ └─┤
                    │ │ ├───┤
                    │ │ │   │
                    └─┴─┴───┘"
                },
            );
            test_should_solve_example(
                indoc! {"
                    ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
                    │         ●           ●   ●           ● │
                    ├   ·   ·   ·   ·   ·   ·   ·   ●   ·   ┤
                    │ ●           ●       ●                 │
                    ├   ·   ·   ·   ·   ·   ·   ·   ·   ·   ┤
                    │ ●                                     │
                    ├   ·   ●   ·   · ● ·   ·   ·   ·   ·   ┤
                    │                                 ●     │
                    ├   ·   ·   ·   ·   ·   ·   ·   ·   ·   ┤
                    │               ●                       │
                    ├   ·   ·   ·   ·   ·   ·   ·   ·   ·   ┤
                    │                   ●               ●   │
                    ├   ●   ·   ·   ·   ·   ·   ·   ·   ·   ┤
                    │         ●           ●                 │
                    ├   ·   ·   ·   ·   ·   ·   ·   ·   ·   ┤
                    │                       ●               │
                    ├   ·   ·   ·   ·   ·   ·   ·   · ● ·   ┤
                    │       ●                 ●             │
                    ├   ·   ·   ·   ·   ·   ·   ·   ·   ·   ┤
                    │ ●       ●       ●       ●       ●   ● │
                    └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘"
                },
                indoc! {"
                    ┌───┬─┬───┬─┬─┬───┬─┐
                    ├─┐ └─┼─┐ └─┴─┤   ├─┤
                    ├─┤   └─┼───┐ └─┬─┘ │
                    ├─┘   ┌─┘ ┌─┴─┬─┘   │
                    ├─┐   ├───┤   │   ┌─┤
                    │ └─┐ └─┬─┘ ┌─┤ ┌─┘ │
                    │   ├─┬─┘ ┌─┤ └─┤ ┌─┤
                    ├─┐ ├─┤   ├─┘ ┌─┴─┘ │
                    │ └─┘ └─┬─┘ ┌─┤     │
                    ├─┐ ┌─┐ ├─┐ ├─┤ ┌─┬─┤
                    └─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘"
                },
            );
        }
    }

    mod mirror_borders {
        use crate::model::objective::{GalaxyCenter, Objective};
        use crate::model::position::Position;
        use crate::model::solver::Solver;
        use std::collections::HashSet;

        #[test]
        fn should_successfully_mirror_borders() {
            let mut solver = Solver::new(
                3,
                4,
                &Objective {
                    walls: HashSet::default(),
                    centers: HashSet::from_iter(vec![
                        GalaxyCenter::from(Position::new(2, 2)),
                        GalaxyCenter::from(Position::new(4, 2)),
                    ]),
                },
            );

            solver.solve();

            // assert_eq!(solver.borders[&Border::down(Position::new(1, 1))], true)
        }
    }
}
