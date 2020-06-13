use crate::chars::*;
use crate::err::*;
use crate::iter::LCChars;
use crate::ptrait::{As, ParseRes, Parser};
//use crate::skip::skip_while;

pub type StrPos = Pos<()>;

#[derive(Debug, Clone, PartialEq)]
pub struct Pos<O> {
    pub line: usize,
    pub col: usize,
    pub start: usize,
    pub fin: Option<usize>,
    pub ob: O,
}

impl<O> Pos<O> {
    ///This version assumes that this is the string it came from
    pub fn on_str<'a>(&self, s: &'a str) -> &'a str {
        match self.fin {
            Some(f) => &s[self.start..f],
            None => &s[self.start..],
        }
    }
}

pub struct PPos<P: Parser> {
    p: P,
}

impl<P: Parser> Parser for PPos<P> {
    type Out = Pos<P::Out>;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let (line, col) = it.lc();
        let start = it.index().ok_or(it.err_p(&self.p))?;
        let (rit, r, pex) = self.p.parse(it)?;
        let fin = rit.index();
        Ok((
            rit,
            Pos {
                line,
                col,
                start,
                fin,
                ob: r,
            },
            pex,
        ))
    }
}

/// ```rust
/// use gobble::*;
/// let s = " \n  hello   ".to_string();
/// let v = "\n ".any().ig_then(str_pos(Alpha.any())).parse_s(&s).unwrap();
/// assert_eq!(v,Pos{line:1,col:2,start:4,fin:Some(9),ob:()});
/// assert_eq!(v.on_str(&s),"hello");
/// ```
pub fn str_pos<P: Parser>(p: P) -> PPos<As<P, ()>> {
    PPos { p: p.ig() }
}

pub fn pos<P: Parser>(p: P) -> PPos<P> {
    PPos { p }
}

pub fn ws_<P: Parser>(p: P) -> impl Parser<Out = P::Out> {
    WS.skip_star().ig_then(p)
}

///Convenience wrapper to say allow whitespace around whatever I'm parsing.
pub fn s_<P: Parser>(p: P) -> impl Parser<Out = P::Out> {
    crate::combi::wrap(WS.skip_star(), p)
}

///Take at least n white space characters
#[deprecated(since = "0.3.0", note = "use WS.any() or WS.min(n) instead")]
pub fn ws(_min: usize) -> impl Parser<Out = ()> {
    WS.skip_star()
}

pub fn do_keyword<'a, P: Parser>(it: &LCChars<'a>, p: &P) -> ParseRes<'a, P::Out> {
    let (t2, r, _) = p.parse(it)?;
    match t2.clone().next() {
        Some(c) => {
            let al = (Alpha, NumDigit, '_');
            if al.char_bool(c) {
                t2.err_ex_r(Expected::except(al.expected()))
            } else {
                Ok((t2, r, None))
            }
        }
        None => Ok((t2, r, None)),
    }
}

impl<P: Parser> Parser for KeyWord<P> {
    type Out = P::Out;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, P::Out> {
        do_keyword(it, &self.p)
    }
}

pub struct KeyWord<P: Parser> {
    p: P,
}

///```rust
/// use gobble::*;
/// assert_eq!(keyword("let").parse_s("let"), Ok("let"));
/// assert_eq!(keyword("let").parse_s("let "), Ok("let"));
/// assert_eq!(keyword("let").parse_s("let*"), Ok("let"));
/// assert!(keyword("let").parse_s("letl").is_err());
///```
pub fn keyword<P: Parser>(p: P) -> KeyWord<P> {
    KeyWord { p }
}

pub fn do_tag<'a>(it: &LCChars<'a>, tg: &'static str) -> ParseRes<'a, &'static str> {
    let mut i = it.clone();
    let mut s_it = tg.chars();
    while let Some(c) = s_it.next() {
        let i2 = it.clone();
        match i.next() {
            None => return i2.err_r(tg),
            Some(ic) => {
                if ic != c {
                    return i2.err_r(tg);
                }
            }
        }
    }
    Ok((i, tg, None))
}

pub fn eoi<'a>(i: &LCChars<'a>) -> ParseRes<'a, ()> {
    let mut r = i.clone();
    if r.next() == None {
        return Ok((r, (), None));
    }
    i.err_ex_r(Expected::EOI)
}

pub fn to_end() -> impl Parser<Out = ()> {
    WS.star().ig_then(eoi)
}

pub struct Peek<P: Parser> {
    p: P,
}

impl<P: Parser> Parser for Peek<P> {
    type Out = P::Out;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, P::Out> {
        let (_, v, c) = self.p.parse(it)?;
        Ok((it.clone(), v, c))
    }
}

pub fn peek<P: Parser>(p: P) -> Peek<P> {
    Peek { p }
}

pub struct CharsUntil<A: Parser<Out = char>, B: Parser> {
    a: A,
    b: B,
}

impl<A: Parser<Out = char>, B: Parser> Parser for CharsUntil<A, B> {
    type Out = (String, B::Out);
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
        let mut res = String::new();
        let mut it = it.clone();
        loop {
            //let it2 = it.clone();
            if let Ok((i, bv, c1)) = self.b.parse(&it) {
                return Ok((i, (res, bv), c1));
            }
            it = match self.a.parse(&it) {
                Ok((i, c, _)) => {
                    res.push(c);
                    i
                }
                Err(e) => return Err(e),
            };
        }
    }
}

pub fn chars_until<A: Parser<Out = char>, B: Parser>(a: A, b: B) -> CharsUntil<A, B> {
    CharsUntil { a, b }
}

pub struct StringRepeat<A: Parser<Out = AV>, AV: Into<String> + AsRef<str>> {
    a: A,
    min: usize,
}

impl<A: Parser<Out = AV>, AV: Into<String> + AsRef<str>> Parser for StringRepeat<A, AV> {
    type Out = String;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, String> {
        let (mut nit, mut res) = match self.a.parse(it) {
            Ok((it2, ss, _)) => (it2, ss.into()),
            Err(e) => {
                if self.min == 0 {
                    return Ok((it.clone(), String::new(), it.err_p_o(&self.a)));
                } else {
                    return Err(e);
                }
            }
        };
        let mut done = 1;
        loop {
            match self.a.parse(&nit) {
                Ok((it, r, _)) => {
                    res.push_str(r.as_ref());
                    nit = it;
                }
                Err(e) => {
                    if done < self.min {
                        return Err(e);
                    } else {
                        let eo = nit.err_p_o(&self.a);
                        return Ok((nit, res, eo));
                    }
                }
            }
            done += 1;
        }
    }
}

pub fn string_repeat<A: Parser<Out = AV>, AV: Into<String> + AsRef<str>>(
    a: A,
    min: usize,
) -> StringRepeat<A, AV> {
    StringRepeat { a, min }
}
