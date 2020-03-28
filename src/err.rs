#[derive(Debug)]
pub struct ParseError {
    mess: &'static str,
    line: u64,
}

impl ParseError {
    pub fn new(s: &'static str, line: u64) -> ParseError {
        ParseError { mess: s, line }
    }
}
