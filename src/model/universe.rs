use crate::model::galaxy::Galaxy;
use crate::model::position::Position;
use petgraph::data::Build;
use petgraph::graphmap::UnGraphMap;
use petgraph::visit::{Dfs, Walker};
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{random, Rng, SeedableRng};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub struct Universe {
    width: usize,
    height: usize,
    graph: UnGraphMap<Position, ()>,
}

impl Universe {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn generate(width: usize, height: usize) -> Universe {
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
                .max_by_key(|universe| universe.get_score())
                .unwrap_or(universe);
        }
        assert!(universe.is_valid());
        universe
    }
}

impl Universe {
    pub fn new(width: usize, height: usize) -> Universe {
        let mut graph: UnGraphMap<Position, ()> = UnGraphMap::new();
        for row in 0..height {
            for column in 0..width {
                let position = Position::from((row, column));
                graph.add_node(position);
            }
        }
        Universe {
            width,
            height,
            graph,
        }
    }

    fn generate_step(&mut self, rng: &mut impl Rng) -> bool {
        // First we pick a random position in the universe
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
                }
                else {
                    p3_candidates.get(rng.gen_range(0..p3_candidates.len())).cloned()
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
            }
            else {
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

    /// Returns a list of galaxies in this universe, in no particular order
    pub fn get_galaxies(&self) -> Vec<Galaxy> {
        let mut galaxies: Vec<Galaxy> = Vec::new();
        let mut remaining_positions: BTreeSet<Position> = self.graph.nodes().collect();
        while let Some(position) = remaining_positions.first() {
            let galaxy = self.get_galaxy(position);
            for p in galaxy.get_positions() {
                remaining_positions.remove(p);
            }
            galaxies.push(galaxy);
        }
        galaxies
    }

    /// Make p have no neighbours
    pub fn remove_all_neighbours(&mut self, p: &Position) {
        for adjacent_position in self.adjacent_positions(p) {
            self.graph.remove_edge(*p, adjacent_position);
        }
    }

    /// Metric of how "cool" is the universe is. Lower is better.
    pub fn get_score(&self) -> i64 {
        let mut score: i64 = 0;

        // Penalize long, straight, horizontal borders
        for row in 1..self.height as i32 {
            let mut current_length: i64 = 0;
            for col in 0..self.width as i32 {
                let up = Position::new(row - 1, col);
                let down = Position::new(row, col);
                if self.are_neighbours(&up, &down) {
                    score -= current_length.pow(2);
                    current_length = 0;
                } else {
                    current_length += 1;
                }
            }
            score -= current_length.pow(2);
        }

        // Penalize long, straight, vertical borders
        for col in 1..self.width as i32 {
            let mut current_length: i64 = 0;
            for row in 0..self.height as i32 {
                let left = Position::new(row, col - 1);
                let right = Position::new(row, col);
                if self.are_neighbours(&left, &right) {
                    score -= current_length.pow(2);
                    current_length = 0;
                } else {
                    current_length += 1;
                }
            }
            score -= current_length.pow(2);
        }

        let galaxies = self.get_galaxies();

        // Penalize big rectangles
        for galaxy in &galaxies {
            for rect in galaxy.rectangles() {
                let area = rect.area() as i64;
                score -= area.pow(2);
            }
        }

        // Penalize many galaxies
        score -= 3 * galaxies.len() as i64;

        // Reward galaxies with high swirl
        score += galaxies.iter().map(|g| g.get_swirl().abs().powi(2)).sum::<f64>() as i64;

        // Reward galaxies with high curl
        score += galaxies.iter().map(|g| g.get_swirl().abs().powi(2)).sum::<f64>() as i64;

        score
    }

    pub fn add_galaxy(&mut self, galaxy: &Galaxy) {
        for p1 in galaxy.get_positions() {
            for p2 in &self.adjacent_positions(p1) {
                if galaxy.contains_position(p2) {
                    self.graph.add_edge(*p1, *p2, ());
                }
            }
        }
    }

    /**
     * Joins p2 into the galaxy of p1, removing any previous edges from p2,
     * and adding edges to all neighbouring positions in the galaxy of p1.
     * Returns whether p1 and p2 were successfully made neighbours.
     */
    pub fn make_neighbours(&mut self, p1: &Position, p2: &Position) {
        let g1 = self.get_galaxy(p1);
        for p2_adjacent in self.adjacent_positions(p2) {
            if g1.contains_position(&p2_adjacent) {
                self.graph.add_edge(*p2, p2_adjacent, ());
            } else {
                self.graph.remove_edge(*p2, p2_adjacent);
            }
        }
    }

    pub fn random_position(&self, rng: &mut impl Rng) -> Position {
        Position::random(self.width, self.height, rng)
    }

    pub fn adjacent_positions(&self, p: &Position) -> Vec<Position> {
        vec![p.left(), p.up(), p.right(), p.down()]
            .iter()
            .copied()
            .filter(|&adjacent_position| self.graph.contains_node(adjacent_position))
            .collect()
    }

    pub fn adjacent_non_neighbours(&self, p: &Position) -> Vec<Position> {
        self.adjacent_positions(p)
            .iter()
            .copied()
            .filter(|adjacent_position| !self.are_neighbours(p, adjacent_position))
            .collect()
    }

    pub fn are_neighbours(&self, p1: &Position, p2: &Position) -> bool {
        self.graph.contains_edge(*p1, *p2)
    }

    pub fn get_galaxy(&self, p: &Position) -> Galaxy {
        let search = Dfs::new(&self.graph, *p);
        Galaxy::from(search.iter(&self.graph))
    }

    pub fn is_valid(&self) -> bool {
        self.get_galaxies().iter().all(|galaxy| galaxy.is_valid())
    }

    pub fn is_outside(&self, p: &Position) -> bool {
        !self.is_inside(p)
    }

    pub fn is_inside(&self, p: &Position) -> bool {
        self.graph.contains_node(*p)
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn get_positions(&self) -> impl Iterator<Item = Position> + '_ {
        (0..self.height)
            .flat_map(move |row| (0..self.width).map(move |col| (row, col)))
            .map(|t| Position::from(t))
    }
}

impl Display for Universe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in 0..=self.height {
            for column in 0..=self.width {
                let bottom_right = Position::from((row, column));
                let bottom_left = bottom_right.left();
                let top_left = bottom_left.up();
                let top_right = bottom_right.up();

                let bar_top = row != 0 && !self.are_neighbours(&top_left, &top_right);
                let bar_right =
                    column != self.width && !self.are_neighbours(&top_right, &bottom_right);
                let bar_bottom =
                    row != self.height && !self.are_neighbours(&bottom_left, &bottom_right);
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
            if row != self.height {
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
        for g in galaxies {
            for p1 in g.get_positions() {
                for p2 in &p1.adjacent() {
                    if g.contains_position(p2) {
                        universe.graph.add_edge(*p1, *p2, ());
                    }
                }
            }
        }

        universe
    }
}
