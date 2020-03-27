use crate::basic::*;

pub struct Wrap<A, B> {
    outer: A,
    inner: B,
}

impl<I, V, A, B> Parser<I, V> for Wrap<A, B>
where
    A: Parser<I, ()>,
    B: Parser<I, V>,
{
    fn parse(&self, i: &I) -> ParseRes<I, V> {
        let (i, _) = self.outer.parse(i)?;
        let (i, res) = self.inner.parse(&i)?;
        let (n, _) = self.outer.parse(&i)?;
        Ok((n, res))
    }
}

pub fn wrap<A, B, I, V>(outer: A, inner: B) -> Wrap<A, B>
where
    A: Parser<I, V>,
    B: Parser<I, ()>,
{
    Wrap { outer, inner }
}
