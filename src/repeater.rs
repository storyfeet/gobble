use crate::err::*;
use crate::iter::LCChars;
use crate::ptrait::*;
use std::marker::PhantomData;

pub struct Separated<A: Parser<AV>, B: Parser<BV>, AV, BV> {
    a: A,
    b: B,
    pha: PhantomData<AV>,
    phb: PhantomData<BV>,
    min_one: bool,
}

impl<A, B, AV, BV> Parser<Vec<AV>> for Separated<A, B, AV, BV>
where
    A: Parser<AV>,
    B: Parser<BV>,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Vec<AV>> {
        let mut res = Vec::new();
        let mut ri = match self.a.parse(i) {
            Ok((r, v)) => {
                res.push(v);
                r
            }
            Err(e) => {
                if !self.min_one {
                    return Ok((i.clone(), res));
                } else {
                    return i.err_cr(ECode::Wrap("No contents", Box::new(e)));
                }
            }
        };
        loop {
            //try sep if not found, return
            ri = match self.b.parse(&ri) {
                Ok((r, _)) => r,
                Err(_) => return Ok((ri, res)),
            };

            ri = match self.a.parse(&ri) {
                Ok((r, v)) => {
                    res.push(v);
                    r
                }
                Err(e) => {
                    return i.err_cr(ECode::Wrap("Nothing after sep", Box::new(e)));
                }
            };
        }
    }
}

pub fn sep<A: Parser<AV>, B: Parser<BV>, AV, BV>(
    a: A,
    b: B,
    min_one: bool,
) -> Separated<A, B, AV, BV> {
    Separated {
        a,
        b,
        min_one,
        pha: PhantomData,
        phb: PhantomData,
    }
}

pub struct Repeater<A, AV> {
    a: A,
    pha: PhantomData<AV>,
    min: usize,
}

impl<A: Parser<AV>, AV> Parser<Vec<AV>> for Repeater<A, AV> {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Vec<AV>> {
        let mut ri = i.clone();
        let mut res = Vec::new();
        loop {
            ri = match self.a.parse(&ri) {
                Ok((r, v)) => {
                    res.push(v);
                    r
                }
                Err(e) => {
                    if res.len() < self.min {
                        return i.err_cr(ECode::Wrap("not enough elems", Box::new(e)));
                    } else {
                        return Ok((ri, res));
                    }
                }
            }
        }
    }
}

pub fn repeat<A: Parser<AV>, AV>(a: A, min: usize) -> Repeater<A, AV> {
    Repeater {
        a,
        min,
        pha: PhantomData,
    }
}
