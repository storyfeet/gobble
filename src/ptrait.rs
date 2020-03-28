use crate::err::ParseError;
use std::marker::PhantomData;

pub type ParseRes<I, V> = Result<(I, V), ParseError>;

pub trait Parser<I, V>: Sized {
    fn parse(&self, i: &I) -> ParseRes<I, V>;
    fn then<P: Parser<I, V2>, V2>(self, p: P) -> Then<Self, P> {
        Then { one: self, two: p }
    }
    fn then_ig<P: Parser<I, V2>, V2>(self, p: P) -> ThenIg<Self, P, V, V2> {
        ThenIg {
            one: self,
            two: p,
            pha: PhantomData,
            phb: PhantomData,
        }
    }
    fn ig_then<P: Parser<I, V2>, V2>(self, p: P) -> IgThen<I, Self, P, V, V2> {
        IgThen {
            one: self,
            two: p,
            pha: PhantomData,
            phb: PhantomData,
            phi: PhantomData,
        }
    }
    fn or<P: Parser<I, V>>(self, p: P) -> Or<I, Self, P> {
        Or {
            a: self,
            b: p,
            phi: PhantomData,
        }
    }
}

impl<V, I, F: Fn(&I) -> ParseRes<I, V>> Parser<I, V> for F {
    fn parse(&self, i: &I) -> ParseRes<I, V> {
        self(i)
    }
}

impl<F, I, C> Parser<I, ()> for Take<F>
where
    F: Fn(C) -> bool,
    I: Clone + Iterator<Item = C>,
{
    fn parse(&self, i: &I) -> ParseRes<I, ()> {
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
            return Err(ParseError::new("End of str before end of take", 0));
        }
        Ok((i2, ()))
    }
}

pub struct Take<F> {
    f: F,
    min: usize,
}

pub fn take<F, C>(f: F, min: usize) -> Take<F>
where
    F: Fn(C) -> bool,
{
    Take { f, min }
}

pub fn ws(min: usize) -> Take<impl Fn(char) -> bool> {
    Take {
        f: |c| match c {
            ' ' | '\t' | '\n' | '\r' => true,
            _ => false,
        },
        min,
    }
}
pub struct Then<A, B> {
    one: A,
    two: B,
}

impl<I, V1, V2, A, B> Parser<I, (V1, V2)> for Then<A, B>
where
    A: Parser<I, V1>,
    B: Parser<I, V2>,
{
    fn parse(&self, i: &I) -> ParseRes<I, (V1, V2)> {
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

impl<I, V1, V2, A, B> Parser<I, V1> for ThenIg<A, B, V1, V2>
where
    A: Parser<I, V1>,
    B: Parser<I, V2>,
{
    fn parse(&self, i: &I) -> ParseRes<I, V1> {
        let (i, v1) = self.one.parse(i)?;
        let (i, _) = self.two.parse(&i)?;
        Ok((i, v1))
    }
}

pub struct IgThen<I, A, B, AV, BV> {
    one: A,
    two: B,
    pha: PhantomData<AV>,
    phb: PhantomData<BV>,
    phi: PhantomData<I>,
}

impl<I, V1, V2, A, B> Parser<I, V2> for IgThen<I, A, B, V1, V2>
where
    A: Parser<I, V1>,
    B: Parser<I, V2>,
{
    fn parse(&self, i: &I) -> ParseRes<I, V2> {
        let (i, _) = self.one.parse(i)?;
        let (i, v2) = self.two.parse(&i)?;
        Ok((i, v2))
    }
}

pub struct Or<I, A, B> {
    a: A,
    b: B,
    phi: PhantomData<I>,
}

impl<I, V1, A, B> Parser<I, V1> for Or<I, A, B>
where
    A: Parser<I, V1>,
    B: Parser<I, V1>,
{
    fn parse(&self, i: &I) -> ParseRes<I, V1> {
        if let Ok((r, v)) = self.a.parse(i) {
            Ok((r, v))
        } else {
            self.b.parse(i)
        }
    }
}
