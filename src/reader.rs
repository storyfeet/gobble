use crate::err::ParseError;
use crate::ptrait::{ParseRes, Parser};
use std::collections::BTreeMap;
use std::marker::PhantomData;
//use std::iter::FromIterator;

pub struct Read<I, F> {
    f: F,
    min: usize,
    phi: PhantomData<I>,
}

//pub fn ident(Str)

//pub trait Reader = Fn(V, C) -> ReadResult<V>;
pub enum ReadResult<V> {
    Cont(V),
    Done(V),
    Back(V),
    Req(V),
    Err(ParseError),
}

impl<F, I, C, V: Default> Parser<I, V> for Read<I, F>
where
    F: Fn(V, C) -> ReadResult<V>,
    I: Clone + Iterator<Item = C>,
{
    fn parse(&self, i: &I) -> ParseRes<I, V> {
        let min_ok = move |n, i, v| {
            if n >= self.min {
                Ok((i, v))
            } else {
                Err(ParseError::new("not enough read", 0))
            }
        };
        let mut res = V::default();
        let mut i = i.clone();
        let mut i2 = i.clone();
        let mut req = self.min > 0;
        let mut n = 0;
        while let Some(p) = i.next() {
            match (self.f)(res, p) {
                ReadResult::Done(v) => return min_ok(n, i, v), //Ok((i, v)),
                ReadResult::Back(v) => return min_ok(n, i2, v),
                ReadResult::Cont(v) => {
                    res = v;
                    req = false;
                }
                ReadResult::Req(v) => {
                    res = v;
                    req = true
                }
                ReadResult::Err(e) => return Err(e),
            }
            i2 = i.clone();
            n += 1
        }
        if req {
            return Err(ParseError::new("Still more required for Read::parse", 0));
        }
        min_ok(n, i, res)
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

pub fn is_num(c: &char) -> bool {
    *c >= '0' && *c <= '9'
}
pub fn is_alpha(c: &char) -> bool {
    (*c >= 'A' && *c <= 'Z') || (*c >= 'a' && *c <= 'z') || *c == '_'
}

pub fn is_alpha_num(c: &char) -> bool {
    is_num(c) || is_alpha(c)
}

pub fn read_fs<I, F>(f: F, min: usize) -> Read<I, impl Fn(String, char) -> ReadResult<String>>
where
    F: Fn(&char) -> bool,
    I: Iterator<Item = char>,
{
    read_f(f, min)
}

pub fn read_f<I, F, C, V>(f: F, min: usize) -> Read<I, impl Fn(V, C) -> ReadResult<V>>
where
    F: Fn(&C) -> bool,
    V: std::iter::Extend<C> + Len,
    I: Iterator<Item = C>,
{
    let fr = move |mut v: V, c: C| {
        if f(&c) {
            v.extend(Some(c));
            if v.get_len() < min {
                return ReadResult::Req(v);
            }
            return ReadResult::Cont(v);
        }
        if v.get_len() < min {
            return ReadResult::Err(ParseError::new("not enough to read_f", 0));
        }
        return ReadResult::Back(v);
    };
    read(fr, min)
}

pub fn read<I, F, V, C>(f: F, min: usize) -> Read<I, F>
where
    F: Fn(V, C) -> ReadResult<V>,
    I: Iterator<Item = C>,
{
    Read {
        f,
        min,
        phi: PhantomData,
    }
}

pub struct Tag {
    s: &'static str,
}
pub fn tag(s: &'static str) -> Tag {
    Tag { s }
}
impl<I: Iterator<Item = char> + Clone> Parser<I, &'static str> for Tag {
    fn parse(&self, i: &I) -> ParseRes<I, &'static str> {
        let mut i = i.clone();
        let mut s_it = self.s.chars();
        while let Some(c) = s_it.next() {
            match i.next() {
                None => return Err(ParseError::new("not long enough for tag", 0)),
                Some(ic) => {
                    if ic != c {
                        return Err(ParseError::new("no_match", 0));
                    }
                }
            }
        }
        Ok((i, self.s))
    }
}

//Currently only escapes single chars
pub struct Escape {
    esc: char,
    map: BTreeMap<char, char>,
    close: char,
}
impl<I> Parser<I, String> for Escape
where
    I: Iterator<Item = char> + Clone,
{
    fn parse(&self, i: &I) -> ParseRes<I, String> {
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
        Err(ParseError::new("un closed escaper", 0))
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

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    pub fn test_escape() {
        let s = r#""he\tl\\lo to you\" "pop"#;
        let p = tag("\"").ig_then(esc('\"', '\\').e_map('t', '\t'));
        let (_, r) = p.parse(&s.chars()).unwrap();
        assert_eq!(r, "he\tl\\lo to you\" ");
    }
}
