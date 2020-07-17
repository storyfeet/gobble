use crate::err::*;
use crate::iter::*;
use crate::ptrait::*;
use std::fmt::Debug;

#[derive(Clone)]
pub struct Maybe<A: Parser> {
    p: A,
}

impl<A> Parser for Maybe<A>
where
    A: Parser,
{
    type Out = Option<A::Out>;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        match self.p.parse(i) {
            Ok((ir, v, ex)) => Ok((ir, Some(v), ex)),
            Err(e) => Ok((i.clone(), None, Some(e))),
        }
    }
}

/// returns an option on whether this item was found A common use would be
/// looking for a minus on the front of a number
///
/// ```rust
/// use gobble::*;
/// use std::str::FromStr;
/// let p = maybe("-").then(NumDigit.min_n(1)).try_map(|(m,n)|{
///     let res:i32 = n.parse().map_err(|e|Expected::Str("[1..9]+"))?;
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
pub fn maybe<P: Parser>(p: P) -> Maybe<P> {
    Maybe { p }
}

pub struct Exists<P: Parser> {
    p: P,
}

impl<P: Parser> Parser for Exists<P> {
    type Out = bool;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, bool> {
        match self.p.parse(it) {
            Ok((nit, _, e)) => Ok((nit, true, e)),
            Err(e) => Ok((it.clone(), false, Some(e))),
        }
    }
}

pub fn exists<P: Parser>(p: P) -> Exists<P> {
    Exists { p }
}

#[derive(Clone)]
pub struct Wrap<A, B> {
    a: A,
    b: B,
}

impl<A, B> Parser for Wrap<A, B>
where
    A: Parser,
    B: Parser,
{
    type Out = B::Out;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let (i, _, c1) = self.a.parse(i)?;
        let (i, res, c2) = self.b.parse(&i).map_err(|e| e.cont(c1))?;
        let (n, _, c3) = self.a.parse(&i).map_err(|e| e.cont(c2))?;
        Ok((n, res, c3))
    }
}

pub fn wrap<A, B>(a: A, b: B) -> Wrap<A, B>
where
    A: Parser,
    B: Parser,
{
    Wrap { a, b }
}

impl<P: Parser<Out = V>, V: Debug> Parser for FailOn<P> {
    type Out = ();
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, ()> {
        match self.p.parse(it) {
            Ok((_, _, _)) => it.err_p_r(self),
            Err(_) => Ok((it.clone(), (), None)),
        }
    }
    fn expected(&self) -> Expected {
        Expected::except(self.p.expected())
    }
}

pub struct FailOn<P: Parser> {
    p: P,
}

pub fn fail_on<P: Parser>(p: P) -> FailOn<P> {
    FailOn { p }
}

#[derive(PartialEq, Debug, Clone)]
pub struct PDebugger<P: Parser> {
    p: P,
    s: &'static str,
}

pub fn debug<P: Parser>(p: P, s: &'static str) -> PDebugger<P> {
    PDebugger { p, s }
}

impl<P: Parser> Parser for PDebugger<P> {
    type Out = P::Out;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        println!("DebuggerPre - {}", self.s);
        let r = self.p.parse(it);
        match &r {
            Ok((nit, _, _)) => {
                let s = match (it.index(), nit.index()) {
                    (Some(st), Some(f)) => &it.as_str()[..f - st],
                    _ => it.as_str(),
                };
                println!("Success - {}, \"{}\"", self.s, s);
            }
            Err(_) => println!("Fail - {}", self.s),
        };
        r
    }
}
