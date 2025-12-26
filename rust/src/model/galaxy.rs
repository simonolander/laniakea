use crate::model::border::Border;
use crate::model::position::{CenterPlacement, Position};
use crate::model::rectangle::Rectangle;
use crate::model::tree::Tree;
use crate::model::vec2::Vec2;
use itertools::Itertools;
use ordered_float::{Float, OrderedFloat};
use std::cmp::{max, min};
use std::collections::hash_map::Entry;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet, LinkedList, VecDeque};
use std::convert::identity;
use std::fmt::{Display, Formatter};
use std::ops::Sub;
use std::slice::Iter;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Galaxy {
    positions: HashSet<Position>,
}

/// A galaxy is a set of positions. A valid galaxy needs to satisfy the following conditions:
/// - It must not be empty
/// - It must be connected
/// - It must contain its center
/// - It must be rotationally symmetric, order 2 (or 4)
impl Galaxy {
    /// Create a galaxy from a string, where non-space characters
    /// are interpreted as belonging to the galaxy.
    /// The resulting galaxy is not necessarily valid.
    fn from_string(string: &str) -> Self {
        string
            .lines()
            .enumerate()
            .flat_map(|(row, line)| {
                line.chars().enumerate().filter_map(move |(column, c)| {
                    if c != ' ' {
                        Some(Position::from((row, column)))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

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
        let rect = self.get_bounding_rectangle();
        let center_half_row = rect.min_row + rect.max_row;
        let center_half_column = rect.min_column + rect.max_column;
        Position::new(center_half_row, center_half_column)
    }

    /// Returns the smallest rectangle that contains the galaxy.
    pub fn get_bounding_rectangle(&self) -> Rectangle {
        Rectangle::bounding_rectangle(self.positions.iter().copied())
    }

    /// Mirrors the position horizontally and vertically with respect to the center of the galaxy
    pub fn mirror_position(&self, p: &Position) -> Position {
        self.center().mirror_position(p)
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
        if let Some(first) = self.positions.iter().next() {
            let mut remaining: HashSet<&Position> = self.positions.iter().collect();
            remaining.remove(first);
            let mut queue: VecDeque<Position> = VecDeque::new();
            queue.push_back(*first);
            while let Some(current) = queue.pop_front() {
                self.get_neighbours(&current)
                    .into_iter()
                    .filter(|p| remaining.remove(p))
                    .for_each(|p| queue.push_back(p));
            }
            remaining.is_empty()
        } else {
            // Galaxy contains no positions
            true
        }
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
            let hamming_distance = hamming_distances[p];
            if hamming_distance != 0 {
                let v = vectors[p];
                self.get_neighbours(p)
                    .into_iter()
                    .filter(|n| hamming_distances[n] < hamming_distance)
                    .map(|parent_position| vectors[&parent_position])
                    .filter(|parent_vector| !parent_vector.is_zero())
                    .map(|parent_vector| parent_vector.angle_to(&v))
                    .for_each(|angle_difference| swirl += angle_difference);
            }
        }

        swirl
    }

    pub fn get_cumulative_swirl(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        let center = self.center();
        let center_positions = self.center().get_center_placement().get_positions();
        let center = Vec2::from(&center) / 2.0;
        let mut partial_swirls: HashMap<Position, Vec<f64>> = HashMap::new();
        let mut queue: VecDeque<Position> = VecDeque::new();
        let mut cumulative_swirl = 0.0;
        for position in center_positions.into_iter() {
            partial_swirls.insert(position, vec![0.0]);
            queue.push_back(position);
        }
        while let Some(position) = queue.pop_front() {
            let v = Vec2::from(&position);
            let swirl = {
                let partials = partial_swirls.get(&position).unwrap();
                partials.iter().sum::<f64>() / partials.len() as f64
            };
            cumulative_swirl += swirl;
            for n in self.get_neighbours(&position) {
                let angle = (v - center).angle_to(&(Vec2::from(&n) - center));
                if !partial_swirls.contains_key(&n) {
                    queue.push_back(n);
                }
                partial_swirls
                    .entry(n)
                    .or_insert_with(Vec::new)
                    .push(swirl + angle);
            }
        }

        cumulative_swirl
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

    /// Return a map with all the positions, mapped to all positions that are one step closer to the center.
    /// For the root positions, the vec is empty.
    fn get_parent_candidates(&self) -> HashMap<Position, Vec<Position>> {
        let hamming_distances = self.get_hamming_distances();
        self.positions
            .iter()
            .map(|position| {
                let hamming_distance = hamming_distances[position];
                let candidates = self
                    .get_neighbours(position)
                    .into_iter()
                    .filter(|n| hamming_distances[n] + 1 == hamming_distance)
                    .collect();
                (*position, candidates)
            })
            .collect()
    }

    pub fn score_spanning_tree(&self, spanning_tree: &Tree) -> f64 {
        let center = Vec2::from(&self.center()) / 2.0;
        spanning_tree
            .iter()
            .map(|(child, maybe_parent)| {
                if let Some(parent) = maybe_parent {
                    let p_v = Vec2::from(parent) - center;
                    let c_v = Vec2::from(child) - center;
                    p_v.angle_to(&c_v)
                } else {
                    0.0
                }
            })
            .sum()
    }

    pub fn get_winding_spanning_tree(&self) -> HashMap<Position, (f64, Option<Position>)> {
        let center = self.center();
        let center_positions = center.get_center_placement().get_positions();
        let center_v = Vec2::from_center(&center);
        let mut parent_map = HashMap::with_capacity(self.size());
        let mut queue = VecDeque::new();
        for &p in center_positions.iter() {
            parent_map.insert(p, (0.0, None));
            queue.push_back(p);
        }
        while parent_map.len() != self.size() {
            let mut parent_candidates = HashMap::<Position, (f64, Position)>::new();
            while let Some(parent) = queue.pop_front() {
                let parent_winding_number = parent_map[&parent].0;
                let parent_v = Vec2::from(&parent) - &center_v;
                self.get_neighbours(&parent)
                    .into_iter()
                    .filter(|neighbour| !parent_map.contains_key(&neighbour))
                    .for_each(|child| {
                        let child_v = Vec2::from(&child) - &center_v;
                        let winding_number = parent_winding_number + parent_v.angle_to(&child_v);
                        parent_candidates
                            .entry(child)
                            .and_modify(|(best_winding_number, best_parent)| {
                                if winding_number.abs() > best_winding_number.abs() {
                                    *best_winding_number = winding_number;
                                    *best_parent = parent;
                                }
                            })
                            .or_insert((winding_number, parent));
                    });
            }
            for (child, (winding_number, parent)) in parent_candidates.into_iter() {
                parent_map.insert(child, (winding_number, Some(parent)));
                queue.push_back(child);
            }
        }

        parent_map
    }

    pub fn get_score(&self) -> f64 {
        let mut score = 0.0;

        if self.is_zig_zag() {
            return 0.0;
        }

        // Penalize big rectangles
        for rect in self.rectangles() {
            let area = rect.area() as f64;
            score -= area.powf(2.);
        }

        // Penalize large amounts of fat
        let skeleton = self.get_skeleton();
        {
            let fat_rate_threshold = 0.1; // Some fat is alright
            let fat_amount = self.size() - skeleton.size();
            let fat_rate = fat_amount as f64 / self.size() as f64;
            if fat_rate > fat_rate_threshold {
                score -= (fat_amount as f64).powf(2.);
            }
        }

        // Reward curly galaxies
        score += self.get_swirl().powf(2.);

        // Reward long arms
        let arms = skeleton.get_arms();
        for arm in &arms {
            score += (arm.len() as f64).powf(2.);
        }

        // Reward many long arms
        {
            let number_of_long_arms = arms.iter().filter(|arm| arm.len() > 1).count();
            score += (number_of_long_arms as f64).powf(2.5);
        }

        // Penalize huge galaxies
        if self.size() > 16 {
            score -= (self.size() as f64).powf(2.);
        }

        // Reward holes
        let holes = self.get_holes();
        score += holes.len() as f64 * 10.0;

        score
    }

    pub fn get_arms(&self) -> Vec<Vec<Position>> {
        let spanning_tree = self.get_spanning_tree();
        let mut remaining_leaves: VecDeque<Position> = {
            let hamming_distances = self.get_hamming_distances();
            let children: HashSet<Position> = spanning_tree.get_positions().into_iter().collect();
            let parents: HashSet<Position> = spanning_tree.iter().filter_map(|(_, &p)| p).collect();
            children
                .sub(&parents)
                .iter()
                .sorted_by_key(|position| hamming_distances[position])
                .cloned()
                .collect()
        };
        let mut arms = Vec::new();
        let mut visited = HashSet::new();
        while let Some(mut position) = remaining_leaves.pop_back() {
            let mut current_arm = Vec::new();
            current_arm.push(position);
            while let Some(parent) = spanning_tree.get_parent(&position) {
                if !visited.insert(parent) {
                    break;
                }
                current_arm.push(parent);
                position = parent;
            }
            arms.push(current_arm);
        }
        arms
    }

    pub fn get_spanning_tree(&self) -> Tree {
        let parent_candidates = self.get_parent_candidates();
        let center = Vec2::from_center(&self.center());
        let get_spanning_tree_internal = |discriminator: fn(f64) -> OrderedFloat<f64>| -> Tree {
            parent_candidates
                .iter()
                .map(|(child, candidates)| {
                    let child_v = Vec2::from(child) - center;
                    let parent = candidates.iter().min_by_key(|&candidate| {
                        let candidate_v = Vec2::from(candidate) - center;
                        let angle = candidate_v.angle_to(&child_v);
                        discriminator(angle)
                    });
                    (*child, parent.copied())
                })
                .collect()
        };

        if self.is_mirror_symmetric() {
            get_spanning_tree_internal(|angle| OrderedFloat(angle.abs()))
        } else {
            let clockwise = get_spanning_tree_internal(|angle| OrderedFloat(-angle));
            let counter_clockwise = get_spanning_tree_internal(|angle| OrderedFloat(angle));
            if self.score_spanning_tree(&counter_clockwise).abs()
                > self.score_spanning_tree(&clockwise).abs()
            {
                counter_clockwise
            } else {
                clockwise
            }
        }
    }

    /// Returns the average number of neighbours of each position
    fn get_thickness(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        self.positions
            .iter()
            .map(|p| self.get_neighbours(p).len() as f64)
            .sum::<f64>()
            / self.size() as f64
    }

    fn get_skeleton(&self) -> Galaxy {
        let mut skeleton = self.clone();
        let center = skeleton.center();
        let center_positions = center.get_center_placement().get_positions();
        let mirror_symmetric = skeleton.is_mirror_symmetric();
        loop {
            let mut maybe_fat = skeleton.positions.iter().sorted().find(|position| {
                if center_positions.contains(position) {
                    return false;
                }
                let north = position.up();
                let n = skeleton.contains_position(&north);
                let west = position.left();
                let w = skeleton.contains_position(&west);
                let south = position.down();
                let s = skeleton.contains_position(&south);
                let east = position.right();
                let e = skeleton.contains_position(&east);

                match (n, w, s, e) {
                    (true, true, false, false) => {
                        let north_west = north.left();
                        skeleton.contains_position(&north_west)
                    }
                    (true, false, false, true) => {
                        let north_east = north.right();
                        skeleton.contains_position(&north_east)
                    }
                    (false, true, true, false) => {
                        let south_west = south.left();
                        skeleton.contains_position(&south_west)
                    }
                    (false, false, true, true) => {
                        let south_east = south.right();
                        skeleton.contains_position(&south_east)
                    }
                    _ => false,
                }
            });
            if maybe_fat.is_none() {
                maybe_fat = skeleton.positions.iter().sorted().find(|position| {
                    if center_positions.contains(position) {
                        return false;
                    }
                    let north = position.up();
                    let n = skeleton.contains_position(&north);
                    let west = position.left();
                    let w = skeleton.contains_position(&west);
                    let south = position.down();
                    let s = skeleton.contains_position(&south);
                    let east = position.right();
                    let e = skeleton.contains_position(&east);

                    match (n, w, s, e) {
                        (false, true, true, true) => {
                            let south_west = south.left();
                            let south_east = south.right();
                            skeleton.contains_position(&south_west)
                                && skeleton.contains_position(&south_east)
                        }
                        (true, false, true, true) => {
                            let north_east = north.right();
                            let south_east = south.right();
                            skeleton.contains_position(&north_east)
                                && skeleton.contains_position(&south_east)
                        }
                        (true, true, false, true) => {
                            let north_west = north.left();
                            let north_east = north.right();
                            skeleton.contains_position(&north_west)
                                && skeleton.contains_position(&north_east)
                        }
                        (true, true, true, false) => {
                            let north_west = north.left();
                            let south_west = south.left();
                            skeleton.contains_position(&north_west)
                                && skeleton.contains_position(&south_west)
                        }
                        _ => false,
                    }
                });
            }
            if let Some(fat) = maybe_fat.copied() {
                skeleton.remove_position(&fat);
                let diagonal_mirror = center.mirror_position(&fat);
                skeleton.remove_position(&diagonal_mirror);
                let horizontal_mirror = Position::new(fat.row, diagonal_mirror.column);
                let vertical_mirror = Position::new(diagonal_mirror.row, fat.column);
                if mirror_symmetric
                    && !fat.is_adjacent_to(&horizontal_mirror)
                    && !fat.is_adjacent_to(&vertical_mirror)
                {
                    skeleton.remove_position(&horizontal_mirror);
                    skeleton.remove_position(&vertical_mirror);
                }
            } else {
                break;
            }
        }
        skeleton
    }

    /// Returns whether every cell of the galaxy is mirrored
    /// across the vertical axis passing through the center.
    /// E.g. the first of the following galaxies is mirror symmetric,
    /// but the second is not.
    /// 1) ┌─┐ ┌─┐  2) ┌─┐
    ///    │ └─┘ │     │ └───┐
    ///    │ ┌─┐ │     └───┐ │
    ///    └─┘ └─┘         └─┘
    fn is_mirror_symmetric(&self) -> bool {
        let center = self.center();
        self.positions
            .iter()
            .map(|p| Position::new(p.row, center.column - p.column))
            .all(|p| self.positions.contains(&p))
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

    /// Returns whether the galaxy surrounds the position,
    /// even though the position does not belong to the galaxy
    fn is_hole(&self, p: &Position) -> bool {
        !self.contains_position(p) && self.get_neighbours(p).len() == 4
    }

    fn get_holes(&self) -> Vec<Position> {
        self.get_bounding_rectangle()
            .positions()
            .iter()
            .filter(|p| self.is_hole(p))
            .cloned()
            .collect()
    }

    fn is_turn(&self, p: &Position) -> bool {
        if self.contains_position(p) {
            let up = self.contains_position(&p.up());
            let down = self.contains_position(&p.down());
            let left = self.contains_position(&p.left());
            let right = self.contains_position(&p.right());
            match (up, right, down, left) {
                (true, true, false, false) => true,
                (false, true, true, false) => true,
                (false, false, true, true) => true,
                (true, false, false, true) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn is_leaf(&self, p: &Position) -> bool {
        if self.contains_position(p) {
            let up = self.contains_position(&p.up());
            let down = self.contains_position(&p.down());
            let left = self.contains_position(&p.left());
            let right = self.contains_position(&p.right());
            match (up, right, down, left) {
                (true, false, false, false) => true,
                (false, true, false, false) => true,
                (false, false, true, false) => true,
                (false, false, false, true) => true,
                (false, false, false, false) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn is_zig_zag(&self) -> bool {
        self.get_positions()
            .all(|p| self.is_turn(&p) || self.is_leaf(&p))
            && self.get_positions().any(|p| self.is_turn(&p))
    }
}

impl Display for Galaxy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bounds = self.get_bounding_rectangle();
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

impl From<Position> for Galaxy {
    fn from(position: Position) -> Self {
        Self::from([position])
    }
}

impl FromIterator<Position> for Galaxy {
    fn from_iter<I: IntoIterator<Item = Position>>(iter: I) -> Self {
        Galaxy::from(iter)
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
        use std::f64::consts::PI;

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

        #[test]
        fn known_shapes() {
            let galaxy = Galaxy::from_string(
                "
                ▉
                ▉▉
                 ▉
                ",
            );
            assert_eq!(galaxy.get_swirl(), 2f64.atan2(1.) * 2.);
            let galaxy = Galaxy::from_string(
                "
                 ▉▉
                ▉▉
                ",
            );
            assert_eq!(galaxy.get_swirl(), 2f64.atan2(1.) * 2.);
            let galaxy = Galaxy::from_string(
                "
                ▉ ▉▉▉
                ▉▉▉ ▉
                ",
            );
            assert_abs_diff_eq!(
                galaxy.get_swirl(),
                2. * (PI / 2.0 + 1f64.atan2(4.)),
                epsilon = 1e-8
            );
            let galaxy = Galaxy::from_string(
                "
                  ▉▉▉
                ▉ ▉ ▉
                ▉▉▉
                ",
            );
            assert_abs_diff_eq!(galaxy.get_swirl(), PI, epsilon = 1e-8);
            let galaxy = Galaxy::from_string(
                "
                ▉
                ▉ ▉▉▉
                ▉ ▉ ▉
                ▉▉▉ ▉
                    ▉
                ",
            );
            assert_abs_diff_eq!(galaxy.get_swirl(), PI * 1.5, epsilon = 1e-8);
            let galaxy = Galaxy::from_string(
                "
                ▉▉▉
                ▉
                ▉ ▉▉▉
                ▉ ▉ ▉
                ▉▉▉ ▉
                    ▉
                  ▉▉▉
                ",
            );
            assert_abs_diff_eq!(galaxy.get_swirl(), PI * 2.0, epsilon = 1e-8);
            let galaxy = Galaxy::from_string(
                "
                 ▉▉▉
                ▉▉
                ▉▉ ▉▉▉
                ▉▉ ▉ ▉▉
                 ▉▉▉ ▉▉
                     ▉▉
                   ▉▉▉
                ",
            );
            assert_gt!(galaxy.get_swirl(), PI * 2.0,);
        }
    }

    mod curl {
        use crate::model::galaxy::Galaxy;
        use crate::model::position::Position;
        use crate::model::rectangle::Rectangle;
        use approx::assert_abs_diff_eq;
        use more_asserts::assert_gt;
        use std::f64::consts::PI;

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

        #[test]
        fn known_shapes() {
            assert_abs_diff_eq!(
                Galaxy::from_string(
                    "
                     ▉▉
                    ▉▉
                    ",
                )
                .get_curl(),
                PI,
                epsilon = 1e-8
            );
            assert_abs_diff_eq!(
                Galaxy::from_string(
                    "
                    ▉▉
                     ▉▉
                    ",
                )
                .get_curl(),
                -PI,
                epsilon = 1e-8
            );
            assert_abs_diff_eq!(
                Galaxy::from_string(
                    "
                      ▉▉▉
                    ▉▉▉
                    ",
                )
                .get_curl(),
                PI,
                epsilon = 1e-8
            );
            assert_abs_diff_eq!(
                Galaxy::from_string(
                    "
                    ▉ ▉▉▉
                    ▉▉▉ ▉
                    ",
                )
                .get_curl(),
                2.0 * PI,
                epsilon = 1e-8
            );
            assert_abs_diff_eq!(
                Galaxy::from_string(
                    "
                    ▉▉▉▉▉
                    ▉
                    ▉ ▉▉▉
                    ▉▉▉ ▉
                        ▉
                    ▉▉▉▉▉
                    ",
                )
                .get_curl(),
                3.0 * PI,
                epsilon = 1e-8
            );
        }
    }

    mod get_skeleton {
        use crate::model::galaxy::Galaxy;

        #[test]
        fn known_shapes() {
            assert_eq!(
                Galaxy::from_string(
                    "
                    ▉▉▉
                    ▉▉▉
                    ",
                )
                .get_skeleton(),
                Galaxy::from_string(
                    "
                     ▉▉
                    ▉▉
                    "
                )
            );
            assert_eq!(
                Galaxy::from_string(
                    "
                    ▉▉▉
                    ▉▉▉
                    ▉▉▉
                    ",
                )
                .get_skeleton(),
                Galaxy::from_string(
                    "
                     ▉
                    ▉▉▉
                     ▉
                    "
                )
            );
            assert_eq!(
                Galaxy::from_string(
                    "
                    ▉▉▉▉
                    ▉▉▉▉
                    ▉▉▉▉
                    ▉▉▉▉
                    ",
                )
                .get_skeleton(),
                Galaxy::from_string(
                    "
                      ▉
                     ▉▉▉
                    ▉▉▉
                     ▉
                    "
                )
            );
            assert_eq!(
                Galaxy::from_string(
                    "
                    ▉▉▉▉▉
                    ▉▉▉▉▉
                    ▉▉▉▉▉
                    ▉▉▉▉▉
                    ▉▉▉▉▉
                    ",
                )
                .get_skeleton(),
                Galaxy::from_string(
                    "
                      ▉  
                      ▉  
                    ▉▉▉▉▉
                      ▉   
                      ▉  
                    "
                )
            );
            let original = Galaxy::from_string(
                "
                 ▉▉▉
                ▉▉
                ▉▉ ▉▉▉
                ▉▉ ▉ ▉▉
                 ▉▉▉ ▉▉
                     ▉▉
                   ▉▉▉
                ",
            );
            let expected = Galaxy::from_string(
                "
                 ▉▉▉
                 ▉
                 ▉ ▉▉▉
                ▉▉ ▉ ▉▉
                 ▉▉▉ ▉
                     ▉
                   ▉▉▉
                ",
            );
            let actual = original.get_skeleton();
            assert_eq!(actual, expected, "Expected:\n{expected}\nActual:\n{actual}");
            let original = Galaxy::from_string(
                "
                 ▉
                 ▉▉
                ▉▉▉▉▉
                ▉▉▉▉
                 ▉▉▉▉
                ▉▉▉▉▉
                  ▉▉
                   ▉
                ",
            );
            let expected = Galaxy::from_string(
                "
                 ▉
                 ▉ 
                 ▉ ▉▉
                ▉▉▉▉
                 ▉▉▉▉
                ▉▉ ▉ 
                   ▉
                   ▉
                ",
            );
            let actual = original.get_skeleton();
            assert_eq!(actual, expected, "Expected:\n{expected}\nActual:\n{actual}");
            let original = Galaxy::from_string(
                "
                  ▉
                 ▉▉▉
                ▉▉▉▉▉▉
                  ▉▉▉
                   ▉
                ",
            );
            let expected = Galaxy::from_string(
                "
                  ▉
                  ▉
                ▉▉▉▉▉▉
                   ▉
                   ▉
                ",
            );
            let actual = original.get_skeleton();
            assert_eq!(actual, expected, "Expected:\n{expected}\nActual:\n{actual}");
            let original = Galaxy::from_string(
                "
                  ▉
                 ▉▉▉▉
                 ▉▉▉▉
                  ▉▉
                ▉▉▉▉▉
                 ▉▉
                ▉▉▉▉
                ▉▉▉▉
                  ▉
                ",
            );
            let expected = Galaxy::from_string(
                "
                  ▉
                  ▉
                 ▉▉▉▉
                   ▉
                ▉▉▉▉▉
                 ▉
                ▉▉▉▉
                  ▉
                  ▉
                ",
            );
            let actual = original.get_skeleton();
            assert_eq!(actual, expected, "Expected:\n{expected}\nActual:\n{actual}");
            let original = Galaxy::from_string(
                "
                ▉▉ ▉▉
                ▉ ▉▉▉▉
                ▉▉▉▉ ▉
                 ▉▉ ▉▉
                ",
            );
            let expected = Galaxy::from_string(
                "
                ▉▉  ▉
                ▉ ▉▉▉▉
                ▉▉▉▉ ▉
                 ▉  ▉▉
                ",
            );
            let actual = original.get_skeleton();
            assert_eq!(actual, expected, "Expected:\n{expected}\nActual:\n{actual}");
            let original = Galaxy::from_string(
                "
                  ▉
                 ▉▉▉
                ▉▉▉ ▉
                ▉ ▉▉▉
                 ▉▉▉
                  ▉
                ",
            );
            let expected = Galaxy::from_string(
                "
                  ▉
                  ▉▉
                ▉▉▉ ▉
                ▉ ▉▉▉
                 ▉▉ 
                  ▉
                ",
            );
            let actual = original.get_skeleton();
            assert_eq!(actual, expected, "Expected:\n{expected}\nActual:\n{actual}");
        }
    }

    mod get_score {
        use crate::model::galaxy::Galaxy;

        // #[test]
        fn debug_score() {
            //       ┌─┐
            // ┌─────┘ │
            // └───┐   └─┐
            // ┌───┘ ┌───┘
            // └─┐   └───┐
            //   │ ┌─────┘
            //   └─┘
            let galaxy = Galaxy::from_string(
                "
                   x
                xxxx
                  xxx
                xxx
                 xxxx
                 x
                ",
            );
            assert_eq!(galaxy.get_score(), 0.0);
        }

        #[test]
        fn cool_galaxies_should_have_higher_score_than_boring_galaxies() {
            let cool_galaxies: Vec<Galaxy> = vec![
                "
                ▉▉▉  ▉▉
                ▉ ▉▉▉▉ ▉
                 ▉▉  ▉▉▉
                ",
                "
                  ▉
                ▉▉▉
                 ▉▉▉
                 ▉
                ",
                "
                ▉▉  ▉
                ▉ ▉▉▉▉
                ▉▉▉▉ ▉
                 ▉  ▉▉
                ",
            ]
            .iter()
            .map(|string| Galaxy::from_string(string))
            .collect();

            let boring_galaxies: Vec<Galaxy> = vec![
                "
                ▉▉
                ▉▉
                ",
            ]
            .iter()
            .map(|string| Galaxy::from_string(string))
            .collect();

            for cool_galaxy in &cool_galaxies {
                for boring_galaxy in &boring_galaxies {
                    assert!(cool_galaxy.get_score() > boring_galaxy.get_score());
                }
            }
        }
    }
}
