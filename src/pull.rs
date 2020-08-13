use crate::err::*;
use crate::iter::LCChars;
use crate::ptrait::*;
use crate::reader::EOI;

pub struct PullParser<'a, P: Parser, E: Parser> {
    p: P,
    pub s: &'a str,
    it: LCChars<'a>,
    end: E,
    errored: bool,
}

impl<'a, P: Parser> PullParser<'a, P, EOI> {
    pub fn new(p: P, s: &'a str) -> Self {
        PullParser::with_end(p, EOI, s)
    }
}
impl<'a, P: Parser, E: Parser> PullParser<'a, P, E> {
    pub fn with_end(p: P, end: E, s: &'a str) -> Self {
        PullParser {
            p,
            end,
            s,
            it: LCChars::str(s),
            errored: false,
        }
    }
}

impl<'a, P: Parser, E: Parser> Iterator for PullParser<'a, P, E> {
    type Item = Result<P::Out, PErr<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }
        match self.p.parse(&self.it) {
            Ok((it2, r, _)) => {
                self.it = it2;
                Some(Ok(r))
            }
            Err(e) => match self.end.parse(&self.it) {
                Ok(_) => None,
                Err(_) => {
                    self.errored = true;
                    Some(Err(e))
                }
            },
        }
    }
}
