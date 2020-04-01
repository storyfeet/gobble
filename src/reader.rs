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

//pub trait Reader = Fn(V, C) -> ReadResult<V>;
pub enum ReadResult<V> {
    Cont(V),
    Done(V),
    Back(V),
    Req(V),
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

pub trait Len {
    fn get_len(&self) -> usize;
}

impl Len for String {
    fn get_len(&self) -> usize {
        self.len()
    }
}
impl<T> Len for Vec<T> {
    fn get_len(&self) -> usize {
        self.len()
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
            if v.get_len() < min {
                return ReadResult::Req(v);
            }
            return ReadResult::Cont(v);
        }
        if v.get_len() < min {
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
pub fn tag(s: &'static str) -> Tag {
    Tag { s }
}

pub fn s_tag(s: &'static str) -> impl Parser<&'static str> {
    //ws(0).ig_then(tag(s)).then_ig(ws(0))
    crate::combi::wrap(ws(0), tag(s))
}

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
                None => return i.err_r("not long enough for tag"),
                Some(ic) => {
                    if ic != c {
                        return i.err_r("no_match");
                    }
                }
            }
        }
        Ok((i, self.s))
    }
}

//Currently only escapes single chars
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

pub fn esc(close: char, esc: char) -> Escape {
    Escape {
        close,
        esc,
        map: BTreeMap::new(),
    }
}

impl Escape {
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
