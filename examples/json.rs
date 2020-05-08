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
    skip_while(
        |c| match c {
            ' ' | '\t' | '\n' | '\r' => true,
            _ => false,
        },
        0,
    )
}

pub fn is_hex_digit(c: char) -> bool {
    match c {
        v if is_num(v) => true,
        v if v >= 'A' && v <= 'F' => true,
        v if v >= 'a' && v <= 'f' => true,
        _ => false,
    }
}

pub fn js_char() -> impl Parser<char> {
    or3(
        "\\u".ig_then(take_n(4)).try_map(|v| {
            let n: u32 = u32::from_str_radix(&v, 16)
                .map_err(|_| ECode::SMess("could not get char from unicode").brk())?;
            std::char::from_u32(n).ok_or(ECode::SMess("Could not get char from u32").brk())
        }),
        "\\".ig_then(one_char("\"/\\bfrnt")).map(|c| match c {
            'b' => '\u{08}',
            'f' => '\u{0C}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            v => v,
        }),
        take_char,
    )
}

pub fn json_string() -> impl Parser<String> {
    "\"".ig_then(repeat_until_ig(js_char(), "\""))
        .map(|a| a.into_iter().collect())
}

///whitespace_newline wrapper
pub fn wsn_<P: Parser<V>, V>(p: P) -> impl Parser<V> {
    (wsn(), p, wsn()).map(|(_, b, _)| b)
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
