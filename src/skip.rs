use crate::chars::*;
use crate::err::ECode;
use crate::iter::LCChars;
use crate::ptrait::{ParseRes, Parser};
use std::marker::PhantomData;

pub struct Skip<CB: CharBool> {
    cb: CB,
}

impl<CB: CharBool> Parser<()> for Skip<CB> {
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, ()> {
        let mut it = it.clone();
        loop {
            let it2 = it.clone();
            match it.next() {
                Some(c) if self.cb.char_bool(c) => {}
                Some(_) | None => return Ok((it2, ())),
            }
        }
    }
}

pub fn skip_c<CB: CharBool>(cb: CB) -> Skip<CB> {
    Skip { cb }
}

#[derive(Clone)]
pub struct SkipMin<CB: CharBool> {
    cb: CB,
    min: usize,
}

/// ```rust
/// use gobble::*;
/// let p = skip_while(|x|x == '$',0).ig_then(read_fs(is_alpha,1));
/// let s =p.parse_s("$$$$$$$hello").unwrap();
/// assert_eq!(s,"hello");
/// ```
pub fn skip_while<CB: CharBool>(cb: CB, min: usize) -> SkipMin<CB> {
    SkipMin { cb, min }
}

impl<CB: CharBool> Parser<()> for SkipMin<CB> {
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

pub struct SkipRepeat<A: Parser<AV>, AV> {
    a: A,
    pha: PhantomData<AV>,
    min: usize,
}
impl<A: Parser<AV>, AV> Parser<()> for SkipRepeat<A, AV> {
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, ()> {
        let mut done = 0;
        let mut it = it.clone();
        loop {
            match self.a.parse(&it) {
                Ok((ri, _)) => {
                    done += 1;
                    it = ri;
                }
                Err(e) => {
                    if done < self.min {
                        return Err(e);
                    }
                    return Ok((it, ()));
                }
            }
        }
    }
}

pub fn skip_repeat<A: Parser<AV>, AV>(a: A, min: usize) -> SkipRepeat<A, AV> {
    SkipRepeat {
        a,
        min,
        pha: PhantomData,
    }
}
