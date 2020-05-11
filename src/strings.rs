use crate::iter::LCChars;
use crate::ptrait::*;
use std::marker::PhantomData;

pub struct String2P<A: Parser<AV>, B: Parser<BV>, AV: Into<String>, BV: AsRef<str>> {
    a: A,
    b: B,
    pha: PhantomData<AV>,
    phb: PhantomData<BV>,
}

impl<A, B, AV, BV> Parser<String> for String2P<A, B, AV, BV>
where
    A: Parser<AV>,
    B: Parser<BV>,
    AV: Into<String>,
    BV: AsRef<str>,
{
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, String> {
        let (it2, av) = self.a.parse(it)?;
        let (itres, bv) = self.b.parse(&it2)?;
        let mut s: String = av.into();
        s.push_str(bv.as_ref());
        Ok((itres, s))
    }
}

pub fn string_2_parts<A, B, AV, BV>(a: A, b: B) -> String2P<A, B, AV, BV>
where
    A: Parser<AV>,
    B: Parser<BV>,
    AV: Into<String>,
    BV: AsRef<str>,
{
    String2P {
        a,
        b,
        pha: PhantomData,
        phb: PhantomData,
    }
}
