use crate::iter::*;
use crate::ptrait::*;

impl<A, AV, B, BV> Parser<(AV, BV)> for (A, B)
where
    A: Parser<AV>,
    B: Parser<BV>,
{
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, (AV, BV)> {
        let (it2, av) = self.0.parse(it)?;
        let (it3, bv) = self.1.parse(&it2)?;
        Ok((it3, (av, bv)))
    }
}

impl<A, AV, B, BV, C, CV> Parser<(AV, BV, CV)> for (A, B, C)
where
    A: Parser<AV>,
    B: Parser<BV>,
    C: Parser<CV>,
{
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, (AV, BV, CV)> {
        let (it2, av) = self.0.parse(it)?;
        let (it3, bv) = self.1.parse(&it2)?;
        let (it4, cv) = self.2.parse(&it3)?;
        Ok((it4, (av, bv, cv)))
    }
}

impl<A, AV, B, BV, C, CV, D, DV> Parser<(AV, BV, CV, DV)> for (A, B, C, D)
where
    A: Parser<AV>,
    B: Parser<BV>,
    C: Parser<CV>,
    D: Parser<DV>,
{
    fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, (AV, BV, CV, DV)> {
        let (it2, av) = self.0.parse(it)?;
        let (it3, bv) = self.1.parse(&it2)?;
        let (it4, cv) = self.2.parse(&it3)?;
        let (it5, dv) = self.3.parse(&it4)?;
        Ok((it5, (av, bv, cv, dv)))
    }
}
