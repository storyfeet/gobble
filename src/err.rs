use std::cmp::Ordering;
//use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use thiserror::*;

#[derive(Debug, PartialEq, Eq, Clone, Error, Hash)]
pub enum Expected {
    EOI,
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
            EOI => write!(f, "EOI"),
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
            Expected::WS => true,
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

#[derive(Clone, PartialEq, Eq, Error, Hash)]
pub struct PErr<'a> {
    pub exp: Expected,
    pub found: &'a str,
    pub index: Option<usize>,
    pub line: usize,
    pub col: usize,
    pub is_brk: bool,
    pub child: Option<Box<PErr<'a>>>,
}

fn compare_index(a: &Option<usize>, b: &Option<usize>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        _ => Ordering::Equal,
    }
}

fn join_children<'a>(a: Option<Box<PErr<'a>>>, b: Option<Box<PErr<'a>>>) -> Option<Box<PErr<'a>>> {
    match (a, b) {
        (Some(ac), Some(bc)) => Some(Box::new((*ac).join(*bc))),
        (None, b) => b,
        (a, None) => a,
    }
}

impl<'a> PErr<'a> {
    pub fn join(mut self, mut b: Self) -> Self {
        match compare_index(&self.index, &b.index) {
            Ordering::Greater => {
                self.child = join_children(self.child, Some(Box::new(b)));
                self
            }
            Ordering::Less => {
                b.child = join_children(b.child, Some(Box::new(self)));
                b
            }
            _ => {
                self.child = join_children(self.child, b.child);
                self.exp = match (self.exp, b.exp) {
                    (Expected::OneOf(mut ae), Expected::OneOf(be)) => {
                        ae.extend(be);
                        Expected::OneOf(ae)
                    }
                    (Expected::OneOf(mut ae), b) | (b, Expected::OneOf(mut ae)) => {
                        if b != Expected::WS {
                            ae.push(b);
                        }
                        Expected::OneOf(ae)
                    }
                    (a, b) => Expected::OneOf(vec![a, b]),
                };
                self
            }
        }
    }

    pub fn join_op(self, b: Option<Self>) -> Self {
        match b {
            Some(v) => self.join(v),
            None => self,
        }
    }

    pub fn strung(self) -> StrungError {
        StrungError {
            exp: self.exp,
            found: self.found.to_string(),
            line: self.line,
            col: self.col,
            index: self.index,
            is_brk: self.is_brk,
            child: self.child.map(|v| Box::new((*v).strung())),
        }
    }

    pub fn wrap(mut self, ne: Self) -> Self {
        match self.child {
            Some(c) => self.child = Some(Box::new(c.wrap(ne))),
            None => self.child = Some(Box::new(ne)),
        }
        self
    }

    /*pub fn new_exp(mut self, nexp: Expected) -> ParseError {
        self.exp = nexp;
        self
    }*/

    pub fn brk(mut self) -> Self {
        self.is_brk = true;
        self
    }

    pub fn set_brk(mut self, b: bool) -> Self {
        self.is_brk = b;
        self
    }
}
impl<'a> fmt::Debug for PErr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let i_str = match self.index {
            Some(n) => n.to_string(),
            None => "EOI".to_string(),
        };
        write!(
            f,
            "Expected '{}', Found '{}', at (i={},l={},c={})\n",
            self.exp, self.found, i_str, self.line, self.col
        )
    }
}
impl<'a> fmt::Display for PErr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let i_str = match self.index {
            Some(n) => n.to_string(),
            None => "EOI".to_string(),
        };
        write!(
            f,
            "Expected '{}', Found '{}', at (i={},l={},c={})\n",
            self.exp, self.found, i_str, self.line, self.col
        )?;
        if let Some(ref c) = self.child {
            write!(f, "\t{}", c)?
        }
        Ok(())
    }
}

//The StrungError has the String it was parsed from attached to it.
#[derive(Clone, Error, PartialEq, Eq, Hash)]
pub struct StrungError {
    pub exp: Expected,
    pub found: String,
    pub index: Option<usize>,
    pub line: usize,
    pub col: usize,
    pub is_brk: bool,
    pub child: Option<Box<StrungError>>,
}

impl fmt::Debug for StrungError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let i_str = match self.index {
            Some(n) => n.to_string(),
            None => "EOI".to_string(),
        };
        write!(
            f,
            "Expected '{}', Found '{}', at (i={},l={},c={})\n",
            self.exp, self.found, i_str, self.line, self.col
        )
    }
}
impl fmt::Display for StrungError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let i_str = match self.index {
            Some(n) => n.to_string(),
            None => "EOI".to_string(),
        };
        write!(
            f,
            "Expected '{}', Found '{}', at (i={},l={},c={})\n",
            self.exp, self.found, i_str, self.line, self.col
        )?;
        if let Some(ref c) = self.child {
            write!(f, "\t{}", c)?
        }
        Ok(())
    }
}
