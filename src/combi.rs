use crate::iter::*;
use crate::ptrait::*;
use std::marker::PhantomData;

pub struct Maybe<A> {
    p: A,
}

impl<A, V> Parser<Option<V>> for Maybe<A>
where
    A: Parser<V>,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Option<V>> {
        match self.p.parse(i) {
            Ok((ir, v)) => Ok((ir, Some(v))),
            Err(_) => Ok((i.clone(), None)),
        }
    }
}

pub fn maybe<P: Parser<V>, V>(p: P) -> Maybe<P> {
    Maybe { p }
}

pub struct Wrap<A, B, VA, VB> {
    a: A,
    b: B,
    pha: PhantomData<VA>,
    phb: PhantomData<VB>,
}

impl<A, B, VA, VB> Parser<VB> for Wrap<A, B, VA, VB>
where
    A: Parser<VA>,
    B: Parser<VB>,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, VB> {
        let (i, _) = self.a.parse(i)?;
        let (i, res) = self.b.parse(&i)?;
        let (n, _) = self.a.parse(&i)?;
        Ok((n, res))
    }
}

pub fn wrap<A, B, VA, VB>(a: A, b: B) -> Wrap<A, B, VA, VB>
where
    A: Parser<VA>,
    B: Parser<VB>,
{
    Wrap {
        a,
        b,
        pha: PhantomData,
        phb: PhantomData,
    }
}
