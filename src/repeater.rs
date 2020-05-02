use crate::err::*;
use crate::iter::LCChars;
use crate::ptrait::*;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct RepeatN<A: Parser<AV>, AV> {
    n: usize,
    a: A,
    pha: PhantomData<AV>,
}

fn _repeat_n<'a, A: Parser<AV>, AV>(it: &LCChars<'a>, a: &A, n: usize) -> ParseRes<'a, Vec<AV>> {
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

impl<A: Parser<AV>, AV> Parser<Vec<AV>> for RepeatN<A, AV> {
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Vec<AV>> {
        _repeat_n(it, &self.a, self.n)
    }
}

pub struct Reflect<A, B, C, AV, BV, CV> {
    a: A,
    b: B,
    c: C,
    pha: PhantomData<AV>,
    phb: PhantomData<BV>,
    phc: PhantomData<CV>,
}
impl<A, B, C, AV, BV, CV> Parser<(Vec<AV>, BV, Vec<CV>)> for Reflect<A, B, C, AV, BV, CV>
where
    A: Parser<AV>,
    B: Parser<BV>,
    C: Parser<CV>,
{
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, (Vec<AV>, BV, Vec<CV>)> {
        let (ni, (va, b)) = _repeat_until(it, &self.a, &self.b)?;
        let (fi, vc) = _repeat_n(&ni, &self.c, va.len())?;
        Ok((fi, (va, b, vc)))
    }
}

/// A function for making sure number match on both sides of an equals
///
/// ```rust
/// use gobble::*;
/// let p = reflect(s_tag("("),read_fs(is_alpha_num,1),s_tag(")"));
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
pub fn reflect<A, B, C, AV, BV, CV>(a: A, b: B, c: C) -> Reflect<A, B, C, AV, BV, CV>
where
    A: Parser<AV>,
    B: Parser<BV>,
    C: Parser<CV>,
{
    Reflect {
        a,
        b,
        c,
        pha: PhantomData,
        phb: PhantomData,
        phc: PhantomData,
    }
}

pub fn repeat_n<A: Parser<AV>, AV>(a: A, n: usize) -> RepeatN<A, AV> {
    RepeatN {
        a,
        n,
        pha: PhantomData,
    }
}

#[derive(Clone)]
pub struct Separated<A: Parser<AV>, B: Parser<BV>, AV, BV> {
    a: A,
    b: B,
    pha: PhantomData<AV>,
    phb: PhantomData<BV>,
    min_one: bool,
}

impl<A, B, AV, BV> Parser<Vec<AV>> for Separated<A, B, AV, BV>
where
    A: Parser<AV>,
    B: Parser<BV>,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Vec<AV>> {
        let mut res = Vec::new();
        let mut ri = match self.a.parse(i) {
            Ok((r, v)) => {
                res.push(v);
                r
            }
            Err(e) => {
                if !self.min_one {
                    return Ok((i.clone(), res));
                } else {
                    return i.err_cr(ECode::Wrap("No contents", Box::new(e)));
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

pub fn sep<A: Parser<AV>, B: Parser<BV>, AV, BV>(
    a: A,
    b: B,
    min_one: bool,
) -> Separated<A, B, AV, BV> {
    Separated {
        a,
        b,
        min_one,
        pha: PhantomData,
        phb: PhantomData,
    }
}

#[derive(Clone)]
pub struct Repeater<A, AV> {
    a: A,
    pha: PhantomData<AV>,
    min: usize,
}

impl<A: Parser<AV>, AV> Parser<Vec<AV>> for Repeater<A, AV> {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Vec<AV>> {
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

pub fn repeat<A: Parser<AV>, AV>(a: A, min: usize) -> Repeater<A, AV> {
    Repeater {
        a,
        min,
        pha: PhantomData,
    }
}

fn _repeat_until<'a, A: Parser<AV>, B: Parser<BV>, AV, BV>(
    it: &LCChars<'a>,
    a: &A,
    b: &B,
) -> ParseRes<'a, (Vec<AV>, BV)> {
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

pub struct RepUntil<A, B, AV, BV> {
    a: A,
    b: B,
    pha: PhantomData<AV>,
    phb: PhantomData<BV>,
}

impl<A: Parser<AV>, B: Parser<BV>, AV, BV> Parser<(Vec<AV>, BV)> for RepUntil<A, B, AV, BV> {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, (Vec<AV>, BV)> {
        _repeat_until(i, &self.a, &self.b)
    }
}

///Repeats the first parser until the second parser.
///returns a vec of the first parsers results
pub fn repeat_until<A: Parser<AV>, B: Parser<BV>, AV, BV>(a: A, b: B) -> RepUntil<A, B, AV, BV> {
    RepUntil {
        a,
        b,
        pha: PhantomData,
        phb: PhantomData,
    }
}

pub fn repeat_until_ig<A: Parser<AV>, B: Parser<BV>, AV, BV>(a: A, b: B) -> impl Parser<Vec<AV>> {
    repeat_until(a, b).map(|(a, _)| a)
}

pub struct SepUntil<A, B, C, AV, BV, CV> {
    a: A,
    b: B,
    c: C,
    pha: PhantomData<AV>,
    phb: PhantomData<BV>,
    phc: PhantomData<CV>,
}

impl<A, B, C, AV, BV, CV> Parser<Vec<AV>> for SepUntil<A, B, C, AV, BV, CV>
where
    A: Parser<AV>,
    B: Parser<BV>,
    C: Parser<CV>,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Vec<AV>> {
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
pub fn sep_until<A, B, C, AV, BV, CV>(a: A, b: B, c: C) -> SepUntil<A, B, C, AV, BV, CV>
where
    A: Parser<AV>,
    B: Parser<BV>,
    C: Parser<CV>,
{
    SepUntil {
        a,
        b,
        c,
        pha: PhantomData,
        phb: PhantomData,
        phc: PhantomData,
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::ptrait::*;
    use crate::*;
    #[test]
    pub fn test_reflecter() {
        let (av, b, cv) = reflect(s_tag("("), read_fs(is_alpha_num, 1), s_tag(")"))
            .parse_s("(((help)))")
            .unwrap();

        assert_eq!(av, vec!["(", "(", "("]);
        assert_eq!(b, "help".to_string());
        assert_eq!(cv, vec![")", ")", ")"]);
    }
}
