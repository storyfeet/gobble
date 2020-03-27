use crate::err::ParseError;

pub type ParseRes<I, V> = Result<(I, V), ParseError>;

pub trait Parser<I, V> {
    fn parse(&self, i: &I) -> ParseRes<I, V>;
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
