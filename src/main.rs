extern crate gobble;
use gobble::*;

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Div,
    Mul,
}

#[derive(Debug)]
pub enum Expr {
    Val(isize),
    Parenth(Box<Expr>),
    Oper(Op, Box<Expr>, Box<Expr>),
    //Bracket(Box<Expr>),
}

fn parse_op() -> impl Parser<Op> {
    s_(or4("+", "-", "*", "/")).try_map(|o| match o {
        "+" => Ok(Op::Add),
        "-" => Ok(Op::Sub),
        "*" => Ok(Op::Mul),
        "/" => Ok(Op::Div),
        _ => Err(ECode::Never("Op not in list")),
    })
}

fn parse_expr_l<'a>(i: &LCChars<'a>) -> ParseRes<'a, Expr> {
    let p = or(
        sel3_b("(", parse_expr, ")").map(|e| Expr::Parenth(Box::new(e))),
        common_int.map(|i| Expr::Val(i)),
    );
    p.parse(i)
}

pub fn parse_expr<'a>(i: &LCChars<'a>) -> ParseRes<'a, Expr> {
    let p = (parse_expr_l, maybe((parse_op(), parse_expr))).map(|(l, opt)| match opt {
        Some((oper, r)) => Expr::Oper(oper, Box::new(l), Box::new(r)),
        None => l,
    });
    p.parse(i)
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
        let e = parse_expr.parse_s(&s);
        println!("{:?}", e);
    }
    //Ok(())
}

#[cfg(test)]
pub mod test {
    use gobble::*;
    #[test]
    fn test_can_build_ws_with_other() {
        let parser = wrap(ws(2), take(|c| c == 'p', 4));
        let cc = LCChars::str("  pppp  ,");
        let (mut r, _) = parser.parse(&cc).unwrap();
        assert_eq!(r.next(), Some(','));
    }

    #[test]
    fn test_can_read_str() {
        let parser = read_fs(|c: char| c == 'a' || c == 'b', 2);
        let v = parser.parse_s("ababc").unwrap();
        assert_eq!(v, "abab");
    }
}
