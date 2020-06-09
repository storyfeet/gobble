use crate::err::*;
use crate::iter::*;
use crate::ptrait::*;
use crate::skip;

//use crate::reader::*;

pub trait CharBool: Sized {
    fn char_bool(&self, c: char) -> bool;
    fn expected(&self) -> Expected {
        return Expected::Unknown;
    }
    fn one(self) -> OneChar<Self> {
        OneChar { cb: self }
    }
    fn any(self) -> Chars<Self> {
        Chars { cb: self, min: 0 }
    }
    /// min_n not min to avoid ambiguity with std::cmp::Ord
    fn min_n(self, min: usize) -> Chars<Self> {
        Chars { cb: self, min }
    }

    fn skip(self) -> skip::Skip<Self> {
        skip::skip_c(self)
    }

    fn skip_min(self, min: usize) -> skip::SkipMin<Self> {
        skip::skip_while(self, min)
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

    fn exact(self, n: usize) -> CBExact<Self> {
        CBExact { a: self, n }
    }
}

/// [a-z][A-Z]
/// ```rust
/// use gobble::*;
/// assert_eq!(Alpha.min_n(4).parse_s("hello_"),Ok("hello".to_string()));
/// assert!(Alpha.min_n(6).parse_s("hello_").is_err());
/// ```
#[derive(Clone, Copy)]
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

///0..9
#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct Any;
impl CharBool for Any {
    fn char_bool(&self, _: char) -> bool {
        true
    }
    fn expected(&self) -> Expected {
        Expected::CharIn("anything")
    }
}

///a-f,A-F,0-9
#[derive(Clone, Copy)]
pub struct HexDigit;
pub fn is_hex(c: char) -> bool {
    is_num(c) || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F')
}
impl CharBool for HexDigit {
    fn char_bool(&self, c: char) -> bool {
        is_hex(c)
    }
    fn expected(&self) -> Expected {
        Expected::CharIn("[0-9][a-f][A-F]")
    }
}

///Whitespace
pub struct WS;
impl CharBool for WS {
    fn char_bool(&self, c: char) -> bool {
        " \t".char_bool(c)
    }
}
///Whitespace and newlines
pub struct WSL;
impl CharBool for WSL {
    fn char_bool(&self, c: char) -> bool {
        " \t\n\r".char_bool(c)
    }
}

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
        i2.err_ex_r(cb.expected())
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

pub fn do_chars<'a, CB: CharBool>(it: &LCChars<'a>, cb: &CB, min: usize) -> ParseRes<'a, String> {
    let mut res = String::new();
    let mut it = it.clone();
    loop {
        let it2 = it.clone();
        let n = it.next();
        match n {
            Some(c) if cb.char_bool(c) => {
                res.push(c);
            }
            Some(_) | None => {
                if res.len() >= min {
                    return Ok((it2, res, None));
                } else {
                    return it.err_ex_r(cb.expected());
                }
            }
        }
    }
}
pub struct Chars<C: CharBool> {
    min: usize,
    cb: C,
}

impl<CB: CharBool> Parser for Chars<CB> {
    type Out = String;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, String> {
        do_chars(it, &self.cb, self.min)
    }
}
pub fn chars<CB: CharBool>(cb: CB, min: usize) -> Chars<CB> {
    Chars { cb, min }
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

pub struct CBExact<A: CharBool> {
    a: A,
    n: usize,
}

pub fn do_cb_exact<'a, A: CharBool>(it: &LCChars<'a>, a: &A, n: usize) -> ParseRes<'a, String> {
    let mut res = String::new();
    let mut it = it.clone();
    for _ in 0..n {
        let i2 = it.clone();
        match it.next() {
            Some(c) if a.char_bool(c) => res.push(c),
            Some(_) | None => return i2.err_ex_r(a.expected()),
        }
    }
    Ok((it, res, None))
}

impl<A: CharBool> Parser for CBExact<A> {
    type Out = String;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, String> {
        do_cb_exact(it, &self.a, self.n)
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
