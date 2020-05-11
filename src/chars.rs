use crate::err::*;
use crate::iter::*;
use crate::ptrait::*;
//use crate::reader::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Expected {
    Unknown,
    Char(char),
    CharIn(&'static str),
    OneOf(Vec<Expected>),
}

pub trait CharBool: Sized {
    fn char_bool(&self, c: char) -> bool;
    fn expected(&self) -> Expected {
        return Expected::Unknown;
    }
    fn one(self) -> OneChar<Self> {
        OneChar { cb: self }
    }
}

pub struct Alpha;
pub fn is_alpha(c: char) -> bool {
    (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z')
}
impl CharBool for Alpha {
    fn char_bool(&self, c: char) -> bool {
        is_alpha(c)
    }
    fn expected(&self) -> Expected {
        Expected::CharIn("[a-z][A-Z]")
    }
}
pub struct NumDigit;
pub fn is_num(c: char) -> bool {
    c >= '0' && c <= '9'
}
impl CharBool for NumDigit {
    fn char_bool(&self, c: char) -> bool {
        is_num(c)
    }
    fn expected(&self) -> Expected {
        Expected::CharIn("[0-9]")
    }
}

impl CharBool for char {
    fn char_bool(&self, c: char) -> bool {
        *self == c
    }
}

impl CharBool for &'static str {
    fn char_bool(&self, c: char) -> bool {
        self.contains(c)
    }
}

impl<F: Fn(char) -> bool> CharBool for F {
    fn char_bool(&self, c: char) -> bool {
        (self)(c)
    }
}

impl<A: CharBool, B: CharBool> CharBool for (A, B) {
    fn char_bool(&self, c: char) -> bool {
        self.0.char_bool(c) || self.1.char_bool(c)
    }
}

impl<A: CharBool, B: CharBool, C: CharBool> CharBool for (A, B, C) {
    fn char_bool(&self, c: char) -> bool {
        self.0.char_bool(c) || self.1.char_bool(c) || self.2.char_bool(c)
    }
}
impl<A, B, C, D> CharBool for (A, B, C, D)
where
    A: CharBool,
    B: CharBool,
    C: CharBool,
    D: CharBool,
{
    fn char_bool(&self, c: char) -> bool {
        self.0.char_bool(c) || self.1.char_bool(c) || self.2.char_bool(c) || self.3.char_bool(c)
    }
}

impl<A, B, C, D, E> CharBool for (A, B, C, D, E)
where
    A: CharBool,
    B: CharBool,
    C: CharBool,
    D: CharBool,
    E: CharBool,
{
    fn char_bool(&self, c: char) -> bool {
        self.0.char_bool(c)
            || self.1.char_bool(c)
            || self.2.char_bool(c)
            || self.3.char_bool(c)
            || self.4.char_bool(c)
    }
}

impl<A, B, C, D, E, F> CharBool for (A, B, C, D, E, F)
where
    A: CharBool,
    B: CharBool,
    C: CharBool,
    D: CharBool,
    E: CharBool,
    F: CharBool,
{
    fn char_bool(&self, c: char) -> bool {
        self.0.char_bool(c)
            || self.1.char_bool(c)
            || self.2.char_bool(c)
            || self.3.char_bool(c)
            || self.4.char_bool(c)
            || self.5.char_bool(c)
    }
}

pub fn do_one_char<'a, CB: CharBool>(i: &LCChars<'a>, cb: &CB) -> ParseRes<'a, char> {
    let mut i2 = i.clone();
    let ic = i2.next().ok_or(i2.err_c(ECode::EOF))?;
    if cb.char_bool(ic) {
        Ok((i2, ic))
    } else {
        i2.err_cr(ECode::CharExpected(cb.expected(), Some(ic)))
    }
}

pub struct OneChar<CB: CharBool> {
    cb: CB,
}

impl<CB: CharBool> Parser<char> for OneChar<CB> {
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, char> {
        do_one_char(it, &self.cb)
    }
}

pub fn one_char<C: CharBool>(cb: C) -> OneChar<C> {
    OneChar { cb }
}

#[cfg(test)]
pub mod test {
    use super::*;
    #[test]
    pub fn test_alpha_works_as_struct() {
        assert_eq!(Alpha.char_bool('a'), true)
    }
}
