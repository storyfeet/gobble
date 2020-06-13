//! The macros primarily exist to make creating zero size parsers easier.
//! Without putting them in macros "&'static str" and "chars" can act as parsers,
//! but they have a size, and when combined they can become bigger.
//! If however all the parsers you combine have zero size, then the final resulting parser
//! will also be zero size and therefor much easier to construct

#[macro_export]
macro_rules! parser {
    ($id:ident,$x:expr) => {
        parser!($id, &'static str, $x, Expected::Str(stringify!($id)));
    };
    ($id:ident,$ot:ty,$x:expr) => {
        parser!($id, $ot, $x, Expected::Str(stringify!($id)));
    };
    ($id:ident,$ot:ty,$x:expr,$exp:expr) => {
        #[derive(Copy, Clone)]
        pub struct $id;
        impl Parser for $id {
            type Out = $ot;
            fn parse<'a>(&self, it: &LCChars<'a>) -> ParseRes<'a, Self::Out> {
                (&$x).parse(it)
            }
            fn expected(&self) -> Expected {
                $exp
            }
        }
    };
}

#[macro_export]
macro_rules! parsers {
    { $(  $x:tt ),*} => {$(parser!($x);)*};
}

#[macro_export]
macro_rules! keyword {
    ($id:ident,$x:expr) => {
        keyword!($id, $x, &'static str);
    };
    ($id:ident,$x:expr,$ot:ty) => {
        #[derive(Copy, Clone)]
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
    { $( ($id:ident,$x:expr) ),*} => {$(keyword!($id,$x);)*};
}

#[macro_export]
macro_rules! char_bool {
    ($id:ident,$x:expr) => {
        char_bool!($id, $x, Expected::CharIn(stringify!($id)));
    };
    ($id:ident,$x:expr,$exp:expr) => {
        #[derive(Copy, Clone)]
        pub struct $id;
        impl CharBool for $id {
            fn char_bool(&self, c: char) -> bool {
                (&$x).char_bool(c)
            }
            fn expected(&self) -> Expected {
                $exp
            }
        }
    };
}

#[macro_export]
macro_rules! char_bools {
    { $( ($id:ident,$x:expr) ),*} => {$(char_bool!($id,$x);)*};
}

#[cfg(test)]
mod test {
    use crate::*;
    parser!(DOG, "dog");
    parsers!((CAR, "car"), (CAT, "cat"));
    parser!(GROW, Vec<String>, rep(or(CAT, DOG)));

    #[test]
    pub fn parser_makes_parser() {
        assert_eq!(DOG.parse_s("dog   "), Ok("dog"));
        assert_eq!(CAT.parse_s("cat    "), Ok("cat"));
        assert_eq!(
            GROW.parse_s("catdogcatcatno"),
            Ok(vec!["cat", "dog", "cat", "cat"])
        );
    }

    keyword!(LET, "let");

    #[test]
    pub fn keyword_makes_parser() {
        assert_eq!(LET.parse_s("let   "), Ok("let"));
        assert!(LET.parse_s("letr   ").is_err());
    }

    char_bool!(HOT, "hot");
    char_bool!(MNUM, |c| c >= '0' && c <= '9');

    #[test]
    pub fn charbool_macro_makes_parser() {
        use Expected::*;
        let p = (HOT, MNUM);
        assert_eq!(std::mem::size_of::<(HOT, MNUM)>(), 0);
        assert_eq!(p.plus().parse_s("09h3f"), Ok("09h3".to_string()));
        assert_eq!(p.expected(), OneOf(vec![CharIn("HOT"), CharIn("MNUM")]));
    }
}
