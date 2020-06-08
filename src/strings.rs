use crate::iter::LCChars;
use crate::ptrait::*;

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
        let (it2, av, c1) = self.a.parse(it)?;
        let (itres, bv, c2) = self.b.parse(&it2).map_err(|e| e.cont(c1))?;
        let mut s: String = av.into();
        s.push_str(bv.as_ref());
        Ok((itres, s, c2))
    }
}

pub fn string_2_parts<A, B>(a: A, b: B) -> String2P<A, B>
where
    A: Parser,
    B: Parser,
    A::Out: Into<String>,
    B::Out: AsRef<str>,
{
    String2P { a, b }
}
