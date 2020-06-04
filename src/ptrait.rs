use crate::err::{ECode, ParseError};
use crate::iter::LCChars;
use std::cmp::Ordering;
use std::marker::PhantomData;

pub type ParseRes<'a, V> = Result<(LCChars<'a>, V), ParseError>;

/// The core trait for parsing
pub trait Parser<V>: Sized {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, V>;
    fn parse_s(&self, s: &str) -> Result<V, ParseError> {
        self.parse(&LCChars::str(s)).map(|(_, v)| v)
    }
    fn parse_sn<'a>(&self, s: &'a str) -> Result<(&'a str, V), ParseError> {
        self.parse(&LCChars::str(s)).map(|(i, v)| (i.as_str(), v))
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
    /// Returns a Parser that converts the result of a successful parse to a different type.
    /// however the map function can fail and return a result
    /// The Error type should be err::ECode, this does not have line associated. That will
    /// be attacked by the TryMap object
    /// so this will pass that error up correctly
    fn try_map<F: Fn(V) -> Result<V2, ECode>, V2>(self, f: F) -> TryMap<Self, V, V2, F> {
        TryMap {
            a: self,
            f,
            phav: PhantomData,
            phb: PhantomData,
        }
    }
    fn asv<R: Clone>(self, r: R) -> As<Self, V, R> {
        As {
            a: self,
            r,
            pha: PhantomData,
        }
    }

    fn ig(self) -> As<Self, V, ()> {
        self.asv(())
    }

    fn map_err<F: Fn(ECode) -> ECode>(self, f: F) -> MapErr<Self, V, F> {
        MapErr {
            p: self,
            f,
            phv: PhantomData,
        }
    }

    fn brk(self) -> Break<Self, V> {
        Break {
            p: self,
            phv: PhantomData,
        }
    }
}

impl<V, F: for<'a> Fn(&LCChars<'a>) -> ParseRes<'a, V>> Parser<V> for F {
    fn parse<'b>(&self, i: &LCChars<'b>) -> ParseRes<'b, V> {
        self(i)
    }
}

impl Parser<&'static str> for &'static str {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, &'static str> {
        crate::reader::do_tag(i, self)
    }
}

impl Parser<char> for char {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, char> {
        let mut i2 = i.clone();
        match i2.next() {
            Some(c) if c == *self => Ok((i2, *self)),
            v => i2.err_cr(ECode::Char(*self, v)),
        }
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
        match self.a.parse(i) {
            Ok((r, v)) => Ok((r, v)),
            Err(e) if e.is_break() => Err(e),
            Err(e) => match self.b.parse(i) {
                Ok((r, v)) => Ok((r, v)),
                Err(e2) if e2.is_break() => Err(e2),
                Err(e2) => match e.partial_cmp(&e2) {
                    Some(Ordering::Equal) | None => i.err_cr(ECode::Or(Box::new(e), Box::new(e2))),
                    Some(Ordering::Less) => Err(e2),
                    Some(Ordering::Greater) => Err(e),
                },
            },
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

#[derive(Clone)]
pub struct TryMap<A: Parser<AV>, AV, B, F: Fn(AV) -> Result<B, ECode>> {
    a: A,
    f: F,
    phb: PhantomData<B>,
    phav: PhantomData<AV>,
}

impl<A: Parser<AV>, AV, B, F: Fn(AV) -> Result<B, ECode>> Parser<B> for TryMap<A, AV, B, F> {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, B> {
        let (ri, v) = self.a.parse(i)?;
        match (self.f)(v) {
            Ok(v2) => Ok((ri, v2)),
            Err(e) => ri.err_cr(e),
        }
    }
}

pub struct As<A: Parser<AV>, AV, R: Clone> {
    a: A,
    pha: PhantomData<AV>,
    r: R,
}
impl<A: Parser<AV>, AV, R: Clone> Parser<R> for As<A, AV, R> {
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, R> {
        let (ri, _) = self.a.parse(it)?;
        Ok((ri, self.r.clone()))
    }
}

pub struct MapErr<P: Parser<V>, V, F: Fn(ECode) -> ECode> {
    p: P,
    f: F,
    phv: PhantomData<V>,
}

impl<P: Parser<V>, V, F: Fn(ECode) -> ECode> Parser<V> for MapErr<P, V, F> {
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, V> {
        match self.p.parse(it) {
            Err(mut e) => {
                e.code = (self.f)(e.code);
                Err(e)
            }
            ov => ov,
        }
    }
}
pub struct Break<P: Parser<V>, V> {
    p: P,
    phv: PhantomData<V>,
}

impl<P: Parser<V>, V> Parser<V> for Break<P, V> {
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, V> {
        match self.p.parse(it) {
            Err(mut e) => {
                e.code = e.code.brk();
                Err(e)
            }
            ov => ov,
        }
    }
}
#[cfg(test)]
pub mod test {
    use super::*;
    use crate::common::*;
    #[test]
    fn test_strs_can_be_parsers() {
        let p = "(((".ig_then(common_int);
        let n = p.parse_s("(((32").unwrap();
        assert_eq!(n, 32);
    }
}
