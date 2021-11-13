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
        &mut self.cells[y * self.height + x]
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
        let chars = [' ', '▀', '▄', '█'];
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
}

type Constraint = Vec<usize>;

// * A parser for nonograms

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
        Ok(write!(
            f,
            "Parse error [line {}] {}",
            self.line, self.message
        )?)
    }
}

impl Parser {
    /// Parse a nonogram using the format of
    /// https://github.com/mikix/nonogram-db/blob/master/db/gnonograms/ubuntu.non
    pub fn parse(f: &mut impl Read) -> Result<Nono, ParserError> {
        let mut source: String = String::default();
        let mut parser = Parser::default();

        // Read input
        f.read_to_string(&mut source)
            .map_err(|_| parser.err("Cannot parse source as UTF-8."))?;

        for line in source.lines() {
            parser.line += 1;
            match parser.mode {
                ParserMode::Main => parser.parse_header_line(line)?,
                ParserMode::Cols => parser.parse_constraint_line(line)?,
                ParserMode::Rows => parser.parse_constraint_line(line)?,
            }
        }

        let err = parser.err("Empty/invalid file.");
        Ok(parser.nono.ok_or(err)?)
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

/// Remove surrounding quotes from a string.
fn unquote(s: &str) -> String {
    if s.starts_with("\"") && s.ends_with("\"") {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[derive(Clone, Debug)]
pub enum CellState {
    Undecided,
    Empty,
    Filled,
}
