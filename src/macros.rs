#[macro_export]
macro_rules! token {
    ($id:ident,$x:expr) => {
        token!($id, $x, &'static str);
    };
    ($id:ident,$x:expr,$ot:ty) => {
        pub struct $id;
        impl Parser for $id {
            type Out = $ot;
            fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
                (&$x).parse(it)
            }
        }
    };
}

#[macro_export]
macro_rules! tokens {
    { $( ($id:ident,$x:expr) ),*} => {$(token!($id,$x);)*};
}

#[macro_export]
macro_rules! keyword {
    ($id:ident,$x:expr) => {
        keyword!($id, $x, &'static str);
    };
    ($id:ident,$x:expr,$ot:ty) => {
        pub struct $id;
        impl Parser for $id {
            type Out = $ot;
            fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
                do_keyword(it, &$x)
            }
        }
    };
}

#[macro_export]
macro_rules! keywords {
    { $( ($id:ident,$x:expr) ),*} => {$(token!($id,$x);)*};
}

#[cfg(test)]
mod test {
    use crate::*;
    token!(DOG, "dog");
    tokens!((CAR, "car"), (CAT, "cat"));

    #[test]
    pub fn token_makes_parser() {
        assert_eq!(DOG.parse_s("dog   "), Ok("dog"));
        assert_eq!(CAT.parse_s("cat    "), Ok("cat"));
    }

    keyword!(LET, "let");

    #[test]
    pub fn keyword_makes_parser() {
        assert_eq!(LET.parse_s("let   "), Ok("let"));
        assert!(LET.parse_s("letr   ").is_err());
    }
}
