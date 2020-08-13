extern crate gobble;
use gobble::*;

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Div,
    Mul,
}

enum_parser! { (OP,op,Op) =>
    ((ADD->Op::Add) '+'),
    ((SUB->Op::Sub) '-'),
    ((DIV->Op::Div) '/'),
    ((MUL->Op::Mul) '*'),
}

#[derive(Debug)]
pub enum Expr {
    Val(isize),
    Parenth(Box<Expr>),
    Oper(Op, Box<Expr>, Box<Expr>),
    //Bracket(Box<Expr>),
}

//parser! {
//   (OP ->Op)
//   s_("+-*/".one()).try_map(|o| match o {
//       '+' => Ok(Op::Add),
//       '-' => Ok(Op::Sub),
//       '*' => Ok(Op::Mul),
//       '/' => Ok(Op::Div),
//       _ => Err(Expected::Str("OP")),
//   })
// }

parser! {
    (LtExpr->Expr)
    or(
        common::Int.map(|i| Expr::Val(i)),
        middle("(", RtExpr, ")").map(|e| Expr::Parenth(Box::new(e)))
    )
}

parser! {
    (RtExpr->Expr)
    (LtExpr, maybe((OP, RtExpr))).map(|(l, opt)| match opt {
        //Note this cares nothing for operation priority except for brackets
        Some((oper, r)) => Expr::Oper(oper, Box::new(l), Box::new(r)),
        None => l,
    })
}

/// This loops asking the user for input of simple sum type "4 + 6 * (3-1)"
/// and will simply output the parsed expression
fn main() -> Result<(), std::io::Error> {
    let stdin = std::io::stdin();
    loop {
        let mut s = String::new();
        match stdin.read_line(&mut s) {
            Ok(0) => return Ok(()),
            Err(e) => return Err(e),
            _ => {}
        }
        match first(RtExpr, ("\n", eoi)).parse_s(&s) {
            Ok(v) => println!("{:?}", v),
            Err(e) => println!("{}", e),
        }
    }
    //Ok(())
}
