use gobble::*;
use std::collections::HashMap;

fn main() {
    let s = r#"{
        "name":"sam\t\u0048",
        "age"    :5,
        "n":null
    }"#;
    let v = p_value.parse_s(s);
    println!("{:?}", v);
}

#[derive(Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

fn wsn() -> impl Parser<()> {
    skip_while(" \t\n\r", 0)
}

pub fn is_hex_digit(c: char) -> bool {
    (NumDigit, "abcdefABCDEF").char_bool(c)
}

pub fn js_char() -> impl Parser<char> {
    or3(
        "\\u".ig_then(take_n(4)).try_map(|v| {
            let n: u32 = u32::from_str_radix(&v, 16)
                .map_err(|_| ECode::SMess("could not get char from unicode").brk())?;
            std::char::from_u32(n).ok_or(ECode::SMess("Could not get char from u32").brk())
        }),
        '\\'.ig_then(or6(
            'b'.asv('\u{08}'),
            'f'.asv('\u{0C}'),
            'n'.asv('\n'),
            'r'.asv('\r'),
            't'.asv('\t'),
            "\"\\".one(),
        )),
        take_char,
    )
}

pub fn json_string() -> impl Parser<String> {
    "\"".ig_then(chars_until(js_char(), '"'))
}

///whitespace_newline wrapper
pub fn wsn_<P: Parser<V>, V>(p: P) -> impl Parser<V> {
    middle(wsn(), p, wsn())
}

pub fn map_item() -> impl Parser<(String, Value)> {
    (json_string(), wsn_(":"), p_value).map(|(a, _, b)| (a, b))
}

pub fn p_value<'a>(it: &LCChars<'a>) -> ParseRes<'a, Value> {
    let p = or6(
        "null".map(|_| Value::Null),
        common_bool.map(|b| Value::Bool(b)),
        or(
            common_float.map(|f| Value::Num(f)),
            common_int.map(|i| Value::Num(i as f64)),
        ),
        json_string().map(|s| Value::Str(s)),
        "[".ig_then(sep_until(wsn_(p_value), ",", "]"))
            .map(|a| Value::Array(a)),
        "{".ig_then(sep_until(wsn_(map_item()), ",", "}")).map(|a| {
            let mut m = HashMap::new();
            for (k, v) in a {
                m.insert(k, v);
            }
            Value::Object(m)
        }),
    );
    p.parse(it)
}
