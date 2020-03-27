pub mod basic;
pub mod combi;
pub mod err;
pub mod reader;

pub enum Expr {
    Val(i32),
    Add(Box<Expr>, Box<Expr>),
}
fn main() {}

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
