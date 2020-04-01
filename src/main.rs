use combi::maybe;
use err::ParseError;
use ptrait::{ws, ParseRes, Parser};
use reader::*;

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Div,
    Mul,
}

#[derive(Debug)]
pub enum Expr {
    Val(i32),
    Parenth(Box<Expr>),
    Oper(Op, Box<Expr>, Box<Expr>),
    //Bracket(Box<Expr>),
}

fn parse_op<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Op> {
    let parser = ws(0)
        .ig_then(tag("+").or(tag("-")).or(tag("*")).or(tag("/")))
        .then_ig(ws(0));
    let (ri, c) = parser.parse(i)?;
    let rop = match c {
        "+" => Op::Add,
        "-" => Op::Sub,
        "*" => Op::Mul,
        "/" => Op::Div,
        _ => return Err(ParseError::new("Not sure how this happend, parse_op", 0)),
    };
    Ok((ri, rop))
}

fn parse_expr_l<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Expr> {
    if let Ok((ir, e)) = tag("(").ig_then(parse_expr).then_ig(tag(")")).parse(i) {
        return Ok((ir, Expr::Parenth(Box::new(e))));
    }
    if let Ok((ir, (neg, v))) = ws(0)
        .ig_then(maybe(tag("-")))
        .then(read_f::<_, _, String>(is_num, 1))
        .parse(i)
    {
        let mut n: i32 = v.parse().unwrap();
        if neg.is_some() {
            n = -n;
        }
        return Ok((ir, Expr::Val(n)));
    }
    Err(ParseError::new("Expr Left fail", 0))
}

pub fn parse_expr<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Expr> {
    let (ir, l) = parse_expr_l.parse(i)?;
    if let Ok((ir, (o, v2))) = parse_op.then(parse_expr).parse(&ir) {
        return Ok((ir, Expr::Oper(o, Box::new(l), Box::new(v2))));
    }
    Ok((ir, l))
}

fn main() -> Result<(), std::io::Error> {
    let stdin = std::io::stdin();
    loop {
        let mut s = String::new();
        match stdin.read_line(&mut s) {
            Ok(0) => return Ok(()),
            Err(e) => return Err(e),
            _ => {}
        }
        let e = parse_expr(&s.chars());
        println!("{:?}", e);
    }
    //Ok(())
}

#[cfg(test)]
pub mod test {
    use crate::combi::*;
    use crate::ptrait::*;
    use crate::reader::*;
    #[test]
    fn test_can_build_ws_with_other() {
        let parser = wrap::<_, _, std::str::Chars, _, _>(ws(2), take(|c| c == 'p', 4));
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
