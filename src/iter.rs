use crate::err::{ECode, ParseError};
use std::str::Chars;

#[derive(Clone, Debug)]
pub struct LCChars<'a> {
    iter: Chars<'a>,
    l: usize,
    c: usize,
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

    pub fn as_str(&self) -> &'a str {
        self.iter.as_str()
    }

    pub fn err(&self, s: &'static str) -> ParseError {
        //println!("err {} {} ", self.l, self.c);
        ParseError::new(s, self.l, self.c)
    }
    pub fn err_r<V>(&self, s: &'static str) -> Result<V, ParseError> {
        //println!("err_r {} {} ", self.l, self.c);
        Err(self.err(s))
    }

    pub fn err_c(&self, c: ECode) -> ParseError {
        //println!("err_c {} {} ", self.l, self.c);
        ParseError::code(c, self.l, self.c)
    }
    pub fn err_cr<V>(&self, c: ECode) -> Result<V, ParseError> {
        //println!("err_cr {} {} ", self.l, self.c);
        Err(ParseError::code(c, self.l, self.c))
    }

    pub fn lc(&self) -> (usize, usize) {
        (self.l, self.c)
    }
}

impl<'a> Iterator for LCChars<'a> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        //println!("lc {} {} ", self.l, self.c);
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
