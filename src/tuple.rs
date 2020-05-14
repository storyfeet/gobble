use crate::iter::*;
use crate::ptrait::*;

impl<A, AV, B, BV> Parser<(AV, BV)> for (A, B)
where
    A: Parser<AV>,
    B: Parser<BV>,
{
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, (AV, BV)> {
        let (it2, av) = self.0.parse(it)?;
        let (it3, bv) = self.1.parse(&it2)?;
        Ok((it3, (av, bv)))
    }
}

impl<A, AV, B, BV, C, CV> Parser<(AV, BV, CV)> for (A, B, C)
where
    A: Parser<AV>,
    B: Parser<BV>,
    C: Parser<CV>,
{
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, (AV, BV, CV)> {
        let (it2, av) = self.0.parse(it)?;
        let (it3, bv) = self.1.parse(&it2)?;
        let (it4, cv) = self.2.parse(&it3)?;
        Ok((it4, (av, bv, cv)))
    }
}

impl<A, AV, B, BV, C, CV, D, DV> Parser<(AV, BV, CV, DV)> for (A, B, C, D)
where
    A: Parser<AV>,
    B: Parser<BV>,
    C: Parser<CV>,
    D: Parser<DV>,
{
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, (AV, BV, CV, DV)> {
        let (it2, av) = self.0.parse(it)?;
        let (it3, bv) = self.1.parse(&it2)?;
        let (it4, cv) = self.2.parse(&it3)?;
        let (it5, dv) = self.3.parse(&it4)?;
        Ok((it5, (av, bv, cv, dv)))
    }
}
impl<A, AV, B, BV, C, CV, D, DV, E, EV> Parser<(AV, BV, CV, DV, EV)> for (A, B, C, D, E)
where
    A: Parser<AV>,
    B: Parser<BV>,
    C: Parser<CV>,
    D: Parser<DV>,
    E: Parser<EV>,
{
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, (AV, BV, CV, DV, EV)> {
        let (it2, av) = self.0.parse(it)?;
        let (it3, bv) = self.1.parse(&it2)?;
        let (it4, cv) = self.2.parse(&it3)?;
        let (it5, dv) = self.3.parse(&it4)?;
        let (it6, ev) = self.4.parse(&it5)?;
        Ok((it6, (av, bv, cv, dv, ev)))
    }
}
impl<A, AV, B, BV, C, CV, D, DV, E, EV, F, FV> Parser<(AV, BV, CV, DV, EV, FV)>
    for (A, B, C, D, E, F)
where
    A: Parser<AV>,
    B: Parser<BV>,
    C: Parser<CV>,
    D: Parser<DV>,
    E: Parser<EV>,
    F: Parser<FV>,
{
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, (AV, BV, CV, DV, EV, FV)> {
        let (it2, av) = self.0.parse(it)?;
        let (it3, bv) = self.1.parse(&it2)?;
        let (it4, cv) = self.2.parse(&it3)?;
        let (it5, dv) = self.3.parse(&it4)?;
        let (it6, ev) = self.4.parse(&it5)?;
        let (it7, fv) = self.5.parse(&it6)?;
        Ok((it7, (av, bv, cv, dv, ev, fv)))
    }
}

pub fn first<A: Parser<AV>, B: Parser<BV>, AV, BV>(a: A, b: B) -> impl Parser<AV> {
    a.then_ig(b)
}
pub fn last<A: Parser<AV>, B: Parser<BV>, AV, BV>(a: A, b: B) -> impl Parser<BV> {
    a.ig_then(b)
}

pub fn middle<A, B, C, AV, BV, CV>(a: A, b: B, c: C) -> impl Parser<BV>
where
    A: Parser<AV>,
    B: Parser<BV>,
    C: Parser<CV>,
{
    a.ig_then(b).then_ig(c)
}

pub fn or<A, B, V>(a: A, b: B) -> impl Parser<V>
where
    A: Parser<V>,
    B: Parser<V>,
{
    a.or(b)
}

pub fn or3<A, B, C, V>(a: A, b: B, c: C) -> impl Parser<V>
where
    A: Parser<V>,
    B: Parser<V>,
    C: Parser<V>,
{
    a.or(b).or(c)
}

pub fn or4<A, B, C, D, V>(a: A, b: B, c: C, d: D) -> impl Parser<V>
where
    A: Parser<V>,
    B: Parser<V>,
    C: Parser<V>,
    D: Parser<V>,
{
    a.or(b).or(c).or(d)
}
pub fn or5<A, B, C, D, E, V>(a: A, b: B, c: C, d: D, e: E) -> impl Parser<V>
where
    A: Parser<V>,
    B: Parser<V>,
    C: Parser<V>,
    D: Parser<V>,
    E: Parser<V>,
{
    a.or(b).or(c).or(d).or(e)
}
pub fn or6<A, B, C, D, E, F, V>(a: A, b: B, c: C, d: D, e: E, f: F) -> impl Parser<V>
where
    A: Parser<V>,
    B: Parser<V>,
    C: Parser<V>,
    D: Parser<V>,
    E: Parser<V>,
    F: Parser<V>,
{
    a.or(b).or(c).or(d).or(e).or(f)
}
