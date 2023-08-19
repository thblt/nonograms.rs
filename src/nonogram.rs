use std::ops::{Index, IndexMut};

// * The Nonogram type

#[derive(Debug)]
pub struct Nonogram {
    // @FIXME All fields should be private.
    width: usize,
    height: usize,
    pub cells: Vec<CellState>,
    pub rows: Vec<Constraint>,
    pub cols: Vec<Constraint>,
}

impl Nonogram {
    /// Create a new, unconstrained (and thus unsolvable) nonogram of
    /// dimensions width*height.
    pub fn new(width: usize, height: usize, cols: Vec<Constraint>, rows: Vec<Constraint>) -> Nonogram {
        Nonogram {
            width,
            height,
            cells: vec![CellState::Undecided; width * height],
            rows,
            cols,
        }
    }

    pub fn builder() -> NonogramBuilder {
        NonogramBuilder::new()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Return a view into a column (starting at 0).  This can't be
    /// made mutable, since columns aren't internally continuous.
    pub fn column(&self, x: usize) -> Option<Vec<CellState>> {
        if x >= self.width {
            None
        } else {
            let mut ret = Vec::with_capacity(self.height);
            for y in 0..self.height {
                ret.push(self[(x, y)]);
            }
            Some(ret)
        }
    }

    /// Return a view into a row.  Unlike [column], this is a real
    /// slice.
    pub fn row(&self, y: usize) -> Option<&[CellState]> {
        if y >= self.height {
            None
        } else {
            return Some(&self.cells[self.xy_to_index(0, y)..self.xy_to_index(self.width, y)]);
        }
    }

    #[inline]
    pub fn xy_to_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Generate a simple representation of this 'gram
    /// using Unicode box-drawing characters.
    pub fn as_text(&self) -> String {
        let mut ret = String::new();
        // let chars = [' ', '▀', '▄', '█'];
        for y in 0..self.height {
            for x in 0..self.width {
                ret.push(match self[(x, y)] {
                    CellState::Undecided => '?',
                    CellState::Empty => ' ',
                    CellState::Filled => '█',
                })
            }
            ret.push('\n')
        }
        ret
    }

    pub fn clear_solution(&mut self) {
        self.cells.fill(CellState::Undecided)
    }
}

impl Index<(usize, usize)> for Nonogram {
    type Output = CellState;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.cells[self.xy_to_index(x, y)]
    }
}

impl IndexMut<(usize, usize)> for Nonogram {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        let index = self.xy_to_index(x, y);
        &mut self.cells[index]
    }
}

pub type Constraint = Vec<usize>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CellState {
    Undecided,
    Empty,
    Filled,
}

impl From<bool> for CellState {
    fn from(b: bool) -> Self {
        match b {
            true => CellState::Filled,
            false => CellState::Empty,
        }
    }
}

impl CellState {
    /// A folding (reduction) helper function to determine consensus.
    /// It is used to fold a series of candidate distributions to
    /// their common part.  It is just equality, but instead of
    /// returning a bool it returns CellState::Undecided if terms are
    /// not equal.
    pub fn consensus_eq(&self, other: &CellState) -> CellState {
        if *self == *other {
            *self
        } else {
            CellState::Undecided
        }
    }

    /// Determine if other can be applied over self.  This is an
    /// helper to determine if a candidate is compatible with the
    /// grid.
    ///
    /// self is a state in the grid, and over a candidate.  This is
    /// true if self is [CellState::Undecided], or if self and other
    /// are the same value.
    pub fn accepts(&self, other: &CellState) -> bool {
        *self == CellState::Undecided || self == other
        // println!("{:?} accepts {:?}? {:?}", self, other, ret);
    }
}

pub struct NonogramBuilder {
    pub width: Option<usize>,
    pub height: Option<usize>,
    rows: Vec<Constraint>,
    cols: Vec<Constraint>,
}

#[derive(Debug)]
pub enum BuilderError {
    Invalid,
    WidthAlreadySet,
    HeightAlreadySet,
}

type BuilderResult<T> = Result<T, BuilderError>;

impl NonogramBuilder {
    pub fn new() -> NonogramBuilder {
        NonogramBuilder {
            width: None,
            height: None,
            rows: vec![],
            cols: vec![],
        }
    }

    #[must_use]
    pub fn width(&mut self, width: usize) -> BuilderResult<&mut Self> {
        match self.width {
            Some(_) => Err(BuilderError::WidthAlreadySet),
            None => {
                self.width = Some(width);
                Ok(self)
            }
        }
    }

    pub fn height(&mut self, height: usize) -> BuilderResult<&mut Self> {
        match self.height {
            Some(_) => Err(BuilderError::HeightAlreadySet),
            None => {
                self.height = Some(height);
                Ok(self)
            }
        }
    }

    pub fn push_row_constraint(&mut self, constraint: Constraint) -> &mut Self {
        self.rows.push(constraint);
        self
    }

    pub fn push_col_constraint(&mut self, constraint: Constraint) -> &mut Self {
        self.cols.push(constraint);
        self
    }

    pub fn validate(&self) -> BuilderResult<()> {
        if self.width.is_none()
            || self.height.is_none()
            || self.height.unwrap() != self.rows.len()
            || self.width.unwrap() != self.cols.len()
        {
            Err(BuilderError::Invalid)
        } else {
            Ok(())
        }
    }


    pub fn build(self) -> BuilderResult<Nonogram> {
        self.validate()?;
        Ok (
            Nonogram::new(
                self.width.unwrap(),
                self.height.unwrap(),
                self.cols,
                self.rows)
        )
    }

}

impl Default for NonogramBuilder {
    fn default() -> Self {
        NonogramBuilder::new()
    }
}
