use std::io;
use std::fmt;

use AnalysisError::*;

pub enum AnalysisError {
    IOError(io::Error),
    ParseError,
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IOError(io_error) => io_error.fmt(f),
            ParseError => write!(f, "failed to parse"),
        }
    }
}

impl From<io::Error> for AnalysisError {
    fn from(value: io::Error) -> Self {
        IOError(value)
    }
}