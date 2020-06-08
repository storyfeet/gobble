use crate::err::{longer, Expected, ParseError};
use crate::iter::LCChars;
use std::cmp::Ordering;

pub type ParseRes<'a, V> = Result<(LCChars<'a>, V, Option<Expected>), ParseError>;

/// The core trait for parsing
pub trait Parser: Sized {
    type Out;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out>;

    fn expected(&self) -> Expected {
        Expected::ObOn(
            std::any::type_name::<Self>(),
            std::any::type_name::<Self::Out>(),
        )
    }
    fn parse_s(&self, s: &str) -> Result<Self::Out, ParseError> {
        self.parse(&LCChars::str(s)).map(|(_, v, _)| v)
    }

    fn parse_sn<'a>(&self, s: &'a str) -> Result<(&'a str, Self::Out), ParseError> {
        self.parse(&LCChars::str(s))
            .map(|(i, v, _)| (i.as_str(), v))
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
    fn try_map<F: Fn(Self::Out) -> Result<V2, Expected>, V2>(self, f: F) -> TryMap<Self, V2, F> {
        TryMap { a: self, f }
    }

    fn asv<R: Clone>(self, r: R) -> As<Self, R> {
        As { a: self, r }
    }

    fn ig(self) -> As<Self, ()> {
        self.asv(())
    }

    fn map_exp<F: Fn(Expected) -> Expected>(self, f: F) -> MapExp<Self, F> {
        MapExp { p: self, f }
    }

    fn brk(self) -> Break<Self> {
        Break { p: self }
    }
}

impl<V, F: for<'a> Fn(&LCChars<'a>) -> ParseRes<'a, V>> Parser for F {
    type Out = V;
    fn parse<'b>(&self, i: &LCChars<'b>) -> ParseRes<'b, V> {
        self(i)
    }
}

impl Parser for &'static str {
    type Out = &'static str;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, &'static str> {
        crate::reader::do_tag(i, self)
    }
    fn expected(&self) -> Expected {
        Expected::Str(self)
    }
}

impl Parser for char {
    type Out = char;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, char> {
        let mut i2 = i.clone();
        match i2.next() {
            Some(c) if c == *self => Ok((i2, *self, None)),
            _ => i2.err_p_r(self),
        }
    }
    fn expected(&self) -> Expected {
        Expected::Char(*self)
    }
}

#[derive(Clone)]
pub struct Then<A, B> {
    a: A,
    b: B,
}

impl<A, B> Parser for Then<A, B>
where
    A: Parser,
    B: Parser,
{
    type Out = (A::Out, B::Out);
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let (i, v1, c1) = self.a.parse(i)?;
        let (i, v2, e) = self.b.parse(&i).map_err(|e| e.cont(c1))?;
        Ok((i, (v1, v2), e))
    }
    fn expected(&self) -> Expected {
        Expected::first(self.a.expected(), self.b.expected())
    }
}

#[derive(Clone)]
pub struct ThenIg<A, B> {
    a: A,
    b: B,
}

impl<A, B> Parser for ThenIg<A, B>
where
    A: Parser,
    B: Parser,
{
    type Out = A::Out;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let (i, v1, c1) = self.a.parse(i)?;
        let (i, _, ct) = self.b.parse(&i).map_err(|e| e.cont(c1))?;
        Ok((i, v1, ct))
    }
    fn expected(&self) -> Expected {
        Expected::first(self.a.expected(), self.b.expected())
    }
}

#[derive(Clone)]
pub struct IgThen<A, B> {
    a: A,
    b: B,
}

impl<A, B> Parser for IgThen<A, B>
where
    A: Parser,
    B: Parser,
{
    type Out = B::Out;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let (i, _, c1) = self.a.parse(i)?;
        let (i, v2, ex) = self.b.parse(&i).map_err(|e| e.cont(c1))?;
        Ok((i, v2, ex))
    }
    fn expected(&self) -> Expected {
        Expected::first(self.a.expected(), self.b.expected())
    }
}

#[derive(Clone)]
pub struct Or<A, B> {
    a: A,
    b: B,
}

impl<A, B, V> Parser for Or<A, B>
where
    A: Parser<Out = V>,
    B: Parser<Out = V>,
{
    type Out = V;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, V> {
        match self.a.parse(i) {
            Ok((r, v, e)) => Ok((r, v, e)),
            Err(e) if e.is_break() => Err(e),
            Err(e) => match self.b.parse(i) {
                Ok((r, v, ex)) => Ok((r, v, ex)),
                Err(e2) if e2.is_break() => Err(e2),
                Err(e2) => match e.partial_cmp(&e2) {
                    Some(Ordering::Equal) | None => Err(longer(e, e2).wrap(i.err_p(self))),
                    Some(Ordering::Less) => Err(e2),
                    Some(Ordering::Greater) => Err(e),
                },
            },
        }
    }
    fn expected(&self) -> Expected {
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
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, B> {
        let (ri, v, ex) = self.a.parse(i)?;
        Ok((ri, (self.f)(v), ex))
    }
    fn expected(&self) -> Expected {
        self.a.expected()
    }
}

#[derive(Clone)]
pub struct TryMap<A: Parser, B, F: Fn(A::Out) -> Result<B, Expected>> {
    a: A,
    f: F,
}

impl<A: Parser, B, F: Fn(A::Out) -> Result<B, Expected>> Parser for TryMap<A, B, F> {
    type Out = B;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, B> {
        let (ri, v, ct) = self.a.parse(i)?;
        match (self.f)(v) {
            Ok(v2) => Ok((ri, v2, ct)),
            Err(e) => ri.err_ex_r(e),
        }
    }
    fn expected(&self) -> Expected {
        self.a.expected()
    }
}

pub struct As<A: Parser, R: Clone> {
    a: A,
    r: R,
}
impl<A: Parser, R: Clone> Parser for As<A, R> {
    type Out = R;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, R> {
        let (ri, _, ct) = self.a.parse(it)?;
        Ok((ri, self.r.clone(), ct))
    }
    fn expected(&self) -> Expected {
        self.a.expected()
    }
}

pub struct MapExp<P: Parser, F: Fn(Expected) -> Expected> {
    p: P,
    f: F,
}

impl<P: Parser, F: Fn(Expected) -> Expected> Parser for MapExp<P, F> {
    type Out = P::Out;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, P::Out> {
        match self.p.parse(it) {
            Err(mut e) => {
                e.exp = (self.f)(e.exp);
                Err(e)
            }
            Ok(ov) => Ok(ov),
        }
    }
}
pub struct Break<P: Parser> {
    p: P,
}

impl<P: Parser> Parser for Break<P> {
    type Out = P::Out;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
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
