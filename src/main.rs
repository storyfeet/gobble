pub enum Expr {
    Val(i32),
    Add(Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub struct ParseError {
    mess: String,
    line: u64,
}

impl ParseError {
    pub fn new(s: &str, line: u64) -> ParseError {
        ParseError {
            mess: s.to_string(),
            line,
        }
    }
}

pub type ParseRes<I, V> = Result<(I, V), ParseError>;

pub trait Parser<I, V> {
    fn parse(&self, i: &I) -> ParseRes<I, V>;
}

impl<V, I, F: Fn(&I) -> ParseRes<I, V>> Parser<I, V> for F {
    fn parse(&self, i: &I) -> ParseRes<I, V> {
        self(i)
    }
}

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
    min: u64,
}

pub fn take<F, C>(f: F, min: u64) -> Take<F>
where
    F: Fn(C) -> bool,
{
    Take { f, min }
}

pub fn ws(min: u64) -> Take<impl Fn(char) -> bool> {
    Take {
        f: |c| match c {
            ' ' | '\t' | '\n' | '\r' => true,
            _ => false,
        },
        min,
    }
}

fn main() {}

#[cfg(test)]
pub mod test {
    use super::*;
    #[test]
    fn test_can_build_ws_with_other() {
        let parser = wrap::<_, _, std::str::Chars, _>(ws(2), take(|c| c == 'p', 4));
        //let parser = take(|c| c == ' ' || c == 'p', 4);
        let (mut r, _) = parser.parse(&"  pppp  ,".chars()).unwrap();
        assert_eq!(r.next(), Some(','));
    }
}
