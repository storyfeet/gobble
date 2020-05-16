use crate::err::*;
use crate::iter::*;
use crate::ptrait::*;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Clone)]
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

/// returns an option on whether this item was found A common use would be
/// looking for a minus on the front of a number
///
/// ```rust
/// use gobble::*;
/// use std::str::FromStr;
/// let p = maybe(tag("-")).then(read_fs(is_num,1)).try_map(|(m,n)|{
///     let res:i32 = n.parse().map_err(|e|ECode::SMess("num could not convert to i32"))?;
///     if m.is_some() {
///         return Ok(-res )
///     }
///     Ok(res)
/// });
/// let s = p.parse_s("-34").unwrap();
/// assert_eq!(s,-34);
/// let s = p.parse_s("34").unwrap();
/// assert_eq!(s,34);
/// ```
pub fn maybe<P: Parser<V>, V>(p: P) -> Maybe<P> {
    Maybe { p }
}

#[derive(Clone)]
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

impl<P: Parser<V>, V: Debug> Parser<()> for FailOn<P, V> {
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, ()> {
        match self.p.parse(it) {
            Ok((_, v)) => it.err_cr(ECode::FailOn(format!("{:?}", v))),
            Err(_) => Ok((it.clone(), ())),
        }
    }
}

pub struct FailOn<P: Parser<V>, V> {
    p: P,
    phv: PhantomData<V>,
}

pub fn fail_on<P: Parser<V>, V>(p: P) -> FailOn<P, V> {
    FailOn {
        p,
        phv: PhantomData,
    }
}
