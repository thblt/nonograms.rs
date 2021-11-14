use std::fmt::{self, Display};
use std::io::Read;
use std::ops::{Index, IndexMut};

// * The Nonogram type

#[derive(Debug)]
pub struct Nono {
    pub width: usize,
    pub height: usize,
    cells: Vec<CellState>,
    rows: Vec<Constraint>,
    cols: Vec<Constraint>,
}

impl Index<(usize, usize)> for Nono {
    type Output = CellState;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.cells[y * self.width + x]
    }
}

impl IndexMut<(usize, usize)> for Nono {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.cells[y * self.width + x]
    }
}

impl Nono {
    /// Create a new, unconstrained (and thus unsolvable) nonogram of
    /// dimensions width*height.
    pub fn new(width: usize, height: usize) -> Nono {
        Nono {
            width,
            height,
            cells: vec![CellState::Undecided; width * height],
            rows: Vec::with_capacity(height),
            cols: Vec::with_capacity(width),
        }
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

type Constraint = Vec<usize>;

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
    fn consensus_eq(&self, other: &CellState) -> CellState {
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
    fn accepts(&self, other: &CellState) -> bool {
        *self == CellState::Undecided || self == other
    }
}

// * A solver

/// A solver for nonograms.
///
/// Solving nonograms is a relatively simple operation.
///
/// First, for each row and column, we compute every possible
/// distribution of the sequence of filled cases, that is, every
/// possible location of the sequences of filled cells.  For example,
/// for 1 2 in a 5-wide grid, that goes:
///
/// <pre>
/// X_XX__
/// X__XX_
/// X___XX
/// _X_XX_
/// _X__XX
/// __X_XX
/// </pre>
///
/// Then, iteratively, and for each row and column:
///
///  1. Fold the list of possible sequences to identify cells that are
///     always filled or always empty.  If there are some, we put them
///     on the result.
///
///  2. Filter out from that list the sequences that don't match with
///     what we know of the grid, eg those with a filled square where
///     we know there must be an empty square, and so on.

pub mod solver {
    use super::*;
    use std::iter::once;

    // @FIXME Convert into newtype.
    type CandidateMask = Vec<CellState>;
    type CandidateMaskSet = Vec<CandidateMask>;

    pub struct Solver<'a> {
        rows: Vec<CandidateMaskSet>,
        cols: Vec<CandidateMaskSet>,
        nono: &'a mut Nono,
    }

    impl<'a> Solver<'a> {
        pub fn new(from: &'a mut Nono) -> Solver<'a> {
            Solver {
                rows: from
                    .rows
                    .iter()
                    .map(|r| candidates(r, from.width))
                    .collect(),
                cols: from
                    .cols
                    .iter()
                    .map(|r| candidates(r, from.height))
                    .collect(),
                nono: from,
            }
        }

        pub fn solve(&mut self) {
            // TODO Prepare
            self.nono.clear_solution();

            self.consensus_step();
            self.filter_step();
            // TODO Finalize
        }

        /// The consensus determines the cells that *must* be empty or
        /// filled given the constraint (and only the constraint, not
        /// the state of the grid) for each row and column, and marks
        /// those cells' statuses on the grid.
        fn consensus_step(&mut self) {
            // Rows
            for (y, row) in self.rows.iter().enumerate() {
                let consensus = find_consensus(row);
                let mut i = 0;
                for (x, square) in consensus.iter().enumerate() {
                    i += 1;
                    if *square != CellState::Undecided {
                        // FIXME don't overwrite a different value: check for conflicts.
                        self.nono[(x, y)] = *square;
                    }
                }
            }

            // Columns
            for (x, col) in self.cols.iter().enumerate() {
                let consensus = find_consensus(col);
                for (y, square) in consensus.iter().enumerate() {
                    if *square != CellState::Undecided {
                        // FIXME don't overwrite a different value: check for conflicts.
                        self.nono[(x, y)] = *square;
                    }
                }
            }
        }

        /// The filter step eliminates, for each row and column, the
        /// candidates that don't fit in the grid, that is, that
        /// require that a cell marked as empty be filled, or filled
        /// be empty.
        fn filter_step(&self) {

        }


    }

    /// Find the intersection of a set of a [CandidateMask], that is,
    /// the common part of all the masks in the set.
    pub fn find_consensus(cands: &CandidateMaskSet) -> CandidateMask {
        let mut ret: CandidateMask = cands[0].clone();

        // @FIXME Use a real iterator (this was tricky when I tried.)
        for cand in cands.iter().skip(1) {
            ret = ret
                .iter()
                .zip(cand.iter())
                .map(|(a, b)| a.consensus_eq(b))
                .collect()
        }
        ret
    }

    /// Generate the full set of candidates for a constraint and a
    /// given capacity (height or width)
    pub fn candidates(constraint: &Constraint, capacity: usize) -> CandidateMaskSet {
        // How many sequences of blanks we need.
        let count = constraint.len() + 1;
        // The total count of squares to fill.
        let occupation = constraint.iter().sum::<usize>();
        // The number of blanks to distribute.
        let blanks = capacity - occupation;

        // println!(
        //     "Proceeding ({:?} in {}). {} seqs, {} squares to fill, so {} blanks",
        //     constraint, capacity, count, occupation, blanks
        // );

        let mut results = vec![];
        make_candidates(blanks, 1, count, vec![], &mut results);

        results
            .iter()
            .map(|cand| into_mask(cand, constraint))
            .collect()
    }

    /// Convert a [Vec<usize>] as produced by [candidates] and a
    /// [Constraint] as lists of lenghths into a CellState mask.
    pub fn into_mask(empty: &Vec<usize>, filled: &Constraint) -> CandidateMask {
        assert!(empty.len() == filled.len() + 1);
        let mut ret: CandidateMask = vec![];
        for (e, f) in empty.iter().zip(filled.iter().chain(once::<&usize>(&0))) {
            for _ in 0..*e {
                ret.push(CellState::Empty);
            }
            for _ in 0..*f {
                ret.push(CellState::Filled);
            }
        }
        ret
    }

    /// Recursively generate the candidate set.
    fn make_candidates(
        blanks: usize,
        nth_seq: usize,
        total_seqs: usize,
        base: Vec<usize>,
        results: &mut Vec<Vec<usize>>,
    ) {
        if nth_seq > total_seqs {
            if blanks == 0 {
                results.push(base);
            }
            return;
        }

        let min = if nth_seq == 1 || nth_seq == total_seqs {
            0
        } else {
            1
        };

        for i in min..blanks + 1 {
            let mut next = base.clone();
            next.push(i);
            make_candidates(blanks - i, nth_seq + 1, total_seqs, next, results);
        }
    }
}

// * A parser for nonograms

pub mod parser {
    use super::*;

    #[derive(Default)]
    pub struct Parser {
        width: Option<usize>,
        height: Option<usize>,
        nono: Option<Nono>,
        line: usize,
        mode: ParserMode,
    }

    enum ParserMode {
        Main,
        Cols,
        Rows,
    }

    impl Default for ParserMode {
        fn default() -> Self {
            ParserMode::Main
        }
    }

    #[derive(Debug)]
    pub struct ParserError {
        message: String,
        line: usize,
    }

    impl Display for ParserError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            Ok(write!(f, "Parse error [{}] {}", self.line, self.message)?)
        }
    }

