//! The macros primarily exist to make creating zero size parsers easier.
//! Without putting them in macros "&'static str" and "chars" can act as parsers,
//! but they have a size, and when combined they can become bigger.
//! If however all the parsers you combine have zero size, then the final resulting parser
//! will also be zero size and therefor much easier to construct
//!

/// Makes zero sized parsers based on the expression given and potentially the return type given.

#[macro_export]
macro_rules! parser {
    ($id:ident,$x:expr) => {
        parser!(($id->&'static str) $x);
    };
    (($id:ident -> $ot:ty) $(,)? $x:expr $(,)?) => {
        parser!(($id->$ot) $x, Expected::Str(stringify!($id)));
    };
    ($id:ident,$x:expr,$exp:expr) => {
        parser!(($id->&'static str) $x, $exp);
    };
    (($id:ident -> $ot:ty) $(,)? $x:expr,$exp:expr $(,)?) => {
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
macro_rules! parser_as {
    (($ot:ty),(($id:ident->$res:expr) $(,)? $main:expr,$exp:expr $(,)?) ) => {
        parser! {($id->$ot) ,$main.map(|_|$res),$exp}
    };
    (($ot:ty),(($id:ident->$res:expr) $(,)? $main:expr $(,)?) ) => {
        parser! {($id->$ot) ,$main.map(|_|$res)}
    };
    (($ot:ty),($id:ident, $main:expr)) => {
        parser! { ($id->$ot) $main}
    };
}

#[macro_export]
macro_rules! as_id {
    ((($id:ident->$_x:expr) $($_t:tt)*) ) => {
        $id
    };
    (($id:ident $($_t:tt)*) ) => {
        $id
    };
}

/// ```rust
///
/// use gobble::*;
/// mod scoper{
///     // had to make a new scope for the doc test but it shouldn't be needed
///     // from outer crates
///     use gobble::*;
///     //declare the enum
///     #[derive(Clone, PartialEq, Debug)]
///     pub enum Oper {
///         Add,
///         Sub,
///         Div,
///         Mul,
///         Var(String),
///     }
///     
///     enum_parser! { (OPER,oper,Oper) =>
///         ((ADD->Oper::Add) '+'),
///         ((SUB->Oper::Sub) '-'),
///         ((DIV->Oper::Div) '/'),
///         ((MUL->Oper::Mul) '*'),
///         (VAR , Alpha.plus().map(|s|Oper::Var(s))),
///     }
/// }
/// use scoper::*;
///
/// let v = rep(scoper::OPER).parse_s("-cat").unwrap();
/// assert_eq!( v, vec![ Oper::Sub, Oper::Var("cat".to_string()) ]);
///
/// let v2 = rep(or!(oper::ADD, oper::SUB)).parse_s("-+-hello").unwrap();
/// assert_eq!(v2, vec![Oper::Sub, Oper::Add, Oper::Sub]);
///
///
/// ```
#[macro_export]
macro_rules! enum_parser{
    ( ($name:ident,$mod:ident,$ot:ty)=>$($mbit:tt),* $(,)?) =>{
        pub mod $mod{
            use $crate::*;
            use super::*;
            $( parser_as!{($ot),$mbit})*
            parser!{ ($name->$ot) ( or!{ $(as_id!{$mbit}),*} )}
        }
        pub use $mod::$name;
    }
}

#[macro_export]
macro_rules! char_bool {
    ($id:ident,$x:expr) => {
        char_bool!($id, $x, Expected::CharIn(stringify!($id)));
    };
    ($id:ident,$x:expr,$s:literal) => {
        char_bool!($id, $x, Expected::CharIn($s));
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
    ( $( ($id:ident,$x:expr) ),*) => {$(char_bool!($id,$x);)*};
}

/// a macro replacement for numbered or statements.
/// ```rust
/// use gobble::*;
/// assert_eq!(or!("cat","dog","car").parse_s("catdogman "),Ok("cat"));
/// ```
#[macro_export]
macro_rules! or{
    ($s:expr,$($x:expr),*) => { $s$(.or($x))*;};
}

#[cfg(test)]
mod test {

    fn size_of<T: Sized>(_t: &T) -> usize {
        std::mem::size_of::<T>()
    }

    use crate::*;
    parser!(DOG, "dog");
    parser!(CAR, "car");
    parser!(CAT, "cat");

    parser!((GROW->Vec<&'static str>) rep(or(CAT, DOG)));

    #[test]
    pub fn parser_makes_parser() {
        assert_eq!(DOG.parse_s("dog   "), Ok("dog"));
        assert_eq!(CAT.parse_s("cat    "), Ok("cat"));
        assert_eq!(
            GROW.parse_s("catdogcatcatno"),
            Ok(vec!["cat", "dog", "cat", "cat"])
        );
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
        assert_eq!(size_of(&p), 0);
    }
    #[derive(Clone, PartialEq, Debug)]
    pub enum Oper {
        Add,
        Sub,
        Div,
        Mul,
        Var(String),
    }

    enum_parser! { (OPER,oper,Oper) =>
        ((ADD->Oper::Add) '+'),
        ((SUB->Oper::Sub) '-'),
        ((DIV->Oper::Div) '/'),
        ((MUL->Oper::Mul) '*'),
        (VAR , Alpha.plus().map(|s|Oper::Var(s))),
    }

    #[test]
    fn test_enum_group_make_parser() {
        let v = rep(OPER).parse_s("-cat").unwrap();
        assert_eq!(v, vec![Oper::Sub, Oper::Var("cat".to_string())]);

        let v2 = rep(or!(oper::ADD, oper::SUB)).parse_s("-+-hello").unwrap();
        assert_eq!(v2, vec![Oper::Sub, Oper::Add, Oper::Sub]);
    }
}
