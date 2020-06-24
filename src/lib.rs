//! Gobble is a simple parser combinator system for parsing strings.
//!
//! For example parsing a function call
//!
//! ```rust
//! use gobble::*;
//! parser!{
//!     (Ident->String)
//!     string((Alpha.one(),(Alpha,NumDigit,'_').star()))
//! }
//!
//! parser!{
//!     (FSig->(String,Vec<String>))
//!     (first(Ident,"("),sep_until(Ident,",",")"))
//! }
//! let (nm, args) = FSig.parse_s("loadFile1(fname,ref)").unwrap();
//! assert_eq!(nm, "loadFile1");
//! assert_eq!(args, vec!["fname", "ref"]);
//! //Idents can't begin with numbers
//! assert!(FSig.parse_s("23file(fname,ref)").is_err());
//! ```
//!
//! If you'd prefer not to use macros, you don't have to:
//!
//! ```rust
//! use gobble::*;
//! let ident = || string_2_parts(Alpha.min_n(1),(Alpha,NumDigit,'_').any());
//!
//! let fsig = (ident().then_ig("("),sep(ident(),",").then_ig(")"));
//!  
//!  let (nm, args) = fsig.parse_s("loadFile1(fname,ref)").unwrap();
//!  assert_eq!(nm, "loadFile1");
//!  assert_eq!(args, vec!["fname", "ref"]);
//!  //identifiers cant start with numbers,
//!  assert!(fsig.parse_s("23file(fname,ref)").is_err());
//!  
//!  ```
//!
//!  But the macros guarantee Zero-Sized types which is nice when combining them
//!
//!
//!  To work this library depends the following:
//!   
//!  ```rust
//!  pub enum ParseError {
//!     //...
//!  }
//!
//!  // In the OK Case the value mean
//!  //   LCChars = copy of original, but moved forward,
//!  //   V = The resulting type
//!  //   Option<ParserError> Only "Some" if the parser could have contined with more data
//!  //   --This is useful for tracking what values would have been expected at a certain point
//!  //
//!  pub type ParseRes<'a, V> = Result<(LCChars<'a>, V,Option<ParseError>), ParseError>;
//!
//!  //implements Iterator and can be cloned relatively cheaply
//!  pub struct LCChars<'a>{
//!     it:std::str::Chars<'a>,
//!     line:usize,
//!     col:usize,
//!  }
//!
//!  pub trait Parser<V> {
//!     // Takes a non-mut pointer to the iterator, so that the caller
//!     // may try something else if this doesn't work
//!     // clone it before reading next
//!     fn parse<'a>(&self,it:&LCChars<'a>)->ParseRes<'a,V>;
//!     
//!     //...helper methods
//!  }
//!  pub trait CharBool {
//!     fn char_bool(&self,c:char)->bool;
//!     //....helper methods
//!     //
//!  }
//!  ```
//!
//!  Parser is automatically implemented for:
//!  * ```Fn<'a>(&LCChars<'a>)->ParseRes<'a,String>```
//!  * ```&'static str``` which will return itself if it matches
//!  * ```char``` which will return itself if it matched the next char
//!  * Tuples of up to 6 parsers. Returning a tuple of all the
//!     parsers matched one after the
//!  other.
//!
//!  Most of the time a parser can be built simply by combining other parsers
//!  ```rust
//!  use gobble::*;
//!
//!  // map can be used to convert one result to another
//!  // keyval is now a function that returns a parser
//!  let keyval = || (common::Ident,":",common::Quoted).map(|(a,_,c)|(a,c));
//!
//!  //this can also be written as below for better type safety
//!  fn keyval2()->impl Parser<Out=(String,String)>{
//!     (common::Ident,":",common::Quoted).map(|(a,_,c)|(a,c))
//!  }
//!
//!  // or as a macro KeyVal is now a struct like:
//!  // pub struct KeyVal;
//!  parser!{
//!     (KeyVal->(String,String))
//!     (common::Ident,":",common::Quoted).map(|(a,_,c)|(a,c))
//!  }
//!  
//!  //parse_s is a helper on Parsers
//!  let (k,v) = keyval().parse_s(r#"car:"mini""#).unwrap();
//!  assert_eq!(k,"car");
//!  assert_eq!(v,"mini");
//!
//!  //this can now be combined with other parsers.
//!  // 'ig_then' combines 2 parsers and drops the result of the first
//!  // 'then_ig' drops the result of the second
//!  // 'sep_until will repeat the first term into a Vec, separated by the second
//!  //    until the final term.
//!  let obj = || "{".ig_then(sep_until(keyval(),",","}"));
//!
//!  let obs = obj().parse_s(r#"{cat:"Tiddles",dog:"Spot"}"#).unwrap();
//!  assert_eq!(obs[0],("cat".to_string(),"Tiddles".to_string()));
//!
//!  ```
//!  ## CharBool
//!
//!  CharBool is the trait for boolean char checks. It is auto implemented for:
//!  * Fn(char)->bool
//!  * char -- Returns true if the input matches the char
//!  * &'static str -- returns true if the str contains the input
//!  * several zero size types - Alpha,NumDigit,HexDigit,WS,WSL,Any
//!  * Tuples of up to 6 CharBools -- returning true if any of the members succeed
//!
//!  This means you can combine them in tuples ```(Alpha,NumDigit,"_").char_bool(c)```
//!  will be true if any of them match
//!
//!  
//!
//!  CharBool also provides several helper methods which each return a parser
//!  * ```one(self)``` matches and returns exactly 1 character
//!  * ```plus(self)``` '+' requires at least 1 matches and ruturns a string
//!  * ```min_n(self,n:usize)```  requires at least n matches and ruturns a string
//!  * ```star(self)``` '*' matches any number of chars returning a string
//!  * ```exact(self,n:usize)``` '*' matches exactly n chars returning a string
//!  * ```skip_plus(self)``` '+' requires at least 1 matches and ruturns a ()
//!  * ```skip_star(self)``` '*' matches any number of chars returning a ()
//!  * ```skip_exact(self,n:usize)``` matches exactly n chars returning a ()
//!  
//!  And a helper that returns a CharBool
//!  * ```except(self,cb:CharBool)``` Passes if self does, and cb doesnt
//! ```rust
//! use gobble::*;
//! let s = |c| c > 'w' || c == 'z';
//! let xv = s.one().parse_s("xhello").unwrap();
//! assert_eq!(xv,'x');
//!
//! let id = (Alpha,"_*").min_n(4).parse_s("sm*shing_game+you").unwrap();
//! assert_eq!(id,"sm*shing_game");
//!
//! // not enough matches
//! assert!((NumDigit,"abc").min_n(4).parse_s("23fflr").is_err());
//!
//! // any succeeds even with no matches equivilent to min_n(0) but "Zero Size"
//! assert_eq!((NumDigit,"abc").star().parse_s("23fflr"),Ok("23".to_string()));
//! assert_eq!((NumDigit,"abc").star().parse_s("fflr"),Ok("".to_string()));
//!
//! ```
//!
//! ## White Space
//!
//! White space is pretty straight forward to handle
//!
//! ```rust
//! use gobble::*;
//! let my_ws = || " \t".star();
//! // middle takes three parsers and returns the result of the middle
//! // this could also be done easily with 'map' or 'then_ig'
//! let my_s = |p| middle(my_ws(),p,my_ws());
//!
//! let sp_id = my_s(common::Ident);
//! let v = sp_id.parse_s("   \t  doggo  ").unwrap();
//! assert_eq!(v,"doggo");
//! ```
//! That said gobble already provides ```ws()``` and ```s_(p)```
//!
//! ```rust
//! use gobble::*;
//! //eoi = end of input
//! let p = repeat_until_ig(s_("abc".min_n(1)),eoi);
//! let r = p.parse_s("aaa \tbbb bab").unwrap();
//! assert_eq!(r,vec!["aaa","bbb","bab"]);
//! ```
//!
//! ## Recursive Structures
//!
//! Some structures like Json, or programming languages need to be able to
//! handle recursion. However with the techniques we have used so far
//! this would lead to infinitely sized structures.
//!
//! The way to handle this is to make sure one member of the loop is not  
//! build into the structure. Instead to create it using the 'Fn'
//!
//! ```rust
//! use gobble::*;
//! #[derive(Debug,PartialEq)]
//! enum Expr {
//!     Val(isize),
//!     Add(Box<Expr>,Box<Expr>),
//!     Paren(Box<Expr>),
//! }
//!
//! fn expr_l()->impl Parser<Out=Expr>{
//!     or(
//!         middle("(",s_(expr),")").map(|e|Expr::Paren(Box::new(e))),
//!         common::Int.map(|v|Expr::Val(v))
//!     )
//! }
//!
//! // using the full fn def we avoid the recursive structure
//! fn expr<'a>(it:&LCChars<'a>)->ParseRes<'a,Expr> {
//!     //note that expr_l has brackets but expr doesnt.
//!     //expr is a reference to a static function
//!     let p = (expr_l(),maybe(s_("+").ig_then(expr)))
//!         .map(|(l,opr)|match opr{
//!             Some(r)=>Expr::Add(Box::new(l),Box::new(r)),
//!             None=>l,
//!         });
//!     
//!
//!     p.parse(it)
//! }
//!
//! let r = expr.parse_s("45 + (34+3 )").unwrap();
//!
//! //recursive structures are never fun to write manually
//! assert_eq!(r,Expr::Add(
//!                 Box::new(Expr::Val(45)),
//!                 Box::new(Expr::Paren(Box::new(Expr::Add(
//!                     Box::new(Expr::Val(34)),
//!                     Box::new(Expr::Val(3))
//!                 ))))
//!             ));
//!
//! ```

#[macro_use]
pub mod macros;

pub mod chars;
pub mod combi;
pub mod common;
pub mod err;
pub mod iter;
pub mod ptrait;
pub mod reader;
pub mod repeater;
pub mod skip;
pub mod strings;
pub mod tuple;

pub use chars::*;
pub use combi::*;
//pub use common::*;
pub use err::*;
pub use iter::*;
pub use macros::*;
pub use ptrait::*;
pub use reader::*;
pub use repeater::*;
pub use skip::*;
pub use strings::*;
pub use tuple::*;
