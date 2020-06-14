/// Generally useful base parsers
/// CommonStr,CommonInt,CommonUint,CommonEsc,CommonFloat
/// ```rust
/// use gobble::*;
/// assert_eq!(CommonStr.parse_s(r#""hello\t\"world\"""#),Ok("hello\t\"world\"".to_string()));
///
/// assert_eq!(CommonIdent.parse_s("me34A_ dothing").unwrap(),"me34A_");
/// let r = common_int.parse_s("32").unwrap();
/// assert_eq!(r,32);
///
/// //floats
/// let r = common_float.parse_s("32.").unwrap();
/// assert_eq!(r, 32.);
/// let r = common_float.parse_s("-23.4").unwrap();
/// assert_eq!(r, -23.4);
/// let r = common_float.parse_s("-23.4e2").unwrap();
/// assert_eq!(r, -2340.);
/// let r = common_float.parse_s("123.4e-2").unwrap();
/// assert_eq!(r, 1.234);
/// ```
/// ```
use crate::chars::*;
use crate::combi::*;
use crate::err::*;
use crate::iter::*;
use crate::ptrait::*;
use crate::reader::*;
use crate::strings::*;
use crate::tuple::*;
use std::convert::TryFrom;

parser!(
    (CommonEsc->char)
    '\\'.ig_then(or4('t'.asv('\t'), 'r'.asv('\r'), 'n'.asv('\n'), Any.one()))
);

pub fn common_esc<'a>(it: &LCChars<'a>) -> ParseRes<'a, char> {
    '\\'.ig_then(or4('t'.asv('\t'), 'r'.asv('\r'), 'n'.asv('\n'), Any.one()))
        .parse(it)
}

parser! {
    (CommonStr->String)
    '"'.ig_then(chars_until(or(CommonEsc, Any.one()), '"').map(|(a, _)| a))
}

#[deprecated(since = "0.5.0", note = "use CommonStr instead")]
pub fn common_str<'a>(it: &LCChars<'a>) -> ParseRes<'a, String> {
    CommonStr.parse(it)
}

parser! {
    (CommonIdent->String)
    string((Alpha.skip_plus(), (Alpha, NumDigit, '_').skip_star()))
}

#[deprecated(since = "0.5.0", note = "use CommonIdent instead")]
pub fn common_ident<'a>(it: &LCChars<'a>) -> ParseRes<'a, String> {
    CommonIdent.parse(it)
}

parser! {
    (CommonUInt->usize)
    common_uint
}

pub fn common_uint<'a>(it: &LCChars<'a>) -> ParseRes<'a, usize> {
    let mut added = false;
    let mut res: usize = 0;
    let mut it = it.clone();
    loop {
        let it2 = it.clone();
        match it.next() {
            Some(v) if is_num(v) => {
                added = true;
                res = res
                    .checked_mul(10)
                    .ok_or(it.err("Num too big"))?
                    .checked_add(v as usize - '0' as usize)
                    .ok_or(it.err("Num too big"))?;
            }
            Some('_') => {}
            _ => {
                if added {
                    return Ok((it2, res, None));
                }
                return it2.err_r("[0-9]*");
            }
        }
    }
}

parser! {
    (CommonInt->isize)
    (maybe('-'),CommonUInt).try_map(|(m,n)|{
        let n = isize::try_from(n).map_err(|_|Expected::Str("Int too big"))?;
        match m {
            Some(_)=>Ok(-n),
            None=>Ok(n),
        }
    })
}

#[deprecated(since = "0.5.0", note = "Use CommonInt instead")]
pub fn common_int<'a>(it: &LCChars<'a>) -> ParseRes<'a, isize> {
    CommonInt.parse(it)
}

/// ```
/// use gobble::*;
/// let v = common_bool.parse_s("true").unwrap();
/// assert!(v);
/// ```
pub fn common_bool<'a>(it: &LCChars<'a>) -> ParseRes<'a, bool> {
    keyword("true")
        .map(|_| true)
        .or(keyword("false").map(|_| false))
        .parse(it)
}

fn dot_part<'a>(i: &LCChars<'a>) -> ParseRes<'a, f64> {
    let mut res = 0.;
    let mut exp = 0.1;
    let mut it = i.clone();
    if it.next() != Some('.') {
        return i.err_r("no_dot_part");
    }
    loop {
        let it2 = it.clone();
        match it.next() {
            Some('_') => {}
            Some(v) if is_num(v) => {
                res += (((v as i64) as f64) - 48.0) * exp;
                exp *= 0.1;
            }
            _ => return Ok((it2, res, None)),
        }
    }
}

parser! {
    (CommonExponent->f64)
    ('e',CommonInt).map(|(_,ex)|{
        let (n,mul)= match ex >=0{
            true => (ex,10.),
            false => (-ex,0.1),
        };
        let mut res = 1.;
        for _ in 0..n{
            res *=mul;
        }
        res
    })
}

parser! {
    (CommonFloat->f64)
    (CommonInt,maybe(dot_part),maybe(CommonExponent)).map(|(n,d,e)|{
        let mut res =n as f64;
        if let Some(dp) =d  {
            res += res.signum() * dp;
        }
        if let Some(exp) = e{
            res *= exp;
        }
        res
    })
}

pub fn common_float<'a>(it: &LCChars<'a>) -> ParseRes<'a, f64> {
    CommonFloat.parse(it)
}

#[cfg(test)]
pub mod test {
    use super::*;
    #[test]
    pub fn test_parse_numbers() {
        let r = common_int.parse_s("32").unwrap();
        assert_eq!(r, 32);
        let r = common_int.parse_s("-45023").unwrap();
        assert_eq!(r, -45023);
        let r = common_int.parse_s("34_234").unwrap();
        assert_eq!(r, 34234);
        assert!(common_int
            .parse_s("45654323456765432345676543212345654")
            .is_err());
        assert!(common_int.parse_s("   45").is_err());
    }

    #[test]
    pub fn parse_floats() {
        let r = common_float.parse_s("32.").unwrap();
        assert_eq!(r, 32.);
        let r = common_float.parse_s("-23.4").unwrap();
        assert_eq!(r, -23.4);
        let r = common_float.parse_s("-23.4e2").unwrap();
        assert_eq!(r, -2340.);
        let r = common_float.parse_s("123.4e-2").unwrap();
        assert_eq!(r, 1.234);
        assert!(common_float.parse_s("123").is_err());
    }
}
