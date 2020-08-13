use crate::chars::CharBool;
use crate::err::{Expected, PErr};
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

    pub fn err(&self, exp: Expected) -> PErr<'a> {
        let found = self.iter.as_str();
        let flen = found.len().min(10);
        PErr {
            exp,
            found: &found[..flen],
            index: self.index(),
            line: self.l,
            col: self.c,
            is_brk: false,
            child: None,
        }
    }

    pub fn err_s(&self, s: &'static str) -> PErr<'a> {
        self.err(Expected::Str(s))
    }

    pub fn err_rs<V>(&self, s: &'static str) -> Result<V, PErr<'a>> {
        Err(self.err_s(s))
    }

    pub fn err_p<P: Parser>(&self, p: &P) -> PErr<'a> {
        self.err(p.expected())
    }
    pub fn err_rp<P: Parser, V>(&self, p: &P) -> Result<V, PErr<'a>> {
        Err(self.err_p(p))
    }
    pub fn err_op<P: Parser>(&self, p: &P) -> Option<PErr<'a>> {
        Some(self.err_p(p))
    }

    pub fn err_oc<C: CharBool>(&self, c: &C) -> Option<PErr<'a>> {
        Some(self.err(c.expected()))
    }

    pub fn err_r<V>(&self, e: Expected) -> Result<V, PErr<'a>> {
        Err(self.err(e))
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
    return Ok((it.clone(), it.index(), None));
}

pub fn line_col<'a>(it: &LCChars<'a>) -> ParseRes<'a, (usize, usize)> {
    return Ok((it.clone(), (it.l, it.c), None));
}
