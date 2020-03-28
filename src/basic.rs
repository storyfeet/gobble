use crate::err::ParseError;
use std::marker::PhantomData;

pub type ParseRes<I, V> = Result<(I, V), ParseError>;

pub trait Parser<I, V>: Sized {
    fn parse(&self, i: &I) -> ParseRes<I, V>;
    fn then<P: Parser<I, V2>, V2>(self, p: P) -> Then<Self, P> {
        Then { one: self, two: p }
    }
    fn then_ig<P: Parser<I, V2>, V2>(self, p: P) -> ThenIg<Self, P, V2> {
        ThenIg {
            one: self,
            two: p,
            ph: PhantomData,
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
        loop {
            let mut i2 = i.clone();
            match i2.next() {
                Some(c) => {
                    if !(self.f)(c) {
                        if n < self.min {
                            return Err(ParseError::new("not enough to take", 0));
                        }
                        return Ok((i, ()));
                    }
                }
                None => {
                    if n < self.min {
                        return Err(ParseError::new("End of str before end of take", 0));
                    }
                }
            }
            i = i2;
            n += 1;
        }
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
        println!("parsing Then");
        let (i, v1) = self.one.parse(i)?;
        let (i, v2) = self.two.parse(&i)?;
        Ok((i, (v1, v2)))
    }
}

pub struct ThenIg<A, B, BV> {
    one: A,
    two: B,
    ph: PhantomData<BV>,
}

impl<I, V1, V2, A, B> Parser<I, V1> for ThenIg<A, B, V2>
where
    A: Parser<I, V1>,
    B: Parser<I, V2>,
{
    fn parse(&self, i: &I) -> ParseRes<I, V1> {
        println!("parsing then_ig");
        let (i, v1) = self.one.parse(i)?;
        let (i, _) = self.two.parse(&i)?;
        Ok((i, v1))
    }
}
