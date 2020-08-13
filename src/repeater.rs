use crate::iter::LCChars;
use crate::ptrait::*;

#[derive(Clone)]
pub struct Exact<A: Parser> {
    n: usize,
    a: A,
}

/// ```
/// use gobble::*;
/// let it = LCChars::str("hello fish car cat");
/// let (_,v,_) = do_exact(&it,&last(WS.star(),common::Ident),3).unwrap();
/// assert_eq!(v,vec!["hello","fish","car"]);
///
/// ```
pub fn do_exact<'a, A: Parser>(it: &LCChars<'a>, a: &A, n: usize) -> ParseRes<'a, Vec<A::Out>> {
    let mut i = it.clone();
    let mut res = Vec::new();
    for _ in 0..n {
        match a.parse(&i) {
            Ok((it2, pres, _)) => {
                res.push(pres);
                i = it2;
            }
            Err(e) => return Err(e.wrap(i.err_p(a))),
        }
    }
    return Ok((i, res, None));
}

impl<A: Parser> Parser for Exact<A> {
    type Out = Vec<A::Out>;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Vec<A::Out>> {
        do_exact(it, &self.a, self.n)
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
        let (ni, (va, b), _) = do_repeat_until(it, 1, &self.a, &self.b)?;
        let (fi, vc, _) = do_exact(&ni, &self.c, va.len())?;
        Ok((fi, (va, b, vc), None))
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

#[deprecated(since = "0.4.0", note = "Use 'exact' instead")]
pub fn repeat_n<A: Parser>(a: A, n: usize) -> Exact<A> {
    Exact { a, n }
}

/// Repeat an exact number of times
///
/// ```
/// use gobble::*;
/// let p = repeat_n(common::Int.then_ig(","),5);
/// let v = p.parse_s("7,6,5,4,3,2,1").unwrap();
/// assert_eq!(v,vec![7,6,5,4,3]);
/// ```
pub fn exact<A: Parser>(a: A, n: usize) -> Exact<A> {
    Exact { a, n }
}

fn do_sep<'a, A: Parser, B: Parser>(
    i: &LCChars<'a>,
    a: &A,
    b: &B,
    min: usize,
) -> ParseRes<'a, Vec<A::Out>> {
    let mut res = Vec::new();
    let mut ri = i.clone();
    loop {
        ri = match a.parse(&ri) {
            Ok((r, v, _)) => {
                res.push(v);
                r
            }
            Err(_) => {
                if res.len() == 0 && min == 0 {
                    let eo = ri.err_op(a);
                    return Ok((ri, res, eo));
                }
                return i.err_rp(a);
            }
        };
        //try sep if not found, return
        ri = match b.parse(&ri) {
            Ok((r, _, _)) => r,
            Err(e) => {
                if res.len() < min {
                    return ri.err_rp(b);
                } else {
                    return Ok((ri, res, Some(e)));
                }
            }
        };
    }
}

#[derive(Clone)]
pub struct SepStar<A: Parser, B: Parser> {
    a: A,
    b: B,
}

impl<A, B> Parser for SepStar<A, B>
where
    A: Parser,
    B: Parser,
{
    type Out = Vec<A::Out>;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        do_sep(it, &self.a, &self.b, 0)
    }
}

#[deprecated(since = "0.5.0", note = "use sep_star instead")]
pub fn sep<A: Parser, B: Parser>(a: A, b: B) -> SepStar<A, B> {
    SepStar { a, b }
}

pub fn sep_star<A: Parser, B: Parser>(a: A, b: B) -> SepStar<A, B> {
    SepStar { a, b }
}
pub fn sep_plus<A: Parser, B: Parser>(a: A, b: B) -> SepPlus<A, B> {
    SepPlus { a, b }
}

#[derive(Clone)]
pub struct SepPlus<A: Parser, B: Parser> {
    a: A,
    b: B,
}

impl<A, B> Parser for SepPlus<A, B>
where
    A: Parser,
    B: Parser,
{
    type Out = Vec<A::Out>;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        do_sep(it, &self.a, &self.b, 1)
    }
}

pub fn do_rep<'a, A: Parser>(i: &LCChars<'a>, a: &A, min: usize) -> ParseRes<'a, Vec<A::Out>> {
    let mut ri = i.clone();
    let mut res = Vec::new();
    //This closure exists to to make sure th
    let f_done = |it: LCChars<'a>, fres: Vec<A::Out>| {
        if fres.len() < min {
            it.err_rp(a)
        } else {
            let eo = it.err_op(a);
            Ok((it, fres, eo))
        }
    };

    loop {
        ri = match a.parse(&ri) {
            Ok((r, v, _)) => {
                if ri.lc() == r.lc() {
                    return f_done(ri, res);
                }
                res.push(v);
                r
            }
            Err(_) => {
                return f_done(ri, res);
            }
        }
    }
}

#[derive(Clone)]
pub struct RepStar<A> {
    a: A,
}

impl<A: Parser> Parser for RepStar<A> {
    type Out = Vec<A::Out>;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        do_rep(i, &self.a, 0)
    }
}

#[deprecated(since = "0.5.0", note = "use star instead")]
pub fn rep<A: Parser>(a: A) -> RepStar<A> {
    RepStar { a }
}

pub fn star<A: Parser>(a: A) -> RepStar<A> {
    RepStar { a }
}

