use crate::model::galaxy::Galaxy;
use crate::model::position::Position;
use crate::model::vec2::Vec2;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{random, Rng, SeedableRng};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct Universe {
    grid: Vec<Vec<usize>>,
}

impl Universe {
    fn new(width: usize, height: usize) -> Self {
        let mut grid = vec![vec![0; width]; height];
        for row in 0..height {
            for col in 0..width {
                grid[row][col] = row * width + col;
            }
        }
        Universe { grid }
    }

    pub fn generate(width: usize, height: usize) -> Self {
        let mut universe = Universe::new(width, height);
        let iterations = width * height * 10;
        let branches = 5;
        let seed: u64 = random();
        println!("Seed: {}", seed);
        let mut rng = StdRng::seed_from_u64(seed);
        for _iteration in 0..iterations {
            let mut next_universes = Vec::with_capacity(branches);
            for _branch in 0..branches {
                let next_universe = universe.clone();
                let success = universe.generate_step(&mut rng);
                if success {
                    next_universes.push(next_universe);
                }
            }

            universe = next_universes
                .into_iter()
                .max_by_key(|universe| OrderedFloat(universe.get_score()))
                .unwrap_or(universe);
        }
        assert!(universe.is_valid());
        universe
    }

    fn generate_step(&mut self, rng: &mut impl Rng) -> bool {
        // First, we pick a random position in the universe
        let p1 = self.random_position(rng);

        // Then we pick one of the adjacent positions that is not already a neighbour
        let p2_option = self.get_adjacent_non_neighbours(&p1).choose(rng).cloned();
        if p2_option.is_none() {
            // There are no adjacent non-neighbours, so we abort
            return false;
        }

        let g1 = self.get_galaxy(&p1);
        let p2 = p2_option.unwrap();

        let g1_with_p2 = g1.with_position(&p2);
        if g1_with_p2.is_symmetric() {
            // If g1_with_p2 is symmetric, we do not need to consider p3 and g3,
            // but we need to properly remove p2 from g2 before adding it to g1.
            let g2 = self.get_galaxy(&p2);
            self.remove_positions_from_galaxy(&g2, &[p2]);
            self.make_neighbours(&p1, &p2);
            true
        } else {
            // If g1_with_p2 is asymmetric, we need to add p3 to it
            let p3_option = {
                let mut p3_candidates = Vec::new();
                {
                    let p3 = g1.mirror_position(&p2);
                    if self.is_inside(&p3) {
                        p3_candidates.push(p3);
                    }
                }
                for p3 in self.get_adjacent_non_neighbours(&p2) {
                    if g1_with_p2.with_position(&p3).is_symmetric() {
                        p3_candidates.push(p3);
                    }
                }
                if p3_candidates.is_empty() {
                    None
                } else {
                    p3_candidates
                        .get(rng.gen_range(0..p3_candidates.len()))
                        .cloned()
                }
            };

            if let Some(p3) = p3_option {
                let g2 = self.get_galaxy(&p2);
                let g3 = self.get_galaxy(&p3);

                if g2 == g3 {
                    // If g2 and g3 is the same galaxy, we need to consider everything together while removing p2 and p3 from it
                    self.remove_positions_from_galaxy(&g2, &[p2, p3]);
                } else {
                    // If g2 and g3 are different galaxies, we can treat them separately
                    self.remove_positions_from_galaxy(&g2, &[p2]);
                    self.remove_positions_from_galaxy(&g3, &[p3]);
                }
                self.make_neighbours(&p1, &p2);
                self.make_neighbours(&p1, &p3);
                true
            } else {
                // No candidates for p3 found to make g1 with p2 symmetric
                false
            }
        }
    }

