use crate::err::{ECode, ParseError};
use std::str::Chars;

#[derive(Clone, Debug)]
pub struct LCChars<'a> {
    iter: Chars<'a>,
    l: u64,
    c: u64,
}

impl<'a> LCChars<'a> {
    pub fn str(s: &'a str) -> Self {
        LCChars {
            iter: s.chars(),
            l: 0,
            c: 0,
        }
    }

    pub fn from_chars(iter: Chars<'a>) -> LCChars<'a> {
        LCChars { iter, l: 0, c: 0 }
    }

    pub fn err(&self, s: &'static str) -> ParseError {
        ParseError::new(s, self.l, self.c)
    }
    pub fn err_r<V>(&self, s: &'static str) -> Result<V, ParseError> {
        Err(self.err(s))
    }

    pub fn err_c(&self, c: ECode) -> ParseError {
        ParseError::code(c, self.l, self.c)
    }
    pub fn err_cr(&self, c: ECode) -> ParseError {
        ParseError::code(c, self.l, self.c)
    }
}

impl<'a> Iterator for LCChars<'a> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        match self.iter.next() {
            Some('\n') => {
                self.l += 1;
                self.c = 0;
                Some('\n')
            }
            Some(v) => {
                self.c += 1;
                Some(v)
            }
            None => None,
        }
    }
}
