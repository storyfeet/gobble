#![allow(deprecated)]
//! Generally useful base parsers
//! Str,Int,Uint,Esc,Float
//!
//! ```rust
//! use gobble::*;
//! use common::*;
//!
//! assert_eq!(Bool.parse_s("true").unwrap(),true);
//! assert_eq!(Bool.parse_s("false").unwrap(),false);
//!
//! assert_eq!(Quoted.parse_s(r#""hello\t\"world\"""#),Ok("hello\t\"world\"".to_string()));
//!
//! assert_eq!(Ident.parse_s("me34A_ dothing").unwrap(),"me34A_");
//! assert_eq!(Int.parse_s("32").unwrap(),32);
//!
//! //floats
//! assert_eq!(Float.parse_s("32.").unwrap(),32.);
//! assert_eq!(Float.parse_s("-23.4").unwrap(),-23.4);
//! assert_eq!(Float.parse_s("-23.4e2").unwrap(),-2340.);
//! assert_eq!(Float.parse_s("123.4e-2").unwrap(),1.234);
//! ```
use crate::chars::*;
use crate::combi::*;
use crate::err::*;
use crate::iter::*;
use crate::ptrait::*;
use crate::reader::*;
use crate::strings::*;
use crate::tuple::*;
use std::convert::TryFrom;

parser! { "Escapes a '\' and converts '\\n' '\\t' '\\r'"
    (Esc->char)
    last('\\',or!('t'.asv('\t'), 'r'.asv('\r'), 'n'.asv('\n'), Any.one()))
}

#[deprecated(since = "0.5.1", note = "use Ident instead")]
pub fn common_esc<'a>(it: &LCChars<'a>) -> ParseRes<'a, char> {
    '\\'.ig_then(or4('t'.asv('\t'), 'r'.asv('\r'), 'n'.asv('\n'), Any.one()))
        .parse(it)
}

parser! { "A string surrounded in Quotes"
    (Quoted->String),
    '"'.ig_then(chars_until(or(Esc, Any.one()), '"').map(|(a, _)| a)),
}

parser! { "A letter followed by numbers and letters or '_'"
    (Ident->String)
    string((Alpha.iplus(), (Alpha, NumDigit, '_').istar()))
}

#[deprecated(since = "0.5.0", note = "use Ident instead")]
pub fn common_ident<'a>(it: &LCChars<'a>) -> ParseRes<'a, String> {
    Ident.parse(it)
}

parser! { "a usize"
    (UInt->usize)
    common_uint
}

#[deprecated(since = "0.5.1", note = " will go private as UInt does the job")]
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

parser! { "Returns a parsed isize"
    (Int->isize)
    (maybe('-'),UInt).try_map(|(m,n)|{
        let n = isize::try_from(n).map_err(|_|Expected::Str("Int too big"))?;
        match m {
            Some(_)=>Ok(-n),
            None=>Ok(n),
        }
    })
}

parser! { "the words 'true' or 'false'"
    (Bool->bool)
    or(keyword("true").map(|_|true),keyword("false").map(|_|false))
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

parser! {"'e' followed by a uint, allowed on floats"
    (Exponent->isize)
    last('e',Int)
}

parser! { "floating point numbers eg '134.4e6'"
    (Float->f64)
    (Int,dot_part,maybe(Exponent)).map(|(n,d,e)|{
        let mut res =n as f64;
        res += res.signum() * d;
        if let Some(exp) = e{
            match exp >= 0{
                true =>{
                    for _ in 0..exp {
                        res *= 10.
                    }
                }
                false =>{
                    for _ in 0..-exp {
                        res /= 10.
                    }
                }
            }
        };
        res
    })

}

#[cfg(test)]
pub mod test {
    use super::*;
    #[test]
    pub fn test_parse_numbers() {
        let r = Int.parse_s("32").unwrap();
        assert_eq!(r, 32);
        let r = Int.parse_s("-45023").unwrap();
        assert_eq!(r, -45023);
        let r = Int.parse_s("34_234").unwrap();
        assert_eq!(r, 34234);
        assert!(Int.parse_s("45654323456765432345676543212345654").is_err());
        assert!(Int.parse_s("   45").is_err());
    }

    #[test]
    pub fn parse_floats() {
        let r = Float.parse_s("32.").unwrap();
        assert_eq!(r, 32.);
        let r = Float.parse_s("-23.4").unwrap();
        assert_eq!(r, -23.4);
        let r = Float.parse_s("-23.4e2").unwrap();
        assert_eq!(r, -2340.);
        let r = Float.parse_s("123.4e-2").unwrap();
        assert_eq!(r, 1.234);
        assert!(Float.parse_s("123").is_err());
    }
}