    pub(crate) fn generate_weighted(width: usize, height: usize) -> Self {
        let mut universe = Universe::new(width, height);
        let mut rng = {
            let seed: u64 = random();
            println!("Seed: {}", seed);
            StdRng::seed_from_u64(seed)
        };

        fn compute_neighbour_weight(
            neighbour: &Position,
            position: &Position,
            winding_number: f64,
            galaxy_id: usize,
            galaxy_center: &Vec2,
            universe: &Universe,
        ) -> f64 {
            let mut weight = 0.0;
            let neighbours_adjacent_to_candidate: Vec<Position> = neighbour
                .adjacent()
                .into_iter()
                .filter(|p| p != position)
                .filter(|p| universe.is_inside(p))
                .filter(|p| universe[p] == galaxy_id)
                .collect();

            weight += match neighbours_adjacent_to_candidate.len() {
                0 => 100.0,
                1 => -100.0,
                2 => -200.0,
                3 => -400.0,
                _ => unreachable!(),
            };

            let candidate_winding_number = {
                let position_v = Vec2::from(position) - galaxy_center;
                let candidate_v = Vec2::from(neighbour) - galaxy_center;
                let candidate_winding_number = winding_number + position_v.angle_to(&candidate_v);
                candidate_winding_number.abs()
            };

            weight += 100.0 * candidate_winding_number;

            weight
        }

        fn compute_position_weight(
            position: &Position,
            winding_number: f64,
            galaxy: &Galaxy,
            galaxy_center: &Vec2,
            universe: &Universe,
        ) -> f64 {
            let mut weight = 0.0;
            let id = universe[position];
            let neighbours = universe.get_neighbours(position);
            let non_neighbours = universe.get_adjacent_non_neighbours(position);

            weight += match neighbours.len() {
                0 => 100.0,
                1 => 50.0,
                2 => 1.0,
                3 => 1.0,
                4 => -100.0,
                _ => unreachable!(),
            };

            weight += 100.0 / galaxy.size() as f64;

            weight += non_neighbours
                .iter()
                .map(|candidate| {
                    compute_neighbour_weight(
                        candidate,
                        position,
                        winding_number,
                        id,
                        &galaxy_center,
                        universe,
                    )
                })
                .map(OrderedFloat::from)
                .max()
                .map(OrderedFloat::into_inner)
                .unwrap_or(0.0);

            weight
        }

        let recompute_galaxy_weights =
            |weights: &mut Vec<f64>, universe: &Universe, galaxy: &Galaxy, galaxy_center: &Vec2| {
                let winding_numbers: HashMap<Position, f64> = galaxy
                    .get_winding_spanning_tree()
                    .into_iter()
                    .map(|(position, (winding_number, _))| (position, winding_number))
                    .collect();
                for position in galaxy.get_positions() {
                    let index = position.to_index(width);
                    let winding_number = winding_numbers[position];
                    weights[index] = compute_position_weight(
                        position,
                        winding_number,
                        galaxy,
                        galaxy_center,
                        universe,
                    );
                }
            };

        let recompute_galaxy_id_weights =
            |weights: &mut Vec<f64>, universe: &Universe, position: &Position| {
                let galaxy = universe.get_galaxy(position);
                let galaxy_center = Vec2::from_center(&galaxy.center());
                recompute_galaxy_weights(weights, &universe, &galaxy, &galaxy_center);
            };

        let mut weights: Vec<f64> = (0..width * height)
            .map(|index| {
                let row = index / width;
                let column = index % width;
                let position = Position::from((row, column));
                let galaxy = Galaxy::from(position);
                let galaxy_center = Vec2::from_center(&galaxy.center());
                let winding_number = 0.0;
                compute_position_weight(
                    &position,
                    winding_number,
                    &galaxy,
                    &galaxy_center,
                    &universe,
                )
            })
            .collect();

        fn get_random_weighted_position(
            weights: &Vec<f64>,
            width: usize,
            rng: &mut StdRng,
        ) -> Option<Position> {
            let weight_sum = weights.iter().sum::<f64>();
            let random_value = rng.gen::<f64>() * weight_sum;
            let mut cumulative_weight = 0.0;
            for (index, &weight) in weights.iter().enumerate() {
                cumulative_weight += weight;
                if cumulative_weight < random_value {
                    continue;
                }
                let row = index / width;
                let column = index % width;
                return Some(Position::from((row, column)));
            }
            None
        }

        let iterations = width * height * 10;
        let mut best_universe = universe.clone();
        for _iteration in 0..iterations {
            let Some(position) = get_random_weighted_position(&weights, width, &mut rng) else {
                panic!("Could not get random position");
            };
            let galaxy = universe.get_galaxy(&position);
            let galaxy_id = universe[&position];
            let galaxy_center = galaxy.center();
            let galaxy_center_v = Vec2::from_center(&galaxy_center);
            let winding_tree = galaxy.get_winding_spanning_tree();
            let winding_number = winding_tree[&position].0;
            let maybe_neighbour = universe
                .get_adjacent_non_neighbours(&position)
                .into_iter()
                .max_by_key(|candidate| {
                    OrderedFloat(compute_neighbour_weight(
                        candidate,
                        &position,
                        winding_number,
                        galaxy_id,
                        &galaxy_center_v,
                        &universe,
                    ))
                });
            if let Some(neighbour) = maybe_neighbour {
                assert_ne!(position, neighbour);
                let neighbour_id = universe[&neighbour];
                let neighbour_galaxy = universe.get_galaxy(&neighbour);
                let galaxy_with_neighbour = galaxy.with_position(&neighbour);
                if galaxy_with_neighbour.is_symmetric() {
                    universe.remove_positions_from_galaxy(&neighbour_galaxy, &[neighbour]);
                    universe[&neighbour] = galaxy_id;
                    recompute_galaxy_id_weights(&mut weights, &universe, &position);
                    recompute_galaxy_id_weights(&mut weights, &universe, &neighbour);
                } else {
                    // If the galaxy is asymmetric after adding the new position,
                    // we need to add another position to it to make it symmetric.
                    // Most often, we need to add the mirror position, but sometimes
                    // there are multiple candidates. In the following example,
                    // there are three candidates to make the galaxy symmetric:
                    //
                    // (AA = existing, BB = new, CC = candidate to make everything symmetric)
                    // ┌────┬────┬────┐
                    // │ CC │ BB │ CC │
                    // ├────┴────┼────┘
                    // │ AA   AA │
                    // ├────┬────┘
                    // │ CC │
                    // └────┘
                    let candidate = {
                        let mut candidates = Vec::new();
                        {
                            let mirror = galaxy_center.mirror_position(&neighbour);
                            if universe.is_inside(&mirror) {
                                candidates.push(mirror);
                            }
                        }

                        for adjacent in universe.get_adjacent_non_neighbours(&neighbour) {
                            if galaxy_with_neighbour
                                .with_position(&adjacent)
                                .is_symmetric()
                            {
                                candidates.push(adjacent);
                            }
                        }

                        if candidates.is_empty() {
                            continue;
                        }

                        candidates[rng.gen_range(0..candidates.len())]
                    };

                    let candidate_id = universe[&candidate];
                    if neighbour_id == candidate_id {
                        universe.remove_positions_from_galaxy(
                            &neighbour_galaxy,
                            &[neighbour, candidate],
                        );
                    } else {
                        let candidate_galaxy = universe.get_galaxy(&candidate);
                        universe.remove_positions_from_galaxy(&neighbour_galaxy, &[neighbour]);
                        universe.remove_positions_from_galaxy(&candidate_galaxy, &[candidate]);
                    }
                    universe[&neighbour] = galaxy_id;
                    universe[&candidate] = galaxy_id;
                    recompute_galaxy_id_weights(&mut weights, &universe, &position);
                    recompute_galaxy_id_weights(&mut weights, &universe, &neighbour);
                    recompute_galaxy_id_weights(&mut weights, &universe, &candidate);
                }
            } else {
                // Do nothing for now, in the future maybe remove position or something...
                continue;
            }
            if universe.get_score() > best_universe.get_score() {
                best_universe = universe.clone();
            } else {
                universe = best_universe.clone();
            }
        }

        best_universe
    }
    // fn generate_weighted(width: usize, height: usize) -> Self {
    //     let mut universe = Universe::new(width, height);
    //     let mut rng = {
    //         let seed: u64 = random();
    //         println!("Seed: {}", seed);
    //         StdRng::seed_from_u64(seed)
    //     };
    //
    //     fn compute_neighbour_weight(
    //         universe: &Universe,
    //         position: &Position,
    //         center: &Vec2,
    //         id: usize,
    //         candidate: &Position,
    //     ) -> f64 {
    //         let mut weight = 0.0;
    //         let neighbours_adjacent_to_candidate: Vec<Position> = candidate
    //             .adjacent()
    //             .into_iter()
    //             .filter(|p| p != position)
    //             .filter(|p| universe.is_inside(p))
    //             .filter(|p| universe[p] == id)
    //             .collect();
    //
    //         weight += 10.0
    //             * match neighbours_adjacent_to_candidate.len() {
    //                 0 => 1.0,
    //                 1 => -1.0,
    //                 2 => -2.0,
    //                 3 => -4.0,
    //                 _ => 0.0,
    //             };
    //
    //         let winding_number = {
    //             let position_v = Vec2::from(position) - center;
    //             let candidate_v = Vec2::from(candidate) - center;
    //             let mut winding_number = position_v.angle_to(&candidate_v);
    //             let current = position;
    //             while let Some(parent) = parents.get(current) {
    //                 let current_v = Vec2::from(current) - center;
    //                 let parent_v = Vec2::from(parent) - center;
    //                 let angle = parent_v.angle_to(&current_v);
    //                 winding_number += angle;
    //             }
    //             winding_number.abs()
    //         };
    //
    //         weight += 10.0 * winding_number;
    //
    //         weight
    //     }
    //
    //     fn compute_weight(universe: &Universe, position: &Position) -> f64 {
    //         let mut weight = 0.0;
    //         let id = universe[position];
    //         let neighbours = universe.get_neighbours(position);
    //         let non_neighbours = universe.get_adjacent_non_neighbours(position);
    //         let galaxy = universe.get_galaxy(position);
    //         let center = Vec2::from(&galaxy.center()) / 2.0;
    //
    //         weight += 10.0
    //             * match neighbours.len() {
    //                 0 => 1.0,
    //                 1 => 0.5,
    //                 2 => 0.25,
    //                 3 => 0.1,
    //                 4 => 0.1,
    //                 _ => 0.0,
    //             };
    //
    //         weight += 10.0 / galaxy.size() as f64;
    //
    //         weight += non_neighbours
    //             .iter()
    //             .map(|candidate| {
    //                 compute_neighbour_weight(universe, parents, position, &center, id, candidate)
    //             })
    //             .map(OrderedFloat::from)
    //             .max()
    //             .map(OrderedFloat::into_inner)
    //             .unwrap_or(0.0);
    //
    //         weight
    //     }
    //
    //     fn make_singleton(universe: &mut Universe, position: &Position) {
    //         let mut galaxy = universe.get_galaxy(position);
    //         let center = galaxy.center();
    //         let children: Vec<Position> = parents
    //             .iter()
    //             .filter(|&(child, parent)| parent == position)
    //             .map(|(child, _)| child)
    //             .copied()
    //             .collect();
    //         galaxy.remove_position(&position);
    //         removed_positions.push(*position);
    //
    //         if !galaxy.is_symmetric() {
    //             let mirror = center.mirror_position(position);
    //             galaxy.remove_position(&mirror);
    //             removed_positions.push(mirror);
    //         }
    //         if !galaxy.is_empty_or_valid() {
    //             for position in galaxy.get_positions().copied() {
    //                 removed_positions.push(position);
    //             }
    //         }
    //     }
    //
    //     let weights: Vec<f64> = (0..width * height)
    //         .map(|index| {
    //             let row = index / width;
    //             let column = index % width;
    //             compute_weight(&universe, &parents, &Position::from((row, column)))
    //         })
    //         .collect();
    //
    //     fn get_random_weighted_position(
    //         weights: &Vec<f64>,
    //         width: usize,
    //         rng: &mut StdRng,
    //     ) -> Option<Position> {
    //         let weight_sum = weights.iter().sum::<f64>();
    //         let random_value = rng.gen::<f64>() * weight_sum;
    //         let mut cumulative_weight = 0.0;
    //         for (index, &weight) in weights.iter().enumerate() {
    //             cumulative_weight += weight;
    //             if cumulative_weight < random_value {
    //                 continue;
    //             }
    //             let row = index / width;
    //             let column = index % width;
    //             return Some(Position::from((row, column)));
    //         }
    //         None
    //     }
    //
    //     let iterations = width * height * 10;
    //     for _iteration in 0..iterations {
    //         let Some(position) = get_random_weighted_position(&weights, width, &mut rng) else {
    //             continue;
    //         };
    //         let galaxy = universe.get_galaxy(&position);
    //         let galaxy_center = galaxy.center();
    //         let galaxy_center_v = Vec2::from_center(&galaxy_center);
    //         let id = universe[&position];
    //         let maybe_neighbour = &universe
    //             .get_adjacent_non_neighbours(&position)
    //             .into_iter()
    //             .max_by_key(|&adj| {
    //                 OrderedFloat(compute_neighbour_weight(
    //                     &universe,
    //                     &position,
    //                     &galaxy_center_v,
    //                     id,
    //                     &adj,
    //                 ))
    //             });
    //         if let Some(neighbour) = maybe_neighbour {
    //             let neighbour_galaxy = universe.get_galaxy(neighbour);
    //             let galaxy_with_neighbour = galaxy.with_position(neighbour);
    //             if galaxy_with_neighbour.is_symmetric() {
    //                 let removed =
    //                     universe.remove_positions_from_galaxy(&neighbour_galaxy, &[neighbour]);
    //             }
    //             let neighbour_id = universe[neighbour];
    //         } else {
    //             // Do nothing
    //         }
    //     }
    //
    //     universe
    // }

