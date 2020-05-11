use crate::chars::*;
use crate::err::ECode;
use crate::iter::LCChars;
use crate::ptrait::{ParseRes, Parser};

#[derive(Clone)]
pub struct Skip<CB: CharBool> {
    cb: CB,
    min: usize,
}

/// ```rust
/// use gobble::*;
/// let p = skip_while(|x|x == '$',0).ig_then(read_fs(is_alpha,1));
/// let s =p.parse_s("$$$$$$$hello").unwrap();
/// assert_eq!(s,"hello");
/// ```
pub fn skip_while<CB: CharBool>(cb: CB, min: usize) -> Skip<CB> {
    Skip { cb, min }
}

impl<CB: CharBool> Parser<()> for Skip<CB> {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, ()> {
        let mut i = i.clone();
        let mut i2 = i.clone();
        let mut ndone = 0;
        while let Some(c) = i.next() {
            match self.cb.char_bool(c) {
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
