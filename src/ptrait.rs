use crate::err::ParseError;
use crate::iter::LCChars;
use std::marker::PhantomData;

pub type ParseRes<'a, V> = Result<(LCChars<'a>, V), ParseError>;

/// The core trait for parsing
pub trait Parser<V>: Sized {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, V>;
    fn parse_s(&self, s: &str) -> Result<V, ParseError> {
        self.parse(&LCChars::str(s)).map(|(_, v)| v)
    }
    /// returns a parser that will combine the results of this and the given parser
    /// into a tuple
    fn then<P: Parser<V2>, V2>(self, p: P) -> Then<Self, P> {
        Then { one: self, two: p }
    }

    /// returns a Parser that will require the given parser completes, but ignores its result
    /// useful for dropping brackets and whitespace
    fn then_ig<P: Parser<V2>, V2>(self, p: P) -> ThenIg<Self, P, V, V2> {
        ThenIg {
            one: self,
            two: p,
            pha: PhantomData,
            phb: PhantomData,
        }
    }
    /// returns a Parser that will require this parser completes, but only return the
    /// result of the given parser
    /// useful for dropping brackets and whitespace etc
    fn ig_then<P: Parser<V2>, V2>(self, p: P) -> IgThen<Self, P, V, V2> {
        IgThen {
            one: self,
            two: p,
            pha: PhantomData,
            phb: PhantomData,
        }
    }
    /// Returns a Parser that will try both child parsers, (A first) and return the first successfl
    /// result
    fn or<P: Parser<V>>(self, p: P) -> Or<Self, P> {
        Or { a: self, b: p }
    }

    /// Returns a Parser that converts the result of a successful parse to a different type.
    /// Much like map on iterators and Result
    fn map<F: Fn(V) -> V2, V2>(self, f: F) -> Map<Self, V, V2, F> {
        Map {
            a: self,
            f,
            phav: PhantomData,
            phb: PhantomData,
        }
    }
}

impl<V, F: for<'a> Fn(&LCChars<'a>) -> ParseRes<'a, V>> Parser<V> for F {
    fn parse<'b>(&self, i: &LCChars<'b>) -> ParseRes<'b, V> {
        self(i)
    }
}

impl<F> Parser<()> for Take<F>
where
    F: Fn(char) -> bool,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, ()> {
        let mut n = 0;
        let mut i = i.clone();
        let mut i2 = i.clone();
        while let Some(c) = i.next() {
            if !(self.f)(c) {
                if n < self.min {
                    return i.err_r("not enough to take");
                }
                return Ok((i2, ()));
            }
            n += 1;
            i2.next();
        }
        if n < self.min {
            return i.err_r("End of str before end of take");
        }
        Ok((i2, ()))
    }
}

#[derive(Clone)]
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
#[derive(Clone)]
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

#[derive(Clone)]
pub struct ThenIg<A, B, AV, BV> {
    one: A,
    two: B,
    pha: PhantomData<AV>,
    phb: PhantomData<BV>,
}

impl<A, B, AV, BV> Parser<AV> for ThenIg<A, B, AV, BV>
where
    A: Parser<AV>,
    B: Parser<BV>,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, AV> {
        let (i, v1) = self.one.parse(i)?;
        let (i, _) = self.two.parse(&i)?;
        Ok((i, v1))
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Map<A: Parser<AV>, AV, B, F: Fn(AV) -> B> {
    a: A,
    f: F,
    phb: PhantomData<B>,
    phav: PhantomData<AV>,
}

impl<A: Parser<AV>, AV, B, F: Fn(AV) -> B> Parser<B> for Map<A, AV, B, F> {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, B> {
        let (ri, v) = self.a.parse(i)?;
        Ok((ri, (self.f)(v)))
    }
}
