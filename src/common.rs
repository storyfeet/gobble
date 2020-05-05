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
    let mut it = it.clone();
    let (minus, mut res) = match it.next() {
        Some('-') => (-1, 0),
        Some(v) if v >= '0' && v <= '9' => (1, (v as isize - '0' as isize)),
        _ => return it.err_cr(ECode::SMess("Not an int")),
    };

    let mut it2 = it.clone();
    loop {
        match it.next() {
            Some(v) if v >= '0' && v <= '9' => {
                res = res
                    .checked_mul(10)
                    .ok_or(it.err_c(ECode::SMess("Num too big")))?
                    .checked_add(v as isize - '0' as isize)
                    .ok_or(it.err_c(ECode::SMess("Num too big")))?;
                it2 = it.clone();
            }
            _ => return Ok((it2, minus * res)),
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

#[cfg(test)]
pub mod test {
    use super::*;
    #[test]
    pub fn test_parse_numbers() {
        let r = common_int.parse_s("32").unwrap();
        assert_eq!(r, 32);
        let r = common_int.parse_s("-45023").unwrap();
        assert_eq!(r, -45023);
        assert!(common_int
            .parse_s("45654323456765432345676543212345654")
            .is_err())
    }
}
