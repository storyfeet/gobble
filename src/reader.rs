use crate::basic::{ParseRes, Parser};
use crate::err::ParseError;
//use std::iter::FromIterator;

pub struct Read<F> {
    f: F,
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

impl<F, I, C, V: Default> Parser<I, V> for Read<F>
where
    F: Fn(V, C) -> ReadResult<V>,
    I: Clone + Iterator<Item = C>,
{
    fn parse(&self, i: &I) -> ParseRes<I, V> {
        let mut res = V::default();
        let mut i = i.clone();
        let mut i2 = i.clone();
        let mut req = true;
        while let Some(p) = i.next() {
            match (self.f)(res, p) {
                ReadResult::Done(v) => return Ok((i, v)),
                ReadResult::Back(v) => return Ok((i2, v)),
                ReadResult::Cont(v) => {
                    res = v;
                    req = false
                }
                ReadResult::Req(v) => {
                    res = v;
                    req = true
                }
                ReadResult::Err(e) => return Err(e),
            }
            i2 = i.clone();
        }
        if req {
            return Err(ParseError::new("Still more required for Read::parse", 0));
        }
        Ok((i, res))
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

pub fn read_f<F, C, V>(f: F, min: usize) -> Read<impl Fn(V, C) -> ReadResult<V>>
where
    F: Fn(&C) -> bool,
    V: std::iter::Extend<C> + Len,
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
    Read { f: fr }
}

pub fn read<F, V, C>(f: F) -> Read<F>
where
    F: Fn(V, C) -> ReadResult<V>,
{
    Read { f }
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
