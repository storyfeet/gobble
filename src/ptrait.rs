use crate::err::{Expectable, Expected, ParseError};
use crate::iter::LCChars;
use std::cmp::Ordering;

pub type ParseERes<'a, V, E> = Result<(LCChars<'a>, V), ParseError<E>>;

pub type ParseRes<'a, V> = ParseERes<'a, V, Expected>;

/// The core trait for parsing
pub trait Parser: Sized {
    type Out;
    type Ex: Expectable;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseERes<'a, Self::Out, Self::Ex>;
    fn expected(&self) -> Self::Ex {
        Self::Ex::default()
    }
    fn parse_s(&self, s: &str) -> Result<Self::Out, ParseError<Self::Ex>> {
        self.parse(&LCChars::str(s)).map(|(_, v)| v)
    }
    fn parse_sn<'a>(&self, s: &'a str) -> Result<(&'a str, Self::Out), ParseError<Self::Ex>> {
        self.parse(&LCChars::str(s)).map(|(i, v)| (i.as_str(), v))
    }
    /// returns a parser that will combine the results of this and the given parser
    /// into a tuple
    fn then<P: Parser<Out = V2>, V2>(self, b: P) -> Then<Self, P> {
        Then { a: self, b }
    }

    /// returns a Parser that will require the given parser completes, but ignores its result
    /// useful for dropping brackets and whitespace
    fn then_ig<P: Parser<Out = V2>, V2>(self, b: P) -> ThenIg<Self, P> {
        ThenIg { a: self, b }
    }
    /// returns a Parser that will require this parser completes, but only return the
    /// result of the given parser
    /// useful for dropping brackets and whitespace etc
    fn ig_then<P: Parser<Out = V2>, V2>(self, b: P) -> IgThen<Self, P> {
        IgThen { a: self, b }
    }
    /// Returns a Parser that will try both child parsers, (A first) and return the first successfl
    /// result
    fn or<P: Parser<Out = Self::Out>>(self, p: P) -> Or<Self, P> {
        Or { a: self, b: p }
    }

    /// Returns a Parser that converts the result of a successful parse to a different type.
    /// Much like map on iterators and Result
    fn map<F: Fn(Self::Out) -> V2, V2>(self, f: F) -> Map<Self, V2, F> {
        Map { a: self, f }
    }

    /// Returns a Parser that converts the result of a successful parse to a different type.
    /// however the map function can fail and return a result
    /// The Error type should be err::ECode, this does not have line associated. That will
    /// be attacked by the TryMap object
    /// so this will pass that error up correctly
    fn try_map<F: Fn(Self::Out) -> Result<V2, Self::Ex>, V2>(self, f: F) -> TryMap<Self, V2, F> {
        TryMap { a: self, f }
    }

    fn asv<R: Clone>(self, r: R) -> As<Self, R> {
        As { a: self, r }
    }

    fn ig(self) -> As<Self, ()> {
        self.asv(())
    }

    fn map_exp<NEX: Expectable, F: Fn(Self::Ex) -> NEX>(self, f: F) -> MapExp<Self, NEX, F> {
        MapExp { p: self, f }
    }

    fn brk(self) -> Break<Self> {
        Break { p: self }
    }
}

impl<V, EX: Expectable, F: for<'a> Fn(&LCChars<'a>) -> ParseERes<'a, V, EX>> Parser for F {
    type Out = V;
    type Ex = EX;
    fn parse<'b>(&self, i: &LCChars<'b>) -> ParseERes<'b, V, EX> {
        self(i)
    }
}

impl Parser for &'static str {
    type Out = &'static str;
    type Ex = Expected;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, &'static str> {
        crate::reader::do_tag(i, self)
    }
}

impl Parser for char {
    type Out = char;
    type Ex = Expected;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, char> {
        let mut i2 = i.clone();
        match i2.next() {
            Some(c) if c == *self => Ok((i2, *self)),
            v => i2.err_pxr(self),
        }
    }
    fn expected(&self) -> Self::Ex {
        Expected::Char(*self)
    }
}

#[derive(Clone)]
pub struct Then<A, B> {
    a: A,
    b: B,
}

impl<A, B, E: Expectable> Parser for Then<A, B>
where
    A: Parser<Ex = E>,
    B: Parser<Ex = E>,
{
    type Out = (A::Out, B::Out);
    type Ex = E;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseERes<'a, Self::Out, E> {
        let (i, v1) = self.a.parse(i)?;
        let (i, v2) = self.b.parse(&i)?;
        Ok((i, (v1, v2)))
    }
    fn expected(&self) -> Self::Ex {
        Expectable::first(self.a.expected(), self.b.expected())
    }
}

