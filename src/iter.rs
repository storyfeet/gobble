use crate::err::{Expectable, Expected, ParseError};
use crate::ptrait::{ParseRes, Parser};
use std::str::{CharIndices, Chars};

#[derive(Clone, Debug)]
pub struct LCChars<'a> {
    iter: CharIndices<'a>,
    l: usize,
    c: usize,
}

impl<'a> LCChars<'a> {
    pub fn str(s: &'a str) -> Self {
        LCChars {
            iter: s.char_indices(),
            l: 0,
            c: 0,
        }
    }

    #[deprecated(since = "0.2.1", note = "use from_char_indices instead")]
    pub fn from_chars(iter: Chars<'a>) -> LCChars<'a> {
        LCChars::str(iter.as_str())
    }

    pub fn from_char_indices(iter: CharIndices<'a>) -> LCChars<'a> {
        LCChars { iter, l: 0, c: 0 }
    }

    pub fn as_str(&self) -> &'a str {
        self.iter.as_str()
    }

    pub fn err(&self, s: &'static str) -> ParseError<Expected> {
        //println!("err {} {} ", self.l, self.c);
        ParseError::new(s, self.index(), self.l, self.c)
    }
    pub fn err_r<V>(&self, s: &'static str) -> Result<V, ParseError<Expected>> {
        //println!("err_r {} {} ", self.l, self.c);
        Err(self.err(s))
    }

    pub fn err_px<P: Parser>(&self, p: &P) -> ParseError<P::Ex> {
        ParseError::expect(p.expected(), self.index(), self.l, self.c)
    }
    pub fn err_pxr<P: Parser, V>(&self, p: &P) -> Result<V, ParseError<P::Ex>> {
        Err(ParseError::expect(
            p.expected(),
            self.index(),
            self.l,
            self.c,
        ))
    }

    pub fn err_ex<E: Expectable>(&self, e: E) -> ParseError<E> {
        ParseError::expect(e, self.index(), self.l, self.c)
    }
    pub fn err_ex_r<V, E: Expectable>(&self, e: E) -> Result<V, ParseError<E>> {
        //println!("err_cr {} {} ", self.l, self.c);
        Err(ParseError::expect(e, self.index(), self.l, self.c))
    }

    pub fn lc(&self) -> (usize, usize) {
        (self.l, self.c)
    }
    pub fn index(&self) -> Option<usize> {
        self.iter.clone().next().map(|(i, _)| i)
    }
}

impl<'a> Iterator for LCChars<'a> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        //println!("lc {} {} ", self.l, self.c);
        match self.iter.next() {
            Some((_, '\n')) => {
                self.l += 1;
                self.c = 0;
                Some('\n')
            }
            Some((_, v)) => {
                self.c += 1;
                Some(v)
            }
            None => None,
        }
    }
}

pub fn index<'a>(it: &LCChars<'a>) -> ParseRes<'a, Option<usize>> {
    return Ok((it.clone(), it.index()));
}

pub fn line_col<'a>(it: &LCChars<'a>) -> ParseRes<'a, (usize, usize)> {
    return Ok((it.clone(), (it.l, it.c)));
}
