use crate::model::galaxy::Galaxy;
use crate::model::position::Position;
use itertools::Itertools;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{random, Rng, SeedableRng};
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};
use ordered_float::OrderedFloat;

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
        let p2_option = self.adjacent_non_neighbours(&p1).choose(rng).cloned();
        if p2_option.is_none() {
            // There are no adjacent non neighbours, so we abort
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
                for p3 in self.adjacent_non_neighbours(&p2) {
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

    fn get_ids(&self) -> impl Iterator<Item = &usize> {
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

    /// Metric of how "cool" is the universe is. Lower is better.
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

        let galaxies = self.get_galaxies();

        // Penalize big rectangles
        for galaxy in &galaxies {
            for rect in galaxy.rectangles() {
                let area = rect.area() as f64;
                score -= area.powf(2.);
            }
        }

        // Penalize many galaxies
        score -= 3. * galaxies.len() as f64;

        // Reward galaxies with high swirl
        score += 10. * galaxies
            .iter()
            .map(|g| g.get_swirl().abs().powf(2.5))
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

    pub fn adjacent_non_neighbours(&self, p: &Position) -> Vec<Position> {
        self.adjacent_positions(p)
            .iter()
            .copied()
            .filter(|adjacent_position| !self.are_neighbours(p, adjacent_position))
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
