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

///A parser that ignores the result of another parser and returns the string between the
///previous
pub fn string<A: Parser>(a: A) -> AsString<A> {
    AsString { a }
}

pub struct AsString<A: Parser> {
    a: A,
}

impl<A: Parser> Parser for AsString<A> {
    type Out = String;
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, String> {
        self.a
            .parse(it)
            .map(|(nit, _, ct)| match (it.index(), nit.index()) {
                (Some(s), Some(f)) => (nit, it.as_str()[0..(f - s)].to_string(), ct),
                _ => (nit, it.as_str().to_string(), ct),
            })
    }
}
