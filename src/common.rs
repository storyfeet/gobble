use crate::err::*;
use crate::iter::*;
use crate::ptrait::*;
use crate::reader::*;

/// A function for parsing integers
/// ```ignore
/// use gobble::*;
/// let r = common_int.parse_s("32").unwrap();
/// assert_eq!(r,32);
/// ```
///
pub fn common_int<'a>(it: &LCChars<'a>) -> ParseRes<'a, isize> {
    //TODO add and mul without panic
    let mut added = false;
    let mut it = it.clone();
    let (minus, mut res) = match it.next() {
        Some('-') => (-1, 0),
        Some(v) if v >= '0' && v <= '9' => {
            added = true;
            (1, (v as isize - '0' as isize))
        }
        _ => return it.err_cr(ECode::SMess("Not an int")),
    };

    let mut it2 = it.clone();
    loop {
        match it.next() {
            Some(v) if v >= '0' && v <= '9' => {
                added = true;
                res = res
                    .checked_mul(10)
                    .ok_or(it.err("Num too big"))?
                    .checked_add(v as isize - '0' as isize)
                    .ok_or(it.err("Num too big"))?;
                it2 = it.clone();
            }
            Some('_') => {
                it2 = it.clone();
            }
            _ => {
                if added {
                    return Ok((it2, minus * res));
                } else {
                    return it.err_r("Expected int, got '-'");
                }
            }
        }
    }
}

/// ```ignore
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

/// ```rust ignore
/// use gobble::*;
/// let r = common_float.parse_s("32").unwrap();
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
            .is_err())
    }

    #[test]
    pub fn parse_floats() {
        let r = common_float.parse_s("32").unwrap();
        assert_eq!(r, 32.);
        let r = common_float.parse_s("-23.4").unwrap();
        assert_eq!(r, -23.4);
        let r = common_float.parse_s("-23.4e2").unwrap();
        assert_eq!(r, -2340.);
        let r = common_float.parse_s("123.4e-2").unwrap();
        assert_eq!(r, 1.234);
    }
}
