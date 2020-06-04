use crate::err::*;
use crate::iter::LCChars;
use crate::ptrait::*;

#[derive(Clone)]
pub struct RepeatN<A: Parser> {
    n: usize,
    a: A,
}

/// ```
/// use gobble::*;
/// let it = LCChars::str("hello fish car cat");
/// let (_,v) = do_repeat_n(&it,&common_ident.then_ig(" "),3).unwrap();
/// assert_eq!(v,vec!["hello","fish","car"]);
///
/// ```
pub fn do_repeat_n<'a, A: Parser>(it: &LCChars<'a>, a: &A, n: usize) -> ParseRes<'a, Vec<A::Out>> {
    let mut i = it.clone();
    let mut res = Vec::new();
    for x in 0..n {
        match a.parse(&i) {
            Ok((it2, pres)) => {
                res.push(pres);
                i = it2;
            }
            Err(e) => return i.err_cr(ECode::Count(n, x, Box::new(e))),
        }
    }
    return Ok((i, res));
}

impl<A: Parser> Parser for RepeatN<A> {
    type Out = Vec<A::Out>;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Vec<A::Out>> {
        do_repeat_n(it, &self.a, self.n)
    }
}

pub struct Reflect<A, B, C> {
    a: A,
    b: B,
    c: C,
}
impl<A, B, C> Parser for Reflect<A, B, C>
where
    A: Parser,
    B: Parser,
    C: Parser,
{
    type Out = (Vec<A::Out>, B::Out, Vec<C::Out>);
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let (ni, (va, b)) = do_repeat_until(it, &self.a, &self.b)?;
        let (fi, vc) = do_repeat_n(&ni, &self.c, va.len())?;
        Ok((fi, (va, b, vc)))
    }
}

/// A function for making sure number match on both sides of an equals
///
/// ```rust
/// use gobble::*;
/// let p = reflect(s_("("),Alpha.min_n(1),s_(")"));
///
/// let (av,b,cv) =p.parse_s("(((help)))").unwrap();
///
/// assert_eq!(av,vec!["(","(","("]);
/// assert_eq!(b,"help".to_string());
/// assert_eq!(cv,vec![")",")",")"]);
///
/// let r2 = p.parse_s("(((no))");
/// assert!(r2.is_err());
/// ```
///
pub fn reflect<A, B, C>(a: A, b: B, c: C) -> Reflect<A, B, C>
where
    A: Parser,
    B: Parser,
    C: Parser,
{
    Reflect { a, b, c }
}

/// Repeat an exact number of times
///
/// ```
/// use gobble::*;
/// let p = repeat_n(common_int.then_ig(","),5);
/// let v = p.parse_s("7,6,5,4,3,2,1").unwrap();
/// assert_eq!(v,vec![7,6,5,4,3]);
/// ```
pub fn repeat_n<A: Parser>(a: A, n: usize) -> RepeatN<A> {
    RepeatN { a, n }
}

#[derive(Clone)]
pub struct Separated<A: Parser, B: Parser> {
    a: A,
    b: B,
    min: usize,
}

impl<A, B> Parser for Separated<A, B>
where
    A: Parser,
    B: Parser,
{
    type Out = Vec<A::Out>;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let mut res = Vec::new();
        let mut ri = match self.a.parse(i) {
            Ok((r, v)) => {
                res.push(v);
                r
            }
            Err(e) => {
                if res.len() >= self.min {
                    return Ok((i.clone(), res));
                } else {
                    return i.err_cr(ECode::Wrap("Not enough contents", Box::new(e)));
                }
            }
        };
        loop {
            //try sep if not found, return
            ri = match self.b.parse(&ri) {
                Ok((r, _)) => r,
                Err(_) => return Ok((ri, res)),
            };

            ri = match self.a.parse(&ri) {
                Ok((r, v)) => {
                    res.push(v);
                    r
                }
                Err(e) => {
                    return i.err_cr(ECode::Wrap("Nothing after sep", Box::new(e)));
                }
            };
        }
    }
}

pub fn sep<A: Parser, B: Parser>(a: A, b: B, min: usize) -> Separated<A, B> {
    Separated { a, b, min }
}

