use crate::ptrait::*;
use std::marker::PhantomData;

pub struct Maybe<I, A> {
    p: A,
    phi: PhantomData<I>,
}

impl<A, I: Iterator + Clone, V> Parser<I, Option<V>> for Maybe<I, A>
where
    A: Parser<I, V>,
{
    fn parse(&self, i: &I) -> ParseRes<I, Option<V>> {
        match self.p.parse(i) {
            Ok((ir, v)) => Ok((ir, Some(v))),
            Err(_) => Ok((i.clone(), None)),
        }
    }
}

pub fn maybe<P: Parser<I, V>, I, V>(p: P) -> Maybe<I, P> {
    Maybe {
        p,
        phi: PhantomData,
    }
}

pub struct Wrap<I, A, B, VA, VB> {
    a: A,
    b: B,
    phi: PhantomData<I>,
    pha: PhantomData<VA>,
    phb: PhantomData<VB>,
}

impl<I, A, B, VA, VB> Parser<I, VB> for Wrap<I, A, B, VA, VB>
where
    A: Parser<I, VA>,
    B: Parser<I, VB>,
{
    fn parse(&self, i: &I) -> ParseRes<I, VB> {
        let (i, _) = self.a.parse(i)?;
        let (i, res) = self.b.parse(&i)?;
        let (n, _) = self.a.parse(&i)?;
        Ok((n, res))
    }
}

pub fn wrap<A, B, I, VA, VB>(a: A, b: B) -> Wrap<I, A, B, VA, VB>
where
    A: Parser<I, VA>,
    B: Parser<I, VB>,
{
    Wrap {
        a,
        b,
        phi: PhantomData,
        pha: PhantomData,
        phb: PhantomData,
    }
}
