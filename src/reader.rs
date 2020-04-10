use crate::err::ECode;
use crate::iter::LCChars;
use crate::ptrait::{ParseRes, Parser};
use std::collections::BTreeMap;
//use std::iter::FromIterator;

#[derive(Clone)]
pub struct Read<F> {
    f: F,
    min: usize,
}

//pub fn ident(Str)

///This is the return result for any function wishing to work with Read
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

pub fn is_num(c: char) -> bool {
    c >= '0' && c <= '9'
}
pub fn is_alpha(c: char) -> bool {
    (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || c == '_'
}

pub fn is_alpha_num(c: char) -> bool {
    is_num(c) || is_alpha(c)
}

pub fn read_fs<F>(f: F, min: usize) -> Read<impl Fn(String, char) -> ReadResult<String>>
where
    F: Fn(char) -> bool,
{
    let fr = move |mut v: String, c: char| {
        if f(c) {
            v.push(c);
            if v.len() < min {
                return ReadResult::Req(v);
            }
            return ReadResult::Cont(v);
        }
        if v.len() < min {
            return ReadResult::Err(ECode::SMess("not enough to read_f"));
        }
        return ReadResult::Back(v);
    };
    read(fr, min)
}

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
pub fn tag(s: &'static str) -> Tag {
    Tag { s }
}

///Conveniece wrapper for tag, often you want to allow whitespace
/// around a tag of some kind
pub fn s_tag(s: &'static str) -> impl Parser<&'static str> {
    s_(tag(s))
}

///Convenience wrapper to say allow whitespace around whatever I'm parsing.
pub fn s_<P: Parser<V>, V>(p: P) -> impl Parser<V> {
    crate::combi::wrap(ws(0), p)
}

///Take at least n white space characters
pub fn ws(min: usize) -> impl Parser<()> {
    take(
        |c| match c {
            ' ' | '\t' | '\r' => true,
            _ => false,
        },
        min,
    )
}

impl Parser<&'static str> for Tag {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, &'static str> {
        let mut i = i.clone();
        let mut s_it = self.s.chars();
        while let Some(c) = s_it.next() {
            match i.next() {
                None => return i.err_cr(ECode::Tag(self.s)),
                Some(ic) => {
                    if ic != c {
                        return i.err_cr(ECode::Tag(self.s));
                    }
                }
            }
        }
        Ok((i, self.s))
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

/// An commonly used form for quoted strings
pub fn common_str() -> impl Parser<String> {
    tag("\"").ig_then(
        esc('\"', '\\')
            .e_map('t', '\t')
            .e_map('n', '\n')
            .e_map('r', '\r'),
    )
}

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
    ws(0).then_ig(eoi)
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    pub fn test_escape() {
        let s = r#""he\tl\\lo to you\" "pop"#;
        let p = tag("\"").ig_then(esc('\"', '\\').e_map('t', '\t'));
        let r = p.parse_s(s).unwrap();
        assert_eq!(r, "he\tl\\lo to you\" ");
    }
}
