use failure_derive::*;
#[derive(Debug, Clone, PartialEq, Fail)]
#[fail(display = "Parse Error on line {} : {}", mess, line)]
pub struct ParseError {
    mess: &'static str,
    line: u64,
}

impl ParseError {
    pub fn new(s: &'static str, line: u64) -> ParseError {
        ParseError { mess: s, line }
    }
}