#[derive(Clone)]
pub struct RepPlus<A> {
    a: A,
}

impl<A: Parser> Parser for RepPlus<A> {
    type Out = Vec<A::Out>;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        do_rep(i, &self.a, 1)
    }
}

#[deprecated(since = "0.5.0", note = "use plus instead")]
pub fn rep_plus<A: Parser>(a: A) -> RepPlus<A> {
    RepPlus { a }
}

pub fn plus<A: Parser>(a: A) -> RepPlus<A> {
    RepPlus { a }
}

fn do_repeat_until<'a, A: Parser, B: Parser>(
    it: &LCChars<'a>,
    min: i32,
    a: &A,
    b: &B,
) -> ParseRes<'a, (Vec<A::Out>, B::Out)> {
    let mut ri = it.clone();
    let mut res = Vec::new();
    let mut done = 0;
    loop {
        let b_err = match done >= min {
            true => match b.parse(&ri) {
                Ok((r, v, _)) => return Ok((r, (res, v), None)),
                Err(e) => Some(e),
            },
            false => None,
        };
        ri = match a.parse(&ri) {
            Ok((r, v, _)) => {
                if r.lc() == ri.lc() {
                    return ri.err_rp(a);
                }
                res.push(v);
                r
            }
            Err(e) => {
                return match b_err {
                    Some(b_err) => Err(e.join(b_err)),
                    None => Err(e),
                }
            }
        };
        done += 1;
    }
}

pub struct StarUntil<A, B> {
    a: A,
    b: B,
}

impl<A: Parser, B: Parser> Parser for StarUntil<A, B> {
    type Out = (Vec<A::Out>, B::Out);
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        do_repeat_until(i, 0, &self.a, &self.b)
    }
}

pub struct PlusUntil<A, B> {
    a: A,
    b: B,
}

impl<A: Parser, B: Parser> Parser for PlusUntil<A, B> {
    type Out = (Vec<A::Out>, B::Out);
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        do_repeat_until(i, 1, &self.a, &self.b)
    }
}

///Repeats the first parser until the second parser.
///returns a vec of the first parsers results
#[deprecated(since = "0.5.2", note = "use star_until instead")]
pub fn repeat_until<A: Parser, B: Parser>(a: A, b: B) -> StarUntil<A, B> {
    StarUntil { a, b }
}

pub fn star_until<A: Parser, B: Parser>(a: A, b: B) -> StarUntil<A, B> {
    StarUntil { a, b }
}
pub fn plus_until<A: Parser, B: Parser>(a: A, b: B) -> PlusUntil<A, B> {
    PlusUntil { a, b }
}

#[deprecated(since = "0.5.2", note = "use star_until_ig or plus_until_ig instead")]
pub fn repeat_until_ig<A: Parser, B: Parser>(a: A, b: B) -> impl Parser<Out = Vec<A::Out>> {
    star_until(a, b).map(|(a, _)| a)
}

pub fn star_until_ig<A: Parser, B: Parser>(a: A, b: B) -> impl Parser<Out = Vec<A::Out>> {
    star_until(a, b).map(|(a, _)| a)
}
pub fn plus_until_ig<A: Parser, B: Parser>(a: A, b: B) -> impl Parser<Out = Vec<A::Out>> {
    plus_until(a, b).map(|(a, _)| a)
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
    type Out = (Vec<A::Out>, C::Out);
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let mut ri = i.clone();
        let mut res = Vec::new();
        match self.c.parse(&ri) {
            Ok((r, v, _)) => return Ok((r, (res, v), None)),
            Err(_) => {}
        }
        loop {
            ri = match self.a.parse(&ri) {
                Ok((r, v, _)) => {
                    res.push(v);
                    r
                }
                Err(e) => return Err(e),
            };
            let c_err = match self.c.parse(&ri) {
                Ok((r, v, _)) => return Ok((r, (res, v), None)),
                Err(e) => e,
            };
            ri = match self.b.parse(&ri) {
                Ok((r, _, _)) => r,
                Err(e) => return Err(e.join(c_err)),
            }
        }
    }
}

///Allows for better errors looping until a specific finish. It does not return the close or the
///seperators the
///close is expected to be some kind of closer like '}'
///If you need the close you will have to use sep(..).then(..) though the errors will be less
///nice Recent changes mean that this now returns the ending result aswel, if you wish to ignore
///that use sep_until_ig
pub fn sep_until<A, B, C>(a: A, b: B, c: C) -> SepUntil<A, B, C>
where
    A: Parser,
    B: Parser,
    C: Parser,
{
    SepUntil { a, b, c }
}

pub fn sep_until_ig<A, B, C>(a: A, b: B, c: C) -> impl Parser<Out = Vec<A::Out>>
where
    A: Parser,
    B: Parser,
    C: Parser,
{
    sep_until(a, b, c).map(|(a, _)| a)
}

#[cfg(test)]
pub mod test {
    use super::*;
    //use crate::ptrait::*;
    use crate::*;
    #[test]
    pub fn test_reflecter() {
        let (av, b, cv) = reflect(ws__("("), (Alpha, NumDigit).plus(), ws__(")"))
            .parse_s("(((help)))")
            .unwrap();

        assert_eq!(av, vec!["(", "(", "("]);
        assert_eq!(b, "help".to_string());
        assert_eq!(cv, vec![")", ")", ")"]);
    }
}
