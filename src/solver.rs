use crate::{CellState,Constraint,Nonogram};
use std::iter::once;

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

// @FIXME Convert into newtype.
type CandidateMask = Vec<CellState>;
type CandidateMaskSet = Vec<CandidateMask>;

pub struct Solver<'a> {
    rows: Vec<CandidateMaskSet>,
    cols: Vec<CandidateMaskSet>,
    nono: &'a mut Nonogram,
}
impl<'a> Solver<'a> {
    pub fn new(from: &'a mut Nonogram) -> Solver<'a> {
        Solver {
            rows: from
                .rows
                .iter()
                .map(|r| candidates(r, from.width()))
                .collect(),
            cols: from
                .cols
                .iter()
                .map(|r| candidates(r, from.height()))
                .collect(),
            nono: from,
        }
    }

    pub fn solve(&mut self) {
        // TODO Prepare
        self.nono.clear_solution();
        while self.nono.cells.iter().any(|c| *c == CellState::Undecided) {
            self.consensus_step();
            self.filter_step();
        }
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
            for (x, square) in consensus.iter().enumerate() {
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
        fn filter_step(&mut self) {
            // Rows
            for y in 0..self.nono.height() {
                let grid_row = self.nono.row(y).unwrap();
                self.rows[y].retain(|cand| can_place(&grid_row, cand));
            }
            // Cols
            for x in 0..self.nono.width() {
                let grid_col = self.nono.column(x).unwrap();
                self.cols[x].retain(|cand| can_place(&grid_col, cand));
            }
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

    /// Compare a row or column of the grid with a candidate, and
    /// return true if this candidate would fit this row or column,
    /// that is, if there are no incompatible Filled/Empty cells
    /// between the grid and the candidate.
    fn can_place(grid: &[CellState], cand: &[CellState]) -> bool {
        grid.iter().zip(cand).all(|(g, c)| g.accepts(c))
        // println!(
        //     "{} over {}? {:?}",
        //     mask_as_string(&cand),
        //     mask_as_string(&grid),
        //     ret
        // );
        // ret
    }