    /// Removes the given positions from the galaxy, while keeping the universe valid.
    /// After calling this method, all positions in [positions_to_remove] are singles.
    fn remove_positions_from_galaxy(&mut self, galaxy: &Galaxy, positions_to_remove: &[Position]) {
        let mut g = galaxy.clone();
        for p in positions_to_remove {
            assert!(galaxy.contains_position(&p));
            self.remove_all_neighbours(p);
            g.remove_position(p);
            if !g.is_symmetric() {
                // If g is asymmetric, we can solve that by removing the mirror of p as well
                let p2 = galaxy.mirror_position(&p);
                self.remove_all_neighbours(&p2);
                g.remove_position(&p2);
            }
            if !g.is_empty_or_valid() {
                // If g is invalid, it's because removing p (and maybe p2) disconnected it or removed its center.
                // In both cases, we solve this by breaking up g completely into singles.
                for remaining_positions in g.get_positions() {
                    self.remove_all_neighbours(remaining_positions);
                }
                return;
            }
        }
    }

    fn get_max_id(&self) -> usize {
        *self.grid.iter().flatten().max().unwrap_or(&0)
    }

    pub fn get_ids(&self) -> impl Iterator<Item = &usize> {
        self.grid.iter().flatten()
    }

    fn get_entries(&self) -> impl Iterator<Item = (Position, usize)> + '_ {
        self.grid.iter().enumerate().flat_map(|(row_index, row)| {
            row.iter()
                .enumerate()
                .map(move |(column_index, id)| (Position::from((row_index, column_index)), *id))
        })
    }

    fn get_width(&self) -> usize {
        self.grid.first().map(|row| row.len()).unwrap_or(0)
    }

    fn get_height(&self) -> usize {
        self.grid.len()
    }

    fn get_next_available_id(&self) -> usize {
        let size = self.get_width() * self.get_height();
        let mut id_in_use = vec![false; size];
        for &id in self.get_ids() {
            id_in_use[id] = true;
        }
        for (id, in_use) in id_in_use.into_iter().enumerate() {
            if !in_use {
                return id;
            }
        }
        size
    }

    /// Returns a list of galaxies in this universe, in no particular order,
    /// by grouping together all cells that have the same id
    pub fn get_galaxies(&self) -> Vec<Galaxy> {
        self.get_entries()
            .map(|(a, b)| (b, a))
            .into_group_map()
            .into_values()
            .map(|positions| Galaxy::from(positions))
            .collect()
    }

    /// Make p have no neighbours
    pub fn remove_all_neighbours(&mut self, p: &Position) {
        self[p] = self.get_next_available_id();
    }

    /// Metric of how "cool" the universe is, higher is better
    pub fn get_score(&self) -> f64 {
        let mut score: f64 = 0.;

        // Penalize long, straight, horizontal borders
        let straight_line_penalty = 3.5;
        for row in 1..self.get_height() as i32 {
            let mut current_length: f64 = 0.;
            for col in 0..self.get_width() as i32 {
                let up = Position::new(row - 1, col);
                let down = Position::new(row, col);
                if self.are_neighbours(&up, &down) {
                    score -= current_length.powf(straight_line_penalty);
                    current_length = 0.;
                } else {
                    current_length += 1.;
                }
            }
            score -= current_length.powf(straight_line_penalty);
        }

        // Penalize long, straight, vertical borders
        for col in 1..self.get_width() as i32 {
            let mut current_length: f64 = 0.;
            for row in 0..self.get_height() as i32 {
                let left = Position::new(row, col - 1);
                let right = Position::new(row, col);
                if self.are_neighbours(&left, &right) {
                    score -= current_length.powf(straight_line_penalty);
                    current_length = 0.;
                } else {
                    current_length += 1.;
                }
            }
            score -= current_length.powf(straight_line_penalty);
        }

        score += self
            .get_galaxies()
            .iter()
            .map(|g| g.get_score())
            .sum::<f64>();

        score
    }

    /// Joins p2 into the galaxy of p1, removing it from its previous galaxy.
    /// Does not preserve galaxy validness.
    pub fn make_neighbours(&mut self, p1: &Position, p2: &Position) {
        self[p2] = self[p1];
    }

    pub fn random_position(&self, rng: &mut impl Rng) -> Position {
        Position::random(self.get_width(), self.get_height(), rng)
    }

    pub fn adjacent_positions(&self, p: &Position) -> Vec<Position> {
        let mut adjacent = Vec::with_capacity(4);
        if p.row > 0 {
            adjacent.push(p.up())
        }
        if p.row < self.get_height() as i32 - 1 {
            adjacent.push(p.down())
        }
        if p.column > 0 {
            adjacent.push(p.left())
        }
        if p.column < self.get_width() as i32 - 1 {
            adjacent.push(p.right())
        }
        adjacent
    }

    pub fn get_adjacent_non_neighbours(&self, p: &Position) -> Vec<Position> {
        self.adjacent_positions(p)
            .iter()
            .copied()
            .filter(|adjacent_position| !self.are_neighbours(p, adjacent_position))
            .collect()
    }

    pub fn get_neighbours(&self, position: &Position) -> Vec<Position> {
        position
            .adjacent()
            .into_iter()
            .filter(|adjacent| self.are_neighbours(position, adjacent))
            .collect()
    }

    pub fn are_neighbours(&self, p1: &Position, p2: &Position) -> bool {
        self.is_inside(p1) && self.is_inside(p2) && self[p1] == self[p2]
    }

    pub fn get_galaxy(&self, p: &Position) -> Galaxy {
        let p_id = &self[p];
        self.get_entries()
            .filter(|(p, id)| id == p_id)
            .map(|(p, _)| p)
            .collect()
    }

    pub fn is_valid(&self) -> bool {
        self.get_galaxies().iter().all(|galaxy| galaxy.is_valid())
    }

    pub fn is_outside(&self, p: &Position) -> bool {
        !self.is_inside(p)
    }

    /// Return true iff the position is within the bounds of the universe
    pub fn is_inside(&self, p: &Position) -> bool {
        p.row >= 0
            && p.row < self.get_height() as i32
            && p.column >= 0
            && p.column < self.get_width() as i32
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn get_positions(&self) -> impl Iterator<Item = Position> + '_ {
        (0..self.get_height())
            .flat_map(move |row| (0..self.get_width()).map(move |col| (row, col)))
            .map(|t| Position::from(t))
    }
}

