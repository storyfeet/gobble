pub mod basic;
pub mod combi;
pub mod err;
pub mod reader;
use basic::{ws, ParseRes, Parser};
use err::ParseError;
use reader::*;

#[derive(Debug)]
pub enum Expr {
    Val(i32),
    Add(Box<Expr>, Box<Expr>),
}

fn parse_expr<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Expr> {
    if let Ok((ir, (v1, v2))) = parse_expr.then_ig(ws(1)).then(parse_expr).parse(i) {
        return Ok((ir, Expr::Add(Box::new(v1), Box::new(v2))));
    }
    if let Ok((ir, v)) = read_f::<_, _, String>(is_num, 1).parse(i) {
        return Ok((ir, Expr::Val(v.parse().unwrap())));
    }
    Err(ParseError::new("Could not form expr", 0))
}

fn main() -> Result<(), std::io::Error> {
    let stdin = std::io::stdin();
    loop {
        let mut s = String::new();
        stdin.read_line(&mut s)?;
        let e = parse_expr(&s.chars());
        println!("{:?}", e)
    }
}

#[cfg(test)]
pub mod test {
    use crate::basic::*;
    use crate::combi::*;
    use crate::reader::*;
    #[test]
    fn test_can_build_ws_with_other() {
        let parser = wrap::<_, _, std::str::Chars, _>(ws(2), take(|c| c == 'p', 4));
        //let parser = take(|c| c == ' ' || c == 'p', 4);
        let (mut r, _) = parser.parse(&"  pppp  ,".chars()).unwrap();
        assert_eq!(r.next(), Some(','));
    }

    #[test]
    fn test_can_read_str() {
        let parser = read_f::<_, _, String>(|c: &char| *c == 'a' || *c == 'b', 2);
        let (mut r, v) = parser.parse(&"ababc".chars()).unwrap();
        assert_eq!(v, "abab");
        assert_eq!(r.next(), Some('c'));
    }
}