#[derive(Clone)]
pub struct Repeater<A> {
    a: A,
    min: usize,
}

impl<A: Parser> Parser for Repeater<A> {
    type Out = Vec<A::Out>;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let mut ri = i.clone();
        let mut res = Vec::new();
        loop {
            ri = match self.a.parse(&ri) {
                Ok((r, v)) => {
                    res.push(v);
                    r
                }
                Err(e) => {
                    if res.len() < self.min {
                        return i.err_cr(ECode::Wrap("not enough elems", Box::new(e)));
                    } else {
                        return Ok((ri, res));
                    }
                }
            }
        }
    }
}

pub fn repeat<A: Parser>(a: A, min: usize) -> Repeater<A> {
    Repeater { a, min }
}

fn do_repeat_until<'a, A: Parser, B: Parser>(
    it: &LCChars<'a>,
    a: &A,
    b: &B,
) -> ParseRes<'a, (Vec<A::Out>, B::Out)> {
    let mut ri = it.clone();
    let mut res = Vec::new();
    loop {
        if let Ok((r, v)) = b.parse(&ri) {
            return Ok((r, (res, v)));
        }
        ri = match a.parse(&ri) {
            Ok((r, v)) => {
                res.push(v);
                r
            }
            Err(e) => match b.parse(&ri) {
                Ok((r, bv)) => return Ok((r, (res, bv))),
                Err(e2) => return ri.err_cr(ECode::Or(Box::new(e), Box::new(e2))),
            },
        }
    }
}

pub struct RepUntil<A, B> {
    a: A,
    b: B,
}

impl<A: Parser, B: Parser> Parser for RepUntil<A, B> {
    type Out = (Vec<A::Out>, B::Out);
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        do_repeat_until(i, &self.a, &self.b)
    }
}

///Repeats the first parser until the second parser.
///returns a vec of the first parsers results
pub fn repeat_until<A: Parser, B: Parser>(a: A, b: B) -> RepUntil<A, B> {
    RepUntil { a, b }
}

pub fn repeat_until_ig<A: Parser, B: Parser>(a: A, b: B) -> impl Parser<Out = Vec<A::Out>> {
    repeat_until(a, b).map(|(a, _)| a)
}

pub struct SepUntil<A, B, C> {
    a: A,
    b: B,
    c: C,
}

impl<A, B, C> Parser for SepUntil<A, B, C>
where
    A: Parser,
    B: Parser,
    C: Parser,
{
    type Out = Vec<A::Out>;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let mut ri = i.clone();
        let mut res = Vec::new();
        match self.c.parse(&ri) {
            Ok((r, _)) => return Ok((r, res)),
            Err(_) => {}
        }
        loop {
            ri = match self.a.parse(&ri) {
                Ok((r, v)) => {
                    res.push(v);
                    r
                }
                Err(e) => return Err(e),
            };
            ri = match self.b.parse(&ri) {
                Ok((r, _)) => r,
                Err(e) => match self.c.parse(&ri) {
                    Ok((r, _)) => return Ok((r, res)),
                    Err(e2) => return ri.err_cr(ECode::Or(Box::new(e), Box::new(e2))),
                },
            }
        }
    }
}

///Allows for better errors looping until a specific finish. It does not return the close or the
///seperators the
///close is expected to be some kind of closer like '}'
///If you need the close you will have to use sep(..).then(..) though the errors will be less
///nice
pub fn sep_until<A, B, C>(a: A, b: B, c: C) -> SepUntil<A, B, C>
where
    A: Parser,
    B: Parser,
    C: Parser,
{
    SepUntil { a, b, c }
}

#[cfg(test)]
pub mod test {
    use super::*;
    //use crate::ptrait::*;
    use crate::*;
    #[test]
    pub fn test_reflecter() {
        let (av, b, cv) = reflect(s_("("), (Alpha, NumDigit).min_n(1), s_(")"))
            .parse_s("(((help)))")
            .unwrap();

        assert_eq!(av, vec!["(", "(", "("]);
        assert_eq!(b, "help".to_string());
        assert_eq!(cv, vec![")", ")", ")"]);
    }
}
