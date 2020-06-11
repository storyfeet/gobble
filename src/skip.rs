use crate::chars::*;
use crate::iter::LCChars;
use crate::ptrait::{ParseRes, Parser};

pub fn do_skip_c<'a, CB: CharBool>(
    it: &LCChars<'a>,
    cb: &CB,
    min: usize,
    exact: bool,
) -> ParseRes<'a, ()> {
    let mut it = it.clone();
    let mut done = 0;
    loop {
        let it2 = it.clone();
        match it.next() {
            Some(c) if cb.char_bool(c) => done += 1,
            Some(_) | None => {
                if done >= min {
                    let eo = it2.err_cb_o(cb);
                    return Ok((it2, (), eo));
                } else {
                    return it2.err_ex_r(cb.expected());
                }
            }
        }
        if done == min && exact {
            return Ok((it, (), None));
        }
    }
}

pub fn do_skip_p<'a, P: Parser>(
    it: &LCChars<'a>,
    p: &P,
    min: usize,
    exact: bool,
) -> ParseRes<'a, ()> {
    let mut it = it.clone();
    let mut done = 0;
    loop {
        let it2 = it.clone();
        it = match p.parse(&it) {
            Ok((nit, _, _)) => {
                done += 1;
                nit
            }
            Err(e) => {
                if done >= min {
                    return Ok((it2, (), Some(e)));
                } else {
                    return Err(e);
                }
            }
        };
        if done == min && exact {
            return Ok((it, (), None));
        }
    }
}

pub struct CharSkip<CB: CharBool> {
    pub cb: CB,
}

impl<CB: CharBool> Parser for CharSkip<CB> {
    type Out = ();
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, ()> {
        do_skip_c(it, &self.cb, 0, false)
    }
}

#[derive(Clone)]
pub struct CharSkipPlus<CB: CharBool> {
    pub cb: CB,
}

impl<CB: CharBool> Parser for CharSkipPlus<CB> {
    type Out = ();
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, ()> {
        do_skip_c(i, &self.cb, 1, false)
    }
}
#[derive(Clone)]
pub struct CharSkipExact<CB: CharBool> {
    pub cb: CB,
    pub n: usize,
}

impl<CB: CharBool> Parser for CharSkipExact<CB> {
    type Out = ();
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, ()> {
        do_skip_c(i, &self.cb, self.n, true)
    }
}

#[derive(Clone)]
pub struct PSkipStar<A: Parser> {
    pub a: A,
}
impl<A: Parser> Parser for PSkipStar<A> {
    type Out = ();
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, ()> {
        do_skip_p(it, &self.a, 0, false)
    }
}

#[derive(Clone)]
pub struct PSkipPlus<A: Parser> {
    pub a: A,
}
impl<A: Parser> Parser for PSkipPlus<A> {
    type Out = ();
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, ()> {
        do_skip_p(it, &self.a, 1, false)
    }
}

#[derive(Clone)]
pub struct PSkipExact<A: Parser> {
    pub a: A,
    pub n: usize,
}
impl<A: Parser> Parser for PSkipExact<A> {
    type Out = ();
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, ()> {
        do_skip_p(it, &self.a, self.n, true)
    }
}

pub fn skip_2_star<A: Parser, B: Parser>(a: A, b: B) -> Skip2Star<A, B> {
    Skip2Star { a, b }
}

pub struct Skip2Star<A: Parser, B: Parser> {
    a: A,
    b: B,
}

impl<A: Parser, B: Parser> Parser for Skip2Star<A, B> {
    type Out = ();
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, ()> {
        let mut it = it.clone();
        loop {
            if let Ok((nit, _, _)) = self.a.parse(&it) {
                it = nit;
            } else if let Ok((nit, _, _)) = self.b.parse(&it) {
                it = nit;
            } else {
                return Ok((it, (), None));
            }
        }
    }
}