    impl Parser {
        pub fn new() -> Parser {
            Parser::default()
        }

        /// Parse a nonogram using the format of
        /// <https://github.com/mikix/nonogram-db/>
        pub fn parse(mut self, f: &mut impl Read) -> Result<Nono, ParserError> {
            let mut source: String = String::default();

            // Read input
            f.read_to_string(&mut source)
                .map_err(|_| self.err("Cannot parse source as UTF-8."))?;

            for line in source.lines() {
                self.line += 1;
                match self.mode {
                    ParserMode::Main => self.parse_header_line(line)?,
                    ParserMode::Cols => self.parse_constraint_line(line)?,
                    ParserMode::Rows => self.parse_constraint_line(line)?,
                }
            }

            let err = self.err("Empty/invalid file.");
            Ok(self.nono.ok_or(err)?)
        }

        fn parse_header_line(&mut self, line: &str) -> Result<(), ParserError> {
            let (command, args) = line.split_at(line.find(' ').unwrap_or(line.len()));
            match command {
                "columns" => self.mode = ParserMode::Cols,
                "rows" => self.mode = ParserMode::Rows,
                "height" => {
                    self.height = Some(
                        args.trim()
                            .parse::<usize>()
                            .map_err(|_| self.err("Cannot parse height."))?,
                    )
                }
                "width" => {
                    self.width = Some(
                        args.trim()
                            .parse::<usize>()
                            .map_err(|_| self.err("Cannot parse width."))?,
                    )
                }
                "goal" => {
                    self.ensure_nono()?;
                    self.nono.as_mut().unwrap().cells = unquote(args.trim())
                        .chars()
                        .map(|c| match c {
                            '0' => Ok(CellState::Empty),
                            '1' => Ok(CellState::Filled),
                            _ => Err(self.err("Cannot parse goal")),
                        })
                        .collect::<Result<Vec<CellState>, ParserError>>()?;
                }
                _ => (),
            }
            Ok(())
        }

        ///
        fn parse_constraint_line(&mut self, line: &str) -> Result<(), ParserError> {
            let line = line.trim();
            if line.is_empty() {
                self.mode = ParserMode::Main;
                return Ok(());
            }
            let parsed = line
                .split(",")
                .map(str::trim)
                .map(str::parse::<usize>)
                .collect::<Result<Vec<usize>, _>>();

            if let Ok(vec) = parsed {
                self.ensure_nono()?;
                match &self.mode {
                    ParserMode::Rows => &self.nono.as_mut().unwrap().rows.push(vec),
                    ParserMode::Cols => &self.nono.as_mut().unwrap().cols.push(vec),
                    ParserMode::Main => return Err(self.err("Abnormal parser state!")),
                };
            } else {
                self.mode = ParserMode::Main;
                // Because there may not be a blank line after the last column or row.
                return self.parse_header_line(line);
            };
            Ok(())
        }

        /// Initialize the inner Nono object, if it isn't already
        fn ensure_nono(&mut self) -> Result<(), ParserError> {
            if self.nono.is_none() {
                self.nono = Some(Nono::new(
                    self.width.ok_or(self.err("Missing width information."))?,
                    self.height.ok_or(self.err("Missing heightinformation."))?,
                ));
            }
            Ok(())
        }

        fn err(&self, msg: &str) -> ParserError {
            ParserError {
                message: String::from(msg),
                line: self.line,
            }
        }
    }

    // ** Parser utilities

    /// Remove surrounding quotes from a string.
    fn unquote(s: &str) -> String {
        if s.starts_with("\"") && s.ends_with("\"") {
            s[1..s.len() - 1].to_string()
        } else {
            s.to_string()
        }
    }
}
