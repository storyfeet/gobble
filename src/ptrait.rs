use crate::err::ParseError;
use crate::iter::LCChars;
use std::marker::PhantomData;

pub type ParseRes<'a, V> = Result<(LCChars<'a>, V), ParseError>;

pub trait Parser<V>: Sized {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, V>;
    fn parse_s(&self, s: &str) -> Result<V, ParseError> {
        self.parse(LCChars::str(s)).map(|(i, v)| v)
    }
    fn then<P: Parser<V2>, V2>(self, p: P) -> Then<Self, P> {
        Then { one: self, two: p }
    }
    fn then_ig<P: Parser<V2>, V2>(self, p: P) -> ThenIg<Self, P, V, V2> {
        ThenIg {
            one: self,
            two: p,
            pha: PhantomData,
            phb: PhantomData,
        }
    }
    fn ig_then<P: Parser<V2>, V2>(self, p: P) -> IgThen<Self, P, V, V2> {
        IgThen { one: self, two: p }
    }
    fn or<P: Parser<V>>(self, p: P) -> Or<Self, P> {
        Or {
            a: self,
            b: p,
            phi: PhantomData,
        }
    }
}

impl<'a, V, F: Fn(&LCChars<'a>) -> ParseRes<'a, V>> Parser<V> for F {
    fn parse<'b>(&self, i: &LCChars<'b>) -> ParseRes<'b, V> {
        self(i)
    }
}

impl<F> Parser<F> for Take<F>
where
    F: Fn(char) -> bool,
{
    fn parse<'a>(&self, i: &LCChars) -> ParseRes<'a, ()> {
        let mut n = 0;
        let mut i = i.clone();
        let mut i2 = i.clone();
        while let Some(c) = i.next() {
            if !(self.f)(c) {
                if n < self.min {
                    return Err(ParseError::new("not enough to take", 0));
                }
                return Ok((i2, ()));
            }
            n += 1;
            i2.next();
        }
        if n < self.min {
            return Err(i.err("End of str before end of take", 0));
        }
        Ok((i2, ()))
    }
}

pub struct Take<F> {
    f: F,
    min: usize,
}

pub fn take<F>(f: F, min: usize) -> Take<F>
where
    F: Fn(char) -> bool,
{
    Take { f, min }
}

pub fn ws(min: usize) -> Take<impl Fn(char) -> bool> {
    Take {
        f: |c| match c {
            ' ' | '\t' | '\r' => true,
            _ => false,
        },
        min,
    }
}
pub struct Then<A, B> {
    one: A,
    two: B,
}

impl<A, B, AV, BV> Parser<(AV, BV)> for Then<A, B>
where
    A: Parser<AV>,
    B: Parser<BV>,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, (AV, BV)> {
        let (i, v1) = self.one.parse(i)?;
        let (i, v2) = self.two.parse(&i)?;
        Ok((i, (v1, v2)))
    }
}

pub struct ThenIg<A, B, AV, BV> {
    one: A,
    two: B,
    pha: PhantomData<AV>,
    phb: PhantomData<BV>,
}

impl<A, B, AV, BV> Parser<BV> for ThenIg<A, B, AV, BV>
where
    A: Parser<AV>,
    B: Parser<BV>,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<AV> {
        let (i, v1) = self.one.parse(i)?;
        let (i, _) = self.two.parse(&i)?;
        Ok((i, v1))
    }
}

pub struct IgThen<A, B, AV, BV> {
    one: A,
    two: B,
    pha: PhantomData<AV>,
    phb: PhantomData<BV>,
}

impl<A, B, AV, BV> Parser<BV> for IgThen<A, B, AV, BV>
where
    A: Parser<AV>,
    B: Parser<BV>,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, BV> {
        let (i, _) = self.one.parse(i)?;
        let (i, v2) = self.two.parse(&i)?;
        Ok((i, v2))
    }
}

pub struct Or<A, B> {
    a: A,
    b: B,
}

impl<A, B, V> Parser<V> for Or<A, B>
where
    A: Parser<V>,
    B: Parser<V>,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, V> {
        if let Ok((r, v)) = self.a.parse(i) {
            Ok((r, v))
        } else {
            self.b.parse(i)
        }
    }
}
