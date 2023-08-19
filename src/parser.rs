use crate::{Nonogram,NonogramBuilder,BuilderError};

use std::io;
use std::fmt::Display;
use std::fmt;
use std::io::Read;
use std::num::ParseIntError;

#[derive(Default)]
pub struct Parser {
    builder: NonogramBuilder,
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
pub enum ParserError {
    InternalError,
    ParseIntError,
    IOError(io::Error),
    BuilderError(BuilderError)
}

impl From<io::Error> for ParserError {
    fn from(value: io::Error) -> Self {
        ParserError::IOError(value)
    }
}

impl From<BuilderError> for ParserError {
    fn from(value: BuilderError) -> Self {
        ParserError::BuilderError(value)
    }
}

impl From<ParseIntError> for ParserError {
    fn from(_: ParseIntError) -> Self {
        Self::ParseIntError
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(write!(f, "Parse error TBD")?) //@FIXME
    }
}

impl Parser {
    pub fn new() -> Parser {
        Parser::default()
    }

    /// Parse a nonogram using the format of
    /// <https://github.com/mikix/nonogram-db/>
    pub fn parse(mut self, f: &mut impl Read) -> Result<Nonogram, ParserError> {
        let mut source: String = String::default();

        // Read input
        f.read_to_string(&mut source)
            .map_err(|e| ParserError::IOError(e))?;

        for line in source.lines() {
            self.line += 1;
            match self.mode {
                ParserMode::Main => self.parse_header_line(line)?,
                ParserMode::Cols => self.parse_constraint_line(line)?,
                ParserMode::Rows => self.parse_constraint_line(line)?,
            }
        }

        Ok(self.builder.build()?)
    }

    fn parse_header_line(&mut self, line: &str) -> Result<(), ParserError> {
        let (command, args) = line.split_at(line.find(' ').unwrap_or(line.len()));
        match command {
            "columns" => self.mode = ParserMode::Cols,
            "rows" => self.mode = ParserMode::Rows,
            "height" => {
                self.builder.height(
                    args.trim()
                        .parse::<usize>()?)?;
            },
            "width" => {
                self.builder.width(
                    args.trim()
                        .parse::<usize>()?)?;
            }
            "goal" => {
                println!("Skipping goal for now");
                // self.ensure_nono()?;
                // self.nono.as_mut().unwrap().cells = unquote(args.trim())
                //     .chars()
                //     .map(|c| match c {
                //         '0' => Ok(CellState::Empty),
                //         '1' => Ok(CellState::Filled),
                //         _ => Err(self.err("Cannot parse goal")),
                //     })
                //     .collect::<Result<Vec<CellState>, ParserError>>()?;
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
            match &self.mode {
                ParserMode::Rows => self.builder.push_row_constraint(vec),
                ParserMode::Cols => self.builder.push_col_constraint(vec),
                ParserMode::Main => return Err(ParserError::InternalError),
            };
        } else {
            self.mode = ParserMode::Main;
            // Because there may not be a blank line after the last column or row.
            return self.parse_header_line(line);
        };
        Ok(())
    }
}

// ** Parser utilities

/// Remove surrounding quotes from a strin.
fn unquote(s: &str) -> String {
    if s.starts_with("\"") && s.ends_with("\"") {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}
