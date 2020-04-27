use failure_derive::*;

#[derive(Debug, Clone, PartialEq, Fail)]
pub enum ECode {
    #[fail(display = "End of Input")]
    EOF,
    #[fail(display = "This Error Should Never Happen: {}", 0)]
    Never(&'static str),
    #[fail(display = "{}", 0)]
    SMess(&'static str),
    #[fail(display = "{}", 0)]
    Mess(String),
    #[fail(display = "{}::{}", 0, 1)]
    Wrap(&'static str, Box<ParseError>),
    #[fail(display = "Error {} or {}", 0, 1)]
    Or(Box<ParseError>, Box<ParseError>),
    #[fail(display = "Expected {}", 0)]
    Tag(&'static str),
    #[fail(display = "Unexpected {}", 0)]
    UnexpectedChar(char),
}

#[derive(Debug, Clone, PartialEq, Fail)]
#[fail(display = "Parse Error on line {} : {}", code, line)]
pub struct ParseError {
    pub code: ECode,
    pub line: u64,
    pub col: u64,
}

impl ParseError {
    pub fn new(s: &'static str, line: u64, col: u64) -> ParseError {
        ParseError {
            code: ECode::SMess(s),
            line,
            col,
        }
    }
    pub fn code(code: ECode, line: u64, col: u64) -> ParseError {
        ParseError { code, line, col }
    }
}
