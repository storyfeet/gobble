use crate::iter::LCChars;
use crate::ptrait::*;
use std::marker::PhantomData;

pub struct String2P<A: Parser, B: Parser> {
    a: A,
    b: B,
}

impl<A, B> Parser for String2P<A, B>
where
    A: Parser,
    B: Parser,
    A::Out: Into<String>,
    B::Out: AsRef<str>,
{
    type Out = String;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, String> {
        let (it2, av) = self.a.parse(it)?;
        let (itres, bv) = self.b.parse(&it2)?;
        let mut s: String = av.into();
        s.push_str(bv.as_ref());
        Ok((itres, s))
    }
}

pub fn string_2_parts<A, B>(a: A, b: B) -> String2P<A, B>
where
    A: Parser,
    B: Parser,
    A::Out: Into<String>,
    B::Out: AsRef<str>,
{
    String2P {
        a,
        b,
        pha: PhantomData,
        phb: PhantomData,
    }
}
