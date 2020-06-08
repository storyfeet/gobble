//use failure_derive::*;
use std::cmp::Ordering;
//use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use thiserror::*;

#[derive(Debug, PartialEq, Clone, Error)]
pub enum Expected {
    Unknown,
    Char(char),
    WS,
    CharIn(&'static str),
    ObOn(&'static str, &'static str),
    Str(&'static str),
    OneOf(Vec<Expected>),
    Except(Box<Expected>),
}

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Expected::*;
        match self {
            Unknown => write!(f, "Unknown"),
            Char(c) => write!(f, "Char:\"{}\"", c),
            WS => write!(f, "WhiteSpace"),
            CharIn(s) => write!(f, "Char In {:?}", s),
            ObOn(p, o) => write!(f, "{} on parser {}", o, p),
            Str(s) => write!(f, "{:?}", s),
            OneOf(v) => {
                write!(f, "one of:(")?;
                for e in v {
                    write!(f, "{} ", e)?;
                }
                write!(f, ")")
            }
            Except(e) => write!(f, " Except : ({})", e),
        }
    }
}

impl Expected {
    pub fn or(self, b: Self) -> Self {
        match self {
            Expected::OneOf(mut v) => {
                v.push(b);
                Expected::OneOf(v)
            }
            v => Expected::OneOf(vec![v, b]),
        }
    }

    pub fn except(a: Self) -> Self {
        Expected::Except(Box::new(a))
    }
    pub fn is_nil(&self) -> bool {
        match self {
            Expected::Unknown | Expected::WS => true,
            _ => false,
        }
    }

    pub fn first(a: Self, b: Self) -> Self {
        match a.is_nil() {
            true => b,
            false => a,
        }
    }
}

pub fn longer(mut a: ParseError, b: ParseError) -> ParseError {
    match a.partial_cmp(&b) {
        Some(Ordering::Greater) => a,
        Some(Ordering::Less) => b,
        _ => {
            a.exp = Expected::OneOf(vec![a.exp, b.exp]);
            a
        }
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
#[error("Parse Error {} at  {},{}: Expected {}",.is_brk, .line, .col, .exp)]
pub struct ParseError {
    pub exp: Expected,
    pub index: Option<usize>,
    pub line: usize,
    pub col: usize,
    pub is_brk: bool,
    pub child: Option<Box<ParseError>>,
}

impl ParseError {
    pub fn new(s: &'static str, index: Option<usize>, line: usize, col: usize) -> Self {
        ParseError {
            exp: Expected::Str(s),
            index,
            line,
            col,
            is_brk: false,
            child: None,
        }
    }
    pub fn expect(exp: Expected, index: Option<usize>, line: usize, col: usize) -> ParseError {
        ParseError {
            exp,
            index,
            line,
            col,
            is_brk: false,
            child: None,
        }
    }

    pub fn print_on(&self, s: &str) -> String {
        let (pstr, ids): (String, String) = match self.index {
            Some(i) => (s[i..].chars().take(10).collect(), i.to_string()),
            None => ("EOI".to_string(), "EOI".to_string()),
        };

        format!(
            "    At {},{},{}: Expected {} -- Found {:?}\n",
            ids, self.line, self.col, self.exp, pstr,
        )
    }

    pub fn print_on_d(&self, s: &str) -> String {
        let mut res = self.print_on(s);
        if let Some(ref c) = self.child {
            res.push_str(&c.print_on(s));
        }
        res
    }
    pub fn deep_print(&self, s: &str) -> String {
        let mut res = "\nErr :\n".to_string();
        res.push_str(&self.print_on_d(s));
        res
    }

    pub fn wrap(mut self, ne: Self) -> Self {
        match self.child {
            Some(c) => self.child = Some(Box::new(c.wrap(ne))),
            None => self.child = Some(Box::new(ne)),
        }
        self
    }

    pub fn new_exp(mut self, nexp: Expected) -> ParseError {
        self.exp = nexp;
        self
    }

    pub fn is_break(&self) -> bool {
        self.is_brk
    }

    pub fn brk(mut self) -> Self {
        self.is_brk = true;
        self
    }

    pub fn de_brk(mut self) -> Self {
        self.is_brk = false;
        self
    }
}

impl PartialOrd for ParseError {
    fn partial_cmp(&self, b: &Self) -> Option<Ordering> {
        //None means the end of the string
        match (self.index, b.index) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) => Some(Ordering::Greater),
            (Some(_), None) => Some(Ordering::Less),
            (Some(av), Some(bv)) => av.partial_cmp(&bv),
        }
    }
}
