use crate::err::Expected;
use crate::iter::*;
use crate::ptrait::*;

impl<A, B> Parser for (A, B)
where
    A: Parser,
    B: Parser,
{
    type Out = (A::Out, B::Out);
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let (it2, av, c1) = self.0.parse(it)?;
        let (it3, bv, c2) = self.1.parse(&it2).map_err(|e| e.cont(c1))?;
        Ok((it3, (av, bv), c2))
    }
    fn expected(&self) -> Expected {
        Expected::first(self.0.expected(), self.1.expected())
    }
}

impl<A, B, C> Parser for (A, B, C)
where
    A: Parser,
    B: Parser,
    C: Parser,
{
    type Out = (A::Out, B::Out, C::Out);
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let (it2, av, c1) = self.0.parse(it)?;
        let (it3, bv, c2) = self.1.parse(&it2).map_err(|e| e.cont(c1))?;
        let (it4, cv, c3) = self.2.parse(&it3).map_err(|e| e.cont(c2))?;
        Ok((it4, (av, bv, cv), c3))
    }
    fn expected(&self) -> Expected {
        Expected::first(self.0.expected(), self.1.expected())
    }
}

impl<A, B, C, D> Parser for (A, B, C, D)
where
    A: Parser,
    B: Parser,
    C: Parser,
    D: Parser,
{
    type Out = (A::Out, B::Out, C::Out, D::Out);
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let (it2, av, c1) = self.0.parse(it)?;
        let (it3, bv, c2) = self.1.parse(&it2).map_err(|e| e.cont(c1))?;
        let (it4, cv, c3) = self.2.parse(&it3).map_err(|e| e.cont(c2))?;
        let (it5, dv, c4) = self.3.parse(&it4).map_err(|e| e.cont(c3))?;
        Ok((it5, (av, bv, cv, dv), c4))
    }
    fn expected(&self) -> Expected {
        Expected::first(self.0.expected(), self.1.expected())
    }
}
impl<A, B, C, D, E> Parser for (A, B, C, D, E)
where
    A: Parser,
    B: Parser,
    C: Parser,
    D: Parser,
    E: Parser,
{
    type Out = (A::Out, B::Out, C::Out, D::Out, E::Out);
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let (it2, av, c1) = self.0.parse(it)?;
        let (it3, bv, c2) = self.1.parse(&it2).map_err(|e| e.cont(c1))?;
        let (it4, cv, c3) = self.2.parse(&it3).map_err(|e| e.cont(c2))?;
        let (it5, dv, c4) = self.3.parse(&it4).map_err(|e| e.cont(c3))?;
        let (it6, ev, c5) = self.4.parse(&it5).map_err(|e| e.cont(c4))?;
        Ok((it6, (av, bv, cv, dv, ev), c5))
    }
    fn expected(&self) -> Expected {
        Expected::first(self.0.expected(), self.1.expected())
    }
}
impl<A, B, C, D, E, F> Parser for (A, B, C, D, E, F)
where
    A: Parser,
    B: Parser,
    C: Parser,
    D: Parser,
    E: Parser,
    F: Parser,
{
    type Out = (A::Out, B::Out, C::Out, D::Out, E::Out, F::Out);
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let (it2, av, c1) = self.0.parse(it)?;
        let (it3, bv, c2) = self.1.parse(&it2).map_err(|e| e.cont(c1))?;
        let (it4, cv, c3) = self.2.parse(&it3).map_err(|e| e.cont(c2))?;
        let (it5, dv, c4) = self.3.parse(&it4).map_err(|e| e.cont(c3))?;
        let (it6, ev, c5) = self.4.parse(&it5).map_err(|e| e.cont(c4))?;
        let (it7, fv, c6) = self.5.parse(&it6).map_err(|e| e.cont(c5))?;
        Ok((it7, (av, bv, cv, dv, ev, fv), c6))
    }
    fn expected(&self) -> Expected {
        Expected::first(self.0.expected(), self.1.expected())
    }
}

pub fn first<A, B>(a: A, b: B) -> impl Parser<Out = A::Out>
where
    A: Parser,
    B: Parser,
{
    a.then_ig(b)
}
pub fn last<A, B>(a: A, b: B) -> impl Parser<Out = B::Out>
where
    A: Parser,
    B: Parser,
{
    a.ig_then(b)
}

pub fn middle<A, B, C>(a: A, b: B, c: C) -> impl Parser<Out = B::Out>
where
    A: Parser,
    B: Parser,
    C: Parser,
{
    a.ig_then(b).then_ig(c)
}

pub fn or<A, B, V>(a: A, b: B) -> impl Parser<Out = V>
where
    A: Parser<Out = V>,
    B: Parser<Out = V>,
{
    a.or(b)
}

/// While you can use the numbered 'or's you may find the 'or!' macro helpful as that works with
/// any number of options without needing to count, and returns the exact same result
pub fn or3<A, B, C, V>(a: A, b: B, c: C) -> impl Parser<Out = V>
where
    A: Parser<Out = V>,
    B: Parser<Out = V>,
    C: Parser<Out = V>,
{
    a.or(b).or(c)
}

pub fn or4<A, B, C, D, V>(a: A, b: B, c: C, d: D) -> impl Parser<Out = V>
where
    A: Parser<Out = V>,
    B: Parser<Out = V>,
    C: Parser<Out = V>,
    D: Parser<Out = V>,
{
    a.or(b).or(c).or(d)
}
pub fn or5<A, B, C, D, E, V>(a: A, b: B, c: C, d: D, e: E) -> impl Parser<Out = V>
where
    A: Parser<Out = V>,
    B: Parser<Out = V>,
    C: Parser<Out = V>,
    D: Parser<Out = V>,
    E: Parser<Out = V>,
{
    a.or(b).or(c).or(d).or(e)
}
pub fn or6<A, B, C, D, E, F, V>(a: A, b: B, c: C, d: D, e: E, f: F) -> impl Parser<Out = V>
where
    A: Parser<Out = V>,
    B: Parser<Out = V>,
    C: Parser<Out = V>,
    D: Parser<Out = V>,
    E: Parser<Out = V>,
    F: Parser<Out = V>,
{
    a.or(b).or(c).or(d).or(e).or(f)
}
