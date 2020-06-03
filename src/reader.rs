use crate::chars::*;
use crate::err::ECode;
use crate::iter::LCChars;
use crate::ptrait::{ParseRes, Parser};
use crate::skip::skip_while;
use std::collections::BTreeMap;
use std::marker::PhantomData;
//use std::iter::FromIterator;

#[derive(Clone)]
pub struct Read<F> {
    f: F,
    min: usize,
}

///This is the return result for any function wishing to work with Read
#[derive(Clone)]
pub enum ReadResult<V> {
    ///Keep asking going
    Cont(V),
    ///Stop here
    Done(V),
    ///Stop, we've gone too far
    Back(V),
    ///There is still more required
    Req(V),
    ///There is an unresolveable problem
    Err(ECode),
}

impl<F> Parser<String> for Read<F>
where
    F: Fn(String, char) -> ReadResult<String>,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, String> {
        let mut res = String::new();
        let mut i = i.clone();
        let mut i2 = i.clone();
        let mut req = self.min > 0;
        let mut n = 0;
        while let Some(p) = i.next() {
            match (self.f)(res, p) {
                ReadResult::Done(v) => {
                    return if n >= self.min {
                        Ok((i, v))
                    } else {
                        i.err_r("not enough read")
                    }
                }
                ReadResult::Back(v) => {
                    return if n >= self.min {
                        Ok((i2, v))
                    } else {
                        i.err_r("not enough read")
                    }
                }
                ReadResult::Cont(v) => {
                    res = v;
                    req = false;
                }
                ReadResult::Req(v) => {
                    res = v;
                    req = true
                }
                ReadResult::Err(e) => return i.err_cr(e),
            }
            i2 = i.clone();
            n += 1
        }
        if req {
            return i.err_r("Still more required for Read::parse");
        }
        if n >= self.min {
            Ok((i, res))
        } else {
            i.err_r("not enough read")
        }
    }
}

#[deprecated(
    since = "0.2.1",
    note = "Use a tuple instead: (CharBool,CharBool) implements CharBool"
)]
pub fn or_char<A: Fn(char) -> bool, B: Fn(char) -> bool>(a: A, b: B) -> impl Fn(char) -> bool {
    move |c| a(c) || b(c)
}

/// ```rust
/// use gobble::*;
/// let (rest,name )= s_(read_fs((Alpha,NumDigit),1)).parse_sn("    gobble ").unwrap();
/// assert_eq!(name,"gobble");
/// assert_eq!(rest,"");
/// ```
#[deprecated(since = "0.2.1", note = "Use CharBool::min_n(n) instead")]
pub fn read_fs<CB: CharBool>(cb: CB, min: usize) -> impl Parser<String> {
    cb.min_n(min)
}

#[deprecated(since = "0.2.1", note = "It's really not pretty")]
pub fn read<F>(f: F, min: usize) -> Read<F>
where
    F: Fn(String, char) -> ReadResult<String>,
{
    Read { f, min }
}

#[derive(Clone)]
pub struct Tag {
    s: &'static str,
}

/// Check for a specifig string
/// Returns the string, so that used with "or" you can see which result you got
#[deprecated(since = "0.1.8", note = "Use &str instead")]
pub fn tag(s: &'static str) -> Tag {
    Tag { s }
}

