use gobble::*;
use std::collections::HashMap;

fn main() {
    let s = r#"{
        "name":"sam\t\u0048",
        "age"    :5,
        "n":null
    }"#;
    let v = JsonValue.parse_s(s);
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

pub fn wsn_<P: Parser>(p: P) -> impl Parser<Out = P::Out> {
    middle(WSL.skip_star(), p, WSL.skip_star())
}

parser!(
    (JsonChar -> char)
    or3(
        "\\u".ig_then(
            HexDigit
                .exact(4)
                .try_map(|v| {
                    let n: u32 =
                        u32::from_str_radix(&v, 16).map_err(|_| Expected::Str("4 hex digits"))?;
                    std::char::from_u32(n).ok_or(Expected::Str("4 Hex digits"))
                })
                .brk(),
        ),
        '\\'.ig_then(or6(
            'b'.asv('\u{08}'),
            'f'.asv('\u{0C}'),
            'n'.asv('\n'),
            'r'.asv('\r'),
            't'.asv('\t'),
            "\"\\".one(),
        )),
        Any.one()
    )
);

parser!(
    (JsonString->String)
    "\"".ig_then(chars_until(JsonChar, '"')).map(|(a, _)| a)
);

parser!(
    (MapItem->(String,Value))
    (JsonString, wsn_(":"), JsonValue).map(|(a, _, b)| (a, b))
);

parser!(
    (JsonValue->Value)
    or!(
        "null".map(|_| Value::Null),
        common::Bool.map(|b| Value::Bool(b)),
        or(
            common::Float.map(|f| Value::Num(f)),
            common::Int.map(|i| Value::Num(i as f64)),
        ),
        JsonString.map(|s| Value::Str(s)),
        "[".ig_then(sep_until(wsn_(JsonValue), ",", "]"))
            .map(|a| Value::Array(a)),
        "{".ig_then(sep_until(wsn_(MapItem), ",", "}")).map(|a| {
            let mut m = HashMap::new();
            for (k, v) in a {
                m.insert(k, v);
            }
            Value::Object(m)
        })
    )
);