#[derive(Clone)]
pub struct ThenIg<A, B> {
    a: A,
    b: B,
}

impl<A, B, E: Expectable> Parser for ThenIg<A, B>
where
    A: Parser<Ex = E>,
    B: Parser<Ex = E>,
{
    type Out = A::Out;
    type Ex = E;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseERes<'a, Self::Out, E> {
        let (i, v1) = self.a.parse(i)?;
        let (i, _) = self.b.parse(&i)?;
        Ok((i, v1))
    }
    fn expected(&self) -> Self::Ex {
        Expectable::first(self.a.expected(), self.b.expected())
    }
}

#[derive(Clone)]
pub struct IgThen<A, B> {
    a: A,
    b: B,
}

impl<A, B, E: Expectable> Parser for IgThen<A, B>
where
    A: Parser<Ex = E>,
    B: Parser<Ex = E>,
{
    type Out = B::Out;
    type Ex = E;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseERes<'a, Self::Out, E> {
        let (i, _) = self.a.parse(i)?;
        let (i, v2) = self.b.parse(&i)?;
        Ok((i, v2))
    }
    fn expected(&self) -> Self::Ex {
        Expectable::first(self.a.expected(), self.b.expected())
    }
}

#[derive(Clone)]
pub struct Or<A, B> {
    a: A,
    b: B,
}

impl<A, B, V, E: Expectable> Parser for Or<A, B>
where
    A: Parser<Out = V, Ex = E>,
    B: Parser<Out = V, Ex = E>,
{
    type Out = V;
    type Ex = E;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseERes<'a, V, E> {
        match self.a.parse(i) {
            Ok((r, v)) => Ok((r, v)),
            Err(e) if e.is_break() => Err(e),
            Err(e) => match self.b.parse(i) {
                Ok((r, v)) => Ok((r, v)),
                Err(e2) if e2.is_break() => Err(e2),
                Err(e2) => match e.partial_cmp(&e2) {
                    Some(Ordering::Equal) | None => i.err_pxr(self),
                    Some(Ordering::Less) => Err(e2),
                    Some(Ordering::Greater) => Err(e),
                },
            },
        }
    }
    fn expected(&self) -> E {
        self.a.expected().or(self.b.expected())
    }
}

#[derive(Clone)]
pub struct Map<A: Parser, B, F: Fn(A::Out) -> B> {
    a: A,
    f: F,
}

impl<A: Parser<Out = AV>, AV, B, F: Fn(A::Out) -> B> Parser for Map<A, B, F> {
    type Out = B;
    type Ex = A::Ex;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseERes<'a, B, Self::Ex> {
        let (ri, v) = self.a.parse(i)?;
        Ok((ri, (self.f)(v)))
    }
    fn expected(&self) -> Self::Ex {
        self.a.expected()
    }
}

#[derive(Clone)]
pub struct TryMap<A: Parser, B, F: Fn(A::Out) -> Result<B, A::Ex>> {
    a: A,
    f: F,
}

impl<A: Parser, B, F: Fn(A::Out) -> Result<B, A::Ex>> Parser for TryMap<A, B, F> {
    type Out = B;
    type Ex = A::Ex;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseERes<'a, B, A::Ex> {
        let (ri, v) = self.a.parse(i)?;
        match (self.f)(v) {
            Ok(v2) => Ok((ri, v2)),
            Err(e) => ri.err_ex_r(e),
        }
    }
    fn expected(&self) -> Self::Ex {
        self.a.expected()
    }
}

pub struct As<A: Parser, R: Clone> {
    a: A,
    r: R,
}
impl<A: Parser, R: Clone> Parser for As<A, R> {
    type Out = R;
    type Ex = A::Ex;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseERes<'a, R, Self::Ex> {
        let (ri, _) = self.a.parse(it)?;
        Ok((ri, self.r.clone()))
    }
    fn expected(&self) -> Self::Ex {
        self.a.expected()
    }
}

pub struct MapExp<P: Parser, NEX, F: Fn(P::Ex) -> NEX> {
    p: P,
    f: F,
}

impl<P: Parser, NEX: Expectable, F: Fn(P::Ex) -> NEX> Parser for MapExp<P, NEX, F> {
    type Out = P::Out;
    type Ex = NEX;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseERes<'a, P::Out, NEX> {
        match self.p.parse(it) {
            Err(mut e) => Err(e.new_exp((self.f)(e.exp))),
            Ok(ov) => Ok(ov),
        }
    }
}
pub struct Break<P: Parser> {
    p: P,
}

impl<P: Parser> Parser for Break<P> {
    type Out = P::Out;
    type Ex = P::Ex;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseERes<'a, Self::Out, Self::Ex> {
        match self.p.parse(it) {
            Err(e) => Err(e.brk()),
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
