//! CharBool is the main trait for checking if a character is in a set.
//! There are several helper methods to turn these into Parsers such as
//! * star plus min_n exact, skip_plus skip_exact
//!
//! ```rust
//! use gobble::*;
//! assert_eq!(Alpha.min_n(4).parse_s("hello_"),Ok("hello".to_string()));
//! assert!(Alpha.min_n(6).parse_s("hello_").is_err());
//!
//! assert_eq!((Alpha,NumDigit).star().parse_s("root123r(sds)"),Ok("root123r".to_string()));
//! ```
//!

use crate::err::*;
use crate::iter::*;
use crate::ptrait::*;
use crate::skip;

//use crate::reader::*;

pub trait CharBool: Sized {
    fn char_bool(&self, c: char) -> bool;
    fn expected(&self) -> Expected {
        Expected::Str(std::any::type_name::<Self>())
    }
    fn one(self) -> OneChar<Self> {
        OneChar { cb: self }
    }
    #[deprecated(since = "0.4.0", note = "Use 'star' instead")]
    fn any(self) -> CharStar<Self> {
        CharStar { cb: self }
    }
    fn star(self) -> CharStar<Self> {
        CharStar { cb: self }
    }
    /// min_n not min to avoid ambiguity with std::cmp::Ord
    fn min_n(self, min: usize) -> CharMin<Self> {
        CharMin { cb: self, min }
    }

    fn plus(self) -> CharPlus<Self> {
        CharPlus { cb: self }
    }
    fn skip_star(self) -> skip::CharSkip<Self> {
        skip::CharSkip { cb: self }
    }

    fn skip_plus(self) -> skip::CharSkipPlus<Self> {
        skip::CharSkipPlus { cb: self }
    }

    fn skip_exact(self, n: usize) -> skip::CharSkipExact<Self> {
        skip::CharSkipExact { cb: self, n }
    }
    ///```rust
    /// use gobble::*;
    /// assert_eq!(
    ///     Any.except("_").min_n(4).parse_s("asedf_wes"),
    ///     Ok("asedf".to_string())
    ///     );
    ///```
    fn except<E: CharBool>(self, e: E) -> CharsExcept<Self, E> {
        CharsExcept { a: self, e }
    }

    fn exact(self, n: usize) -> CharExact<Self> {
        CharExact { a: self, n }
    }
}

pub fn is_alpha(c: char) -> bool {
    (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z')
}
char_bool!(Alpha, is_alpha);

pub fn is_num(c: char) -> bool {
    c >= '0' && c <= '9'
}
char_bool!(NumDigit, is_num);

char_bool!(Any, |_| true);

pub fn is_hex(c: char) -> bool {
    is_num(c) || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F')
}
char_bool!(HexDigit, is_hex);

char_bool!(WS, "\t ");
char_bool!(WSL, " \t\n\r");

impl CharBool for char {
    fn char_bool(&self, c: char) -> bool {
        *self == c
    }
    fn expected(&self) -> Expected {
        Expected::Char(*self)
    }
}

impl CharBool for &'static str {
    fn char_bool(&self, c: char) -> bool {
        self.contains(c)
    }
    fn expected(&self) -> Expected {
        Expected::CharIn(self)
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
    fn expected(&self) -> Expected {
        Expected::OneOf(vec![self.0.expected(), self.1.expected()])
    }
}

impl<A: CharBool, B: CharBool, C: CharBool> CharBool for (A, B, C) {
    fn char_bool(&self, c: char) -> bool {
        self.0.char_bool(c) || self.1.char_bool(c) || self.2.char_bool(c)
    }
    fn expected(&self) -> Expected {
        Expected::OneOf(vec![
            self.0.expected(),
            self.1.expected(),
            self.2.expected(),
        ])
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
    fn expected(&self) -> Expected {
        Expected::OneOf(vec![
            self.0.expected(),
            self.1.expected(),
            self.2.expected(),
            self.3.expected(),
        ])
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
    fn expected(&self) -> Expected {
        Expected::OneOf(vec![
            self.0.expected(),
            self.1.expected(),
            self.2.expected(),
            self.3.expected(),
            self.4.expected(),
        ])
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
    fn expected(&self) -> Expected {
        Expected::OneOf(vec![
            self.0.expected(),
            self.1.expected(),
            self.2.expected(),
            self.3.expected(),
            self.4.expected(),
            self.5.expected(),
        ])
    }
}

pub fn do_one_char<'a, CB: CharBool>(i: &LCChars<'a>, cb: &CB) -> ParseRes<'a, char> {
    let mut i2 = i.clone();
    let ic = i2.next().ok_or(i2.err_ex(cb.expected()))?;
    if cb.char_bool(ic) {
        Ok((i2, ic, None))
    } else {
        i.err_ex_r(cb.expected())
    }
}

pub struct OneChar<CB: CharBool> {
    cb: CB,
}

impl<CB: CharBool> Parser for OneChar<CB> {
    type Out = char;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, char> {
        do_one_char(it, &self.cb)
    }
}

pub fn one_char<C: CharBool>(cb: C) -> OneChar<C> {
    OneChar { cb }
}

pub fn do_chars<'a, CB: CharBool>(
    it: &LCChars<'a>,
    cb: &CB,
    min: usize,
    exact: bool,
) -> ParseRes<'a, String> {
    let mut res = String::new();
    let mut it = it.clone();
    loop {
        let it2 = it.clone();
        match it.next() {
            Some(c) if cb.char_bool(c) => {
                res.push(c);
            }
            Some(_) | None => {
                if res.len() >= min {
                    let eo = it2.err_cb_o(cb);
                    return Ok((it2, res, eo));
                } else {
                    return it2.err_ex_r(cb.expected());
                }
            }
        }
        if res.len() == min && exact {
            return Ok((it, res, None));
        }
    }
}
#[derive(Clone)]
pub struct CharStar<C: CharBool> {
    cb: C,
}

impl<CB: CharBool> Parser for CharStar<CB> {
    type Out = String;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, String> {
        do_chars(it, &self.cb, 0, false)
    }
}

#[derive(Clone)]
pub struct CharPlus<C: CharBool> {
    cb: C,
}

impl<CB: CharBool> Parser for CharPlus<CB> {
    type Out = String;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, String> {
        do_chars(it, &self.cb, 1, false)
    }
}

pub struct CharsExcept<A: CharBool, E: CharBool> {
    a: A,
    e: E,
}

impl<A: CharBool, E: CharBool> CharBool for CharsExcept<A, E> {
    fn char_bool(&self, c: char) -> bool {
        self.a.char_bool(c) && !self.e.char_bool(c)
    }
    fn expected(&self) -> Expected {
        self.a.expected().or(Expected::except(self.e.expected()))
    }
}

#[derive(Clone)]
pub struct CharExact<A: CharBool> {
    a: A,
    n: usize,
}

impl<A: CharBool> Parser for CharExact<A> {
    type Out = String;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, String> {
        do_chars(it, &self.a, self.n, true)
    }
}

#[derive(Clone)]
pub struct CharMin<A: CharBool> {
    cb: A,
    min: usize,
}

impl<A: CharBool> Parser for CharMin<A> {
    type Out = String;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, String> {
        do_chars(it, &self.cb, self.min, false)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    #[test]
    pub fn test_alpha_works_as_struct() {
        assert_eq!(Alpha.char_bool('a'), true)
    }
}
