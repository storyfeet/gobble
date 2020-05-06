//! Gobble is a simple parser combinator system for parsing strings.
//!
//! For example parsing a function call
//!
//! ```rust
//! use gobble::*;
//! let ident = || {
//!     read_fs(is_alpha, 1)
//!         .then(read_fs(is_alpha_num, 0))
//!         .map(|(mut a, b)| {
//!             a.push_str(&b);
//!             a
//!     })
//! };
//!
//! let fsig = ident()
//!     .then_ig(tag("("))
//!     .then(sep(ident(), tag(","), true))
//!     .then_ig(tag(")"));
//!  
//!  let (nm, args) = fsig.parse_s("loadFile1(fname,ref)").unwrap();
//!  assert_eq!(nm, "loadFile1");
//!  assert_eq!(args, vec!["fname", "ref"]);
//!
//!  assert!(fsig.parse_s("23file(fname,ref)").is_err());
//!  
//!  ```
//!
//!  While from the example it is not so clear. A parser is a trait.
//!   
//!  ```ignore
//!
//!  pub type ParseRes<'a, V> = Result<(LCChars<'a>, V), ParseError>;
//!
//!  pub trait Parser<V> {
//!     fn parse<'a>(&self, LCChars<'a>)->ParseRes<'a,V>;
//!
//!     fn parse(&self,&str)->Result<V,ParseError>{
//!         // a helper method, parse_s which parses strings, by
//!         // converting them to a LLCChars, which is just a wrapper around
//!         // the chars iterator to keep track of line and column number
//!     }
//!
//!     //There are combinator methods and a helper methods that produce
//!     // other parsers wrapped around this one.  "or","map","then","then_ig","ig_then"
//!  }
//!  ```
//!
//!  Parser is also automatically implemented for any function that follows
//!  the same signature (without the self)
//!  ```rust
//!  use gobble::*;
//!  fn my_parser<'a>(i:&LCChars<'a>)->ParseRes<'a,String>{
//!     // i is an non mut pointer, so you have to clone it to mutate it,
//!     // this means parent iterators can try something else if this one fails
//!     // The combinators will do that for you
//!     let (it_clone,val) = tag("(").or(tag("[")).ig_then(read_fs(is_num,1)).parse(i)?;
//!     Ok((it_clone,val))
//!     //or you can just loop on the iterator clone yourself
//!  }
//!  //my_parser now can be combined with other parsers
//!  let (n,s) = my_parser.then(read_fs(is_alpha,1)).parse_s("(45red").unwrap();
//!  assert_eq!(n,"45");
//!  assert_eq!(s,"red");
//!  ```

pub mod combi;
pub mod common;
pub mod err;
pub mod iter;
pub mod ptrait;
pub mod reader;
pub mod repeater;
pub mod skip;
pub mod tuple;

pub use combi::*;
pub use common::*;
pub use err::*;
pub use iter::*;
pub use ptrait::*;
pub use reader::*;
pub use repeater::*;
pub use skip::*;
pub use tuple::*;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn demo_test() {
        let ident = || {
            read_fs(is_alpha, 1)
                .then(read_fs(is_alpha_num, 0))
                .map(|(mut a, b)| {
                    a.push_str(&b);
                    a
                })
        };
        let fsig = ident()
            .then_ig(tag("("))
            .then(sep(ident(), tag(","), true))
            .then_ig(tag(")"));

        let (nm, args) = fsig.parse_s("loadFile1(fname,ref)").unwrap();
        assert_eq!(nm, "loadFile1");
        assert_eq!(args, vec!["fname", "ref"]);
    }
}
