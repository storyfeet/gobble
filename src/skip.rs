use crate::err::ECode;
use crate::iter::LCChars;
use crate::ptrait::{ParseRes, Parser};

#[derive(Clone)]
pub struct Skip<F> {
    f: F,
    min: usize,
}

pub fn skip_while<F: Fn(char) -> bool>(f: F, min: usize) -> Skip<F> {
    Skip { f, min }
}

impl<F> Parser<()> for Skip<F>
where
    F: Fn(char) -> bool,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, ()> {
        let mut i = i.clone();
        let mut i2 = i.clone();
        let mut ndone = 0;
        while let Some(c) = i.next() {
            match (self.f)(c) {
                true => {
                    i2 = i.clone();
                    ndone += 1;
                }
                false => {
                    return if ndone >= self.min {
                        Ok((i2, ()))
                    } else {
                        i.err_cr(ECode::UnexpectedChar(c))
                    }
                }
            }
        }
        if ndone < self.min {
            return i.err_cr(ECode::EOF);
        }
        Ok((i, ()))
    }
}
