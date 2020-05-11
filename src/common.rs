use crate::chars::*;
use crate::err::*;
use crate::iter::*;
use crate::ptrait::*;
use crate::reader::*;
use crate::strings::*;
use crate::tuple::*;
use std::convert::TryFrom;

pub fn common_esc<'a>(it: &LCChars<'a>) -> ParseRes<'a, char> {
    '\\'.ig_then(or4('t'.asv('\t'), 'r'.asv('\r'), 'n'.asv('\n'), take_char))
        .parse(it)
}

/// ```rust
/// use gobble::*;
/// assert_eq!(common_str.parse_s(r#""hello\t\"world\"""#),Ok("hello\t\"world\"".to_string()));
/// ```
pub fn common_str<'a>(it: &LCChars<'a>) -> ParseRes<'a, String> {
    '"'.ig_then(chars_until(or(common_esc, take_char), '"'))
        .parse(it)
}

pub fn common_ident<'a>(it: &LCChars<'a>) -> ParseRes<'a, String> {
    string_2_parts(Alpha.min(1), (Alpha, NumDigit, '_').any()).parse(it)
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
                    return Ok((it2, res));
                }
                return it.err_r("No numerical digits");
            }
        }
    }
}

/// A function for parsing integers
/// ```
/// use gobble::*;
/// let r = common_int.parse_s("32").unwrap();
/// assert_eq!(r,32);
/// ```
///
pub fn common_int<'a>(it: &LCChars<'a>) -> ParseRes<'a, isize> {
    //TODO add and mul without panic
    let mut it2 = it.clone();
    let (minus, it2) = match it2.next() {
        Some('-') => (-1, it2),
        Some(v) if is_num(v) => (1, it.clone()),
        _ => return it.err_cr(ECode::SMess("Not an int")),
    };

    let (it3, n) = common_uint(&it2)?;
    let n: isize = isize::try_from(n).map_err(|_| it3.err("Int Too Big"))?;
    Ok((it3, n * minus))
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

pub fn dot_part<'a>(i: &LCChars<'a>) -> ParseRes<'a, f64> {
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
            _ => return Ok((it2, res)),
        }
    }
}

fn do_exponent<'a>(mut n: f64, i: &LCChars<'a>) -> ParseRes<'a, f64> {
    let mut it = i.clone();
    if it.next() != Some('e') {
        return Ok((i.clone(), n));
    }
    let (it2, exp) = common_int(&it)?;
    if exp > 0 {
        for _ in 0..exp {
            n *= 10.;
        }
    }
    if exp < 0 {
        for _ in 0..-exp {
            n /= 10.;
        }
    }
    Ok((it2, n))
}

/// ```rust
/// use gobble::*;
/// let r = common_float.parse_s("32.").unwrap();
/// assert_eq!(r, 32.);
/// let r = common_float.parse_s("-23.4").unwrap();
/// assert_eq!(r, -23.4);
/// let r = common_float.parse_s("-23.4e2").unwrap();
/// assert_eq!(r, -2340.);
/// let r = common_float.parse_s("123.4e-2").unwrap();
/// assert_eq!(r, 1.234);
/// ```
pub fn common_float<'a>(it: &LCChars<'a>) -> ParseRes<'a, f64> {
    let (it2, n) = common_int(it)?;
    let minus = if n >= 0 { 1. } else { -1. };
    let it3 = it2.clone();
    let mut nf = n as f64;

    let (it3, dp) = dot_part(&it3)?;
    nf += dp * minus;
    do_exponent(nf, &it3)
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
