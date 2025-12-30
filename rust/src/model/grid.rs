use crate::model::position::Position;
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct Grid<T>(Vec<Vec<T>>);

impl<T> Grid<T> {
    pub fn new(width: usize, height: usize, default: T) -> Self
    where
        T: Clone,
    {
        Grid(vec![vec![default; width]; height])
    }

    pub fn width(&self) -> usize {
        self.0.first().map(|row| row.len()).unwrap_or(0)
    }

    pub fn height(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Position, &T)> {
        self.0.iter().enumerate().flat_map(|(r, row)| {
            row.iter()
                .enumerate()
                .map(move |(c, value)| (Position::from((r, c)), value))
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Position, &mut T)> {
        self.0.iter_mut().enumerate().flat_map(|(r, row)| {
            row.iter_mut()
                .enumerate()
                .map(move |(c, value)| (Position::from((r, c)), value))
        })
    }
}

impl<'a, T> IntoIterator for &'a Grid<T> {
    type Item = (Position, &'a T);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

impl<'a, T> IntoIterator for &'a mut Grid<T> {
    type Item = (Position, &'a mut T);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter_mut())
    }
}

impl<T> Index<&Position> for Grid<T> {
    type Output = T;
    fn index(&self, index: &Position) -> &Self::Output {
        &self.0[index.row as usize][index.column as usize]
    }
}

impl<T> IndexMut<&Position> for Grid<T> {
    fn index_mut(&mut self, index: &Position) -> &mut Self::Output {
        &mut self.0[index.row as usize][index.column as usize]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::grid::Grid;
    use crate::model::position::Position;

    #[test]
    fn test_grid_iter() {
        let mut grid = Grid::new(2, 2, 0);
        grid[&Position::new(0, 0)] = 1;
        grid[&Position::new(0, 1)] = 2;
        grid[&Position::new(1, 0)] = 3;
        grid[&Position::new(1, 1)] = 4;

        let items: Vec<(Position, &i32)> = grid.iter().collect();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], (Position::new(0, 0), &1));
        assert_eq!(items[1], (Position::new(0, 1), &2));
        assert_eq!(items[2], (Position::new(1, 0), &3));
        assert_eq!(items[3], (Position::new(1, 1), &4));
    }

    #[test]
    fn test_grid_iter_mut() {
        let mut grid = Grid::new(2, 2, 0);
        for (_pos, val) in grid.iter_mut() {
            *val += 1;
        }
        assert_eq!(grid[&Position::new(0, 0)], 1);
        assert_eq!(grid[&Position::new(1, 1)], 1);
    }
}