impl Display for Universe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in 0..=self.get_height() {
            let mut line = String::new();
            for column in 0..=self.get_width() {
                let bottom_right = Position::from((row, column));
                let bottom_left = bottom_right.left();
                let top_left = bottom_left.up();
                let top_right = bottom_right.up();

                let bar_top = row != 0 && !self.are_neighbours(&top_left, &top_right);
                let bar_right =
                    column != self.get_width() && !self.are_neighbours(&top_right, &bottom_right);
                let bar_bottom =
                    row != self.get_height() && !self.are_neighbours(&bottom_left, &bottom_right);
                let bar_left = column != 0 && !self.are_neighbours(&top_left, &bottom_left);
                line.push_str(match (bar_top, bar_right, bar_bottom, bar_left) {
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
                })
            }
            f.write_str(line.trim_end())?;
            if row != self.get_height() {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

impl From<&[Galaxy]> for Universe {
    fn from(galaxies: &[Galaxy]) -> Self {
        let width = galaxies
            .iter()
            .flat_map(|g| g.get_positions())
            .map(|p| p.column + 1)
            .max()
            .unwrap_or(0) as usize;
        let height = galaxies
            .iter()
            .flat_map(|g| g.get_positions())
            .map(|p| p.row + 1)
            .max()
            .unwrap_or(0) as usize;
        let mut universe = Universe::new(width, height);
        for (id, g) in galaxies.iter().enumerate() {
            for p in g.get_positions() {
                universe[p] = id
            }
        }

        universe
    }
}

impl Index<&Position> for Universe {
    type Output = usize;

    fn index(&self, pos: &Position) -> &Self::Output {
        &self.grid[pos.row as usize][pos.column as usize]
    }
}

impl IndexMut<&Position> for Universe {
    fn index_mut(&mut self, pos: &Position) -> &mut Self::Output {
        &mut self.grid[pos.row as usize][pos.column as usize]
    }
}
