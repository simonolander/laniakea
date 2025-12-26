use crate::model::position::Position;

#[derive(Eq, PartialEq, Default, Debug, Ord, PartialOrd, Copy, Clone)]
pub struct Rectangle {
    /** Smallest row index, inclusive */
    pub min_row: i32,
    /** Largest row index, exclusive */
    pub max_row: i32,
    /** Smallest column index, inclusive */
    pub min_column: i32,
    /** Largest column index, exclusive */
    pub max_column: i32,
}

impl Rectangle {
    pub fn new(min_row: i32, max_row: i32, min_column: i32, max_column: i32) -> Rectangle {
        Rectangle {
            min_row,
            max_row,
            min_column,
            max_column,
        }
    }

    /// Returns the smallest rectangle that contains the given positions
    pub fn bounding_rectangle(positions: impl IntoIterator<Item = Position>) -> Rectangle {
        let mut min_row = i32::MAX;
        let mut max_row = i32::MIN;
        let mut min_column = i32::MAX;
        let mut max_column = i32::MIN;
        let mut any_position = false;
        for position in positions {
            min_row = min_row.min(position.row);
            max_row = max_row.max(position.row);
            min_column = min_column.min(position.column);
            max_column = max_column.max(position.column);
            any_position = true;
        }
        if any_position {
            Rectangle::new(min_row, max_row, min_column, max_column)
        } else {
            Rectangle::default()
        }
    }

    pub fn width(&self) -> i32 {
        self.max_column - self.min_column
    }

    pub fn height(&self) -> i32 {
        self.max_row - self.min_row
    }

    pub fn area(&self) -> i32 {
        self.width() * self.height()
    }

    pub fn top_left(&self) -> Position {
        Position::new(self.min_row, self.min_column)
    }

    pub fn top_right(&self) -> Position {
        Position::new(self.min_row, self.max_column)
    }

    pub fn bottom_left(&self) -> Position {
        Position::new(self.max_row, self.min_column)
    }

    pub fn bottom_right(&self) -> Position {
        Position::new(self.max_row, self.max_column)
    }

    pub fn corners(&self) -> Vec<Position> {
        let single_row = self.min_row == self.max_row;
        let single_column = self.min_column == self.max_column;
        if single_row && single_column {
            vec![self.top_left()]
        } else if single_row || single_column {
            vec![self.top_left(), self.bottom_right()]
        } else {
            vec![
                self.top_left(),
                self.top_right(),
                self.bottom_left(),
                self.bottom_right(),
            ]
        }
    }

    pub fn positions(&self) -> Vec<Position> {
        (self.min_row..self.max_row)
            .flat_map(|row| {
                (self.min_column..self.max_column).map(move |column| Position::new(row, column))
            })
            .collect()
    }
}

impl From<&(usize, usize)> for Rectangle {
    fn from((width, height): &(usize, usize)) -> Self {
        Rectangle::new(0, *height as i32, 0, *width as i32)
    }
}

#[cfg(test)]
mod test {
    use crate::model::rectangle::Rectangle;
    use proptest::prelude::*;

    impl Arbitrary for Rectangle {
        type Parameters = ();

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            let rows = any::<(i8, i8)>().prop_filter(
                "Min row should be smaller or equal to max row",
                |(min_row, max_row)| min_row <= max_row,
            );

            // Generate valid `min_column` and `max_column` where `min_column <= max_column`
            let columns = any::<(i8, i8)>().prop_filter(
                "Min column should be smaller or equal to max column",
                |(min_col, max_col)| min_col <= max_col,
            );

            // Combine row and column strategies into a Rectangle
            (rows, columns)
                .prop_map(|((min_row, max_row), (min_column, max_column))| Rectangle {
                    min_row: min_row as i32,
                    max_row: max_row as i32,
                    min_column: min_column as i32,
                    max_column: max_column as i32,
                })
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }

    #[test]
    fn test_from_width_and_height() {
        for width in 1..10 {
            for height in 1..10 {
                let rect = Rectangle::from(&(width, height));
                assert_eq!(rect, Rectangle::new(0, height as i32, 0, width as i32));
                assert_eq!(rect.positions().len(), width * height);
            }
        }
    }
}