///Conveniece wrapper for tag, often you want to allow whitespace
/// around a tag of some kind
#[deprecated(since = "0.1.8", note = "Use s_(&str) instead")]
pub fn s_tag(s: &'static str) -> impl Parser<&'static str> {
    s_(s)
}

pub fn ws_<P: Parser<V>, V>(p: P) -> impl Parser<V> {
    WS.skip().ig_then(p)
}

///Convenience wrapper to say allow whitespace around whatever I'm parsing.
pub fn s_<P: Parser<V>, V>(p: P) -> impl Parser<V> {
    crate::combi::wrap(WS.skip(), p)
}

///Take at least n white space characters
#[deprecated(since = "0.3.2", note = "use WS.any() or WS.min(n) instead")]
pub fn ws(min: usize) -> impl Parser<()> {
    skip_while(
        |c| match c {
            ' ' | '\t' | '\r' => true,
            _ => false,
        },
        min,
    )
}

impl<P: Parser<V>, V> Parser<V> for KeyWord<P, V> {
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, V> {
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

pub struct KeyWord<P: Parser<V>, V> {
    p: P,
    phv: PhantomData<V>,
}

///```rust
/// use gobble::*;
/// assert_eq!(keyword("let").parse_s("let"), Ok("let"));
/// assert_eq!(keyword("let").parse_s("let "), Ok("let"));
/// assert_eq!(keyword("let").parse_s("let*"), Ok("let"));
/// assert!(keyword("let").parse_s("letl").is_err());
///```
pub fn keyword<P: Parser<V>, V>(p: P) -> KeyWord<P, V> {
    KeyWord {
        p,
        phv: PhantomData,
    }
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

impl Parser<&'static str> for Tag {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, &'static str> {
        do_tag(i, self.s)
    }
}

///A reader for strings, that allows escaping one char and mapping to another char. The
///returned string has already had the escape replace done
#[derive(Clone)]
pub struct Escape {
    esc: char,
    map: BTreeMap<char, char>,
    close: char,
}
impl Parser<String> for Escape {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, String> {
        let mut i = i.clone();
        let mut res = String::new();
        while let Some(c) = i.next() {
            if c == self.close {
                return Ok((i, res));
            }
            if c == self.esc {
                if let Some(c2) = i.next() {
                    match self.map.get(&c2) {
                        Some(cr) => res.push(*cr),
                        None => res.push(c2),
                    }
                }
            } else {
                res.push(c);
            }
        }
        i.err_r("un closed escape")
    }
}
#[deprecated(
    since = "0.1.7",
    note = "see common::common_str() for how to handle escapes"
)]
/// Build an escaper - used to complete a string, you will already have called checked for the
/// opening part of the string
pub fn esc(close: char, esc: char) -> Escape {
    Escape {
        close,
        esc,
        map: BTreeMap::new(),
    }
}

impl Escape {
    /// Add a character to the map, in a builder pattern way.
    /// ```rust
    /// use gobble::*;
    /// let s = tag("\"").ig_then(esc('\"','\\')
    ///     .e_map('t','\t').e_map('p','$'))
    ///     .parse_s(r#""my \t \pstring""#).unwrap();
    /// assert_eq!(s,"my \t $string")
    /// ```
    pub fn e_map(mut self, f: char, t: char) -> Self {
        self.map.insert(f, t);
        self
    }
}

impl<F> Parser<()> for Take<F>
where
    F: Fn(char) -> bool,
{
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, ()> {
        let mut n = 0;
        let mut i = i.clone();
        let mut i2 = i.clone();
        while let Some(c) = i.next() {
            if !(self.f)(c) {
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

pub fn to_end() -> impl Parser<()> {
    WS.any().ig_then(eoi)
}

pub struct TakeN {
    n: usize,
}

impl Parser<String> for TakeN {
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

pub struct Peek<P: Parser<V>, V> {
    p: P,
    phv: PhantomData<V>,
}

impl<P: Parser<V>, V> Parser<V> for Peek<P, V> {
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, V> {
        let (_, v) = self.p.parse(it)?;
        Ok((it.clone(), v))
    }
}

pub fn peek<P: Parser<V>, V>(p: P) -> Peek<P, V> {
    Peek {
        p,
        phv: PhantomData,
    }
}

pub struct CharF<F: Fn(char) -> bool> {
    f: F,
}

impl<F: Fn(char) -> bool> Parser<char> for CharF<F> {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, char> {
        let mut i2 = i.clone();
        match i2.next() {
            Some(c) if (self.f)(c) => Ok((i2, c)),
            v => i2.err_cr(ECode::Char('?', v)),
        }
    }
}

pub fn char_f<F: Fn(char) -> bool>(f: F) -> CharF<F> {
    CharF { f }
}

pub struct CharsUntil<A: Parser<char>, B: Parser<BV>, BV> {
    a: A,
    b: B,
    phb: PhantomData<BV>,
}

impl<A: Parser<char>, B: Parser<BV>, BV> Parser<String> for CharsUntil<A, B, BV> {
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, String> {
        let mut res = String::new();
        let mut it = it.clone();
        loop {
            //let it2 = it.clone();
            if let Ok((i, _)) = self.b.parse(&it) {
                return Ok((i, res));
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

pub fn chars_until<A: Parser<char>, B: Parser<BV>, BV>(a: A, b: B) -> CharsUntil<A, B, BV> {
    CharsUntil {
        a,
        b,
        phb: PhantomData,
    }
}

pub struct StringRepeat<A: Parser<AV>, AV: Into<String> + AsRef<str>> {
    a: A,
    pha: PhantomData<AV>,
    min: usize,
}

impl<A: Parser<AV>, AV: Into<String> + AsRef<str>> Parser<String> for StringRepeat<A, AV> {
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

pub fn string_repeat<A: Parser<AV>, AV: Into<String> + AsRef<str>>(
    a: A,
    min: usize,
) -> StringRepeat<A, AV> {
    StringRepeat {
        a,
        min,
        pha: PhantomData,
    }
}
