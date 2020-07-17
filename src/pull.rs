use crate::err::StrError;
use crate::iter::LCChars;
use crate::ptrait::*;

pub struct PullParser<'a, P: Parser, E: Parser> {
    pub p: P,
    pub s: &'a str,
    pub it: LCChars<'a>,
    pub end: E,
}

impl<'a, P: Parser, E: Parser> Iterator for PullParser<'a, P, E> {
    type Item = Result<P::Out, StrError<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.p.parse(&self.it) {
            Ok((it2, r, _)) => {
                self.it = it2;
                Some(Ok(r))
            }
            Err(e) => match self.end.parse(&self.it) {
                Ok(_) => None,
                Err(_) => Some(Err(e.on_str(self.s))),
            },
        }
    }
}
