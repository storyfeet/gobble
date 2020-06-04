use crate::chars::*;
use crate::err::ECode;
use crate::iter::LCChars;
use crate::ptrait::{ParseRes, Parser};
use crate::skip::skip_while;

pub struct StrPos {
    start: usize,
    fin: Option<usize>,
}

impl StrPos {
    ///This version assumes that this is the string it came from
    pub fn on_str<'a>(&self, s: &'a str) -> &'a str {
        match self.fin {
            Some(f) => &s[self.start..f],
            None => &s[self.start..],
        }
    }
}

pub struct SPos<P: Parser> {
    p: P,
}

impl<P: Parser> Parser for SPos<P> {
    type Out = StrPos;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, StrPos> {
        let start = it.index().ok_or(it.err_c(ECode::EOF))?;
        let (rit, _) = self.p.parse(it)?;
        let fin = rit.index();
        Ok((rit, StrPos { start, fin }))
    }
}

pub fn str_pos<P: Parser>(p: P) -> SPos<P> {
    SPos { p }
}
pub fn ws_<P: Parser>(p: P) -> impl Parser<Out = P::Out> {
    WS.skip().ig_then(p)
}

///Convenience wrapper to say allow whitespace around whatever I'm parsing.
pub fn s_<P: Parser>(p: P) -> impl Parser<Out = P::Out> {
    crate::combi::wrap(WS.skip(), p)
}

///Take at least n white space characters
#[deprecated(since = "0.3.0", note = "use WS.any() or WS.min(n) instead")]
pub fn ws(min: usize) -> impl Parser<Out = ()> {
    skip_while(
        |c| match c {
            ' ' | '\t' | '\r' => true,
            _ => false,
        },
        min,
    )
}

impl<P: Parser> Parser for KeyWord<P> {
    type Out = P::Out;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, P::Out> {
        let (t2, r) = self.p.parse(it)?;
        match t2.clone().next() {
            Some(c) => {
                if (Alpha, NumDigit, '_').char_bool(c) {
                    t2.err_cr(ECode::SMess("Not Keyword"))
                } else {
                    Ok((t2, r))
                }
            }
            None => Ok((t2, r)),
        }
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
        match i.next() {
            None => return i.err_cr(ECode::Tag(tg)),
            Some(ic) => {
                if ic != c {
                    return i.err_cr(ECode::Tag(tg));
                }
            }
        }
    }
    Ok((i, tg))
}

impl<F> Parser for Take<F>
where
    F: CharBool,
{
    type Out = ();
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, ()> {
        let mut n = 0;
        let mut i = i.clone();
        let mut i2 = i.clone();
        while let Some(c) = i.next() {
            if !self.f.char_bool(c) {
                if n < self.min {
                    return i.err_r("not enough to take");
                }
                return Ok((i2, ()));
            }
            n += 1;
            i2.next();
        }
        if n < self.min {
            return i.err_r("End of str before end of take");
        }
        Ok((i2, ()))
    }
}

#[derive(Clone)]
pub struct Take<F> {
    f: F,
    min: usize,
}

pub fn take<F>(f: F, min: usize) -> Take<F>
where
    F: Fn(char) -> bool,
{
    Take { f, min }
}

pub fn eoi<'a>(i: &LCChars<'a>) -> ParseRes<'a, ()> {
    let mut r = i.clone();
    if r.next() == None {
        return Ok((r, ()));
    }
    i.err_r("Still More Input")
}

pub fn to_end() -> impl Parser<Out = ()> {
    WS.any().ig_then(eoi)
}

pub struct TakeN {
    n: usize,
}

impl Parser for TakeN {
    type Out = String;
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, String> {
        let mut res = String::new();
        let mut it = i.clone();
        for _ in 0..self.n {
            res.push(it.next().ok_or(it.err_c(ECode::EOF))?);
        }
        Ok((it, res))
    }
}
pub fn take_n(n: usize) -> TakeN {
    TakeN { n }
}

pub fn take_char<'a>(i: &LCChars<'a>) -> ParseRes<'a, char> {
    let mut ri = i.clone();
    let c = ri.next().ok_or(ri.err_c(ECode::EOF))?;
    Ok((ri, c))
}

pub struct Peek<P: Parser> {
    p: P,
}

impl<P: Parser> Parser for Peek<P> {
    type Out = P::Out;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, P::Out> {
        let (_, v) = self.p.parse(it)?;
        Ok((it.clone(), v))
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
            if let Ok((i, bv)) = self.b.parse(&it) {
                return Ok((i, (res, bv)));
            }
            it = match self.a.parse(&it) {
                Ok((i, c)) => {
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
            Ok((it2, ss)) => (it2, ss.into()),
            Err(e) => {
                if self.min == 0 {
                    return Ok((it.clone(), String::new()));
                } else {
                    return Err(e);
                }
            }
        };
        let mut done = 1;
        loop {
            match self.a.parse(&nit) {
                Ok((it, r)) => {
                    res.push_str(r.as_ref());
                    nit = it;
                }
                Err(e) => {
                    if done < self.min {
                        return Err(e);
                    } else {
                        return Ok((nit, res));
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
