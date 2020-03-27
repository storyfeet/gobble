
#[derive(Debug)]
pub struct ParseError {
    mess: String,
    line: u64,
}

impl ParseError {
    pub fn new(s: &str, line: u64) -> ParseError {
        ParseError {
            mess: s.to_string(),
            line,
        }
    }
}
