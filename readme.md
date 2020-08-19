Gobble is a simple parser combinator system for parsing strings.  

*Note:It works well but it is currently still under heavy development, so the API may change significantly between versions. if the 'b' changes in "0.b.c" there will be breaking changes. Though I do believe right now I'm close to setting on the API

I'm very open to recieving feedback on github*

Creating Parsers in rust should be quite straight forward. For example parsing a function call

```rust
use gobble::*;
parser!{
    (Ident->String)
    string((Alpha.one(),(Alpha,NumDigit,'_').istar()))
}

parser!{
    (FSig->(String,Vec<String>))
    (first(Ident,"("),sep_until_ig(Ident,",",")"))
}
let (nm, args) = FSig.parse_s("loadFile1(fname,ref)").unwrap();
assert_eq!(nm, "loadFile1");
assert_eq!(args, vec!["fname", "ref"]);
//Idents can't begin with numbers
assert!(FSig.parse_s("23file(fname,ref)").is_err());
```

If you'd prefer not to use macros, you don't have to:

```rust
use gobble::*;
let ident = || string((Alpha.one(),(Alpha,NumDigit,'_').istar()));

let fsig = (first(ident(),"("),sep_until_ig(ident(),",",")"));
 
 let (nm, args) = fsig.parse_s("loadFile1(fname,ref)").unwrap();
 assert_eq!(nm, "loadFile1");
 assert_eq!(args, vec!["fname", "ref"]);
 //identifiers cant start with numbers,
 assert!(fsig.parse_s("23file(fname,ref)").is_err());
 
 ```

 But the macros guarantee of Zero-Sized types which is nice when combining them


 To work this library depends the following:
  
 ```rust
 pub enum ParseError {
    //...
 }

 // In the OK Case the value mean
 //   LCChars = copy of original, but moved forward,
 //   V = The resulting type
 //   Option<ParserError> Only "Some" if the parser could have contined with more data
 //   --This is useful for tracking what values would have been expected at a certain point
 //
 pub type ParseRes<'a, V> = Result<(LCChars<'a>, V,Option<ParseError>), ParseError>;

 //implements Iterator and can be cloned relatively cheaply
 pub struct LCChars<'a>{
    it:std::str::Chars<'a>,
    line:usize,
    col:usize,
 }

 pub trait Parser<V> {
    // Takes a non-mut pointer to the iterator, so that the caller
    // may try something else if this doesn't work
    // clone it before reading next
    fn parse<'a>(&self,it:&LCChars<'a>)->ParseRes<'a,V>;
    
    //...helper methods
 }
 pub trait CharBool {
    fn char_bool(&self,c:char)->bool;
    //....helper methods
    //
 }
 ```

 Parser is automatically implemented for:
 * ```Fn<'a>(&LCChars<'a>)->ParseRes<'a,String>```
 * ```&'static str``` which will return itself if it matches
 * ```char``` which will return itself if it matched the next char
 * Tuples of up to 6 parsers. Returning a tuple of all the
    parsers matched one after the
 other.

 Most of the time a parser can be built simply by combining other parsers
 ```rust
 use gobble::*;

 // map can be used to convert one result to another
 // keyval is now a function that returns a parser
 let keyval = || (common::Ident,":",common::Quoted).map(|(a,_,c)|(a,c));

 //this can also be written as below for better type safety
 fn keyval2()->impl Parser<Out=(String,String)>{
    (common::Ident,":",common::Quoted).map(|(a,_,c)|(a,c))
 }

 // or as a macro KeyVal is now a struct like:
 // pub struct KeyVal;
 parser!{
    (KeyVal->(String,String))
    (common::Ident,":",common::Quoted).map(|(a,_,c)|(a,c))
 }
 
 //parse_s is a helper on Parsers
 let (k,v) = keyval().parse_s(r#"car:"mini""#).unwrap();
 assert_eq!(k,"car");
 assert_eq!(v,"mini");

 //this can now be combined with other parsers.
 // 'ig_then' combines 2 parsers and drops the result of the first
 // 'then_ig' drops the result of the second
 // 'sep_until will repeat the first term into a Vec, separated by the second
 //    until the final term.
 let obj = || "{".ig_then(sep_until_ig(keyval(),",","}"));

 let obs = obj().parse_s(r#"{cat:"Tiddles",dog:"Spot"}"#).unwrap();
 assert_eq!(obs[0],("cat".to_string(),"Tiddles".to_string()));

 ```
 ## CharBool

 CharBool is the trait for boolean char checks. It is auto implemented for:
 * Fn(char)->bool
 * char -- Returns true if the input matches the char
 * &'static str -- returns true if the str contains the input
 * several zero size types - Alpha,NumDigit,HexDigit,WS,WSL,Any
 * Tuples of up to 6 CharBools -- returning true if any of the members succeed

 This means you can combine them in tuples ```(Alpha,NumDigit,"_").char_bool(c)```
 will be true if any of them match

 

 CharBool also provides several helper methods which each return a parser
 * ```one(self)``` matches and returns exactly 1 character
 * ```plus(self)``` '+' requires at least 1 matches and ruturns a string
 * ```min_n(self,n:usize)```  requires at least n matches and ruturns a string
 * ```star(self)``` '*' matches any number of chars returning a string
 * ```exact(self,n:usize)``` '*' matches exactly n chars returning a string
 * ```iplus(self)``` '+' requires at least 1 matches and ruturns a ()
 * ```istar(self)``` '*' matches any number of chars returning a ()
 * ```iexact(self,n:usize)``` matches exactly n chars returning a ()
 
 And a helper that returns a CharBool
 * ```except(self,cb:CharBool)``` Passes if self does, and cb doesnt
```rust
use gobble::*;
let s = |c| c > 'w' || c == 'z';
let xv = s.one().parse_s("xhello").unwrap();
assert_eq!(xv,'x');

let id = (Alpha,"_*").min_n(4).parse_s("sm*shing_game+you").unwrap();
assert_eq!(id,"sm*shing_game");

// not enough matches
assert!((NumDigit,"abc").min_n(4).parse_s("23fflr").is_err());

// any succeeds even with no matches equivilent to min_n(0) but "Zero Size"
assert_eq!((NumDigit,"abc").star().parse_s("23fflr"),Ok("23".to_string()));
assert_eq!((NumDigit,"abc").star().parse_s("fflr"),Ok("".to_string()));

```

## White Space

White space is pretty straight forward to handle

```rust
use gobble::*;
let my_ws = || " \t".star();
// middle takes three parsers and returns the result of the middle
// this could also be done easily with 'map' or 'then_ig'
let my_s = |p| middle(my_ws(),p,my_ws());

let sp_id = my_s(common::Ident);
let v = sp_id.parse_s("   \t  doggo  ").unwrap();
assert_eq!(v,"doggo");
```
That said gobble already provides ```WS``` and ```s_(p)```

```rust
use gobble::*;
//eoi = end of input
let p = repeat_until_ig(s_("abc".plus()),eoi);
let r = p.parse_s("aaa \tbbb bab").unwrap();
assert_eq!(r,vec!["aaa","bbb","bab"]);
```

## Recursive Structures

Some structures like Json, or programming languages need to be able to
handle recursion. However with the techniques we have used so far
this would lead to infinitely sized structures.

The way to handle this is to make sure one member of the loop is not  
build into the structure. Instead to create it using the 'Fn' or with a macro which will return a zero sized struct for certain

```rust
use gobble::*;
#[derive(Debug,PartialEq)]
enum Expr {
    Val(isize),
    Add(Box<Expr>,Box<Expr>),
    Paren(Box<Expr>),
}

fn expr_l()->impl Parser<Out=Expr>{
    or(
        middle("(",s_(expr),")").map(|e|Expr::Paren(Box::new(e))),
        common::Int.map(|v|Expr::Val(v))
    )
}

// using the full fn def we avoid the recursive structure
fn expr<'a>(it:&LCChars<'a>)->ParseRes<'a,Expr> {
    //note that expr_l has brackets but expr doesnt.
    //expr is a reference to a static function
    let p = (expr_l(),maybe(s_("+").ig_then(expr)))
        .map(|(l,opr)|match opr{
            Some(r)=>Expr::Add(Box::new(l),Box::new(r)),
            None=>l,
        });
    

    p.parse(it)
}

let r = expr.parse_s("45 + (34+3 )").unwrap();

//recursive structures are never fun to write manually
assert_eq!(r,Expr::Add(
                Box::new(Expr::Val(45)),
                Box::new(Expr::Paren(Box::new(Expr::Add(
                    Box::new(Expr::Val(34)),
                    Box::new(Expr::Val(3))
                ))))
            ));

```


## Changelog:

### v 0.6.3

* Added a traits module for exporting traits only;

### v 0.6.2
* Fix a bug with non ascii chars 

### v 0.6.1
* Put back the 'longer' method on ParseErr so that Or wouldn't give crazy long errors

### v 0.6.0
* Updated Error to contain &strs for the Found part of the result;
  This change is technically breaking, but I do not expect it to break any real code.

### v 0.5.3
* Added Hash to error type
* Added Exists method
* Added not method for charbool
* Added or_ig!() to ig the result of all inner parsers.

### v 0.5.2 
* Added catch for repeats on zero length parse results
* StrungErr now implements PartialEq


### v 0.5.1
* Added auto docs to the macros, hope to improve these soon

### v 0.5.0
* Added macros -- Unexpectedly, Somewhat unexpectedly. see the Docs
* swapped skip_star to istar etc for charbools
* Added StrungError and StrError, which can print themselves with great information

### v 0.4.4:
* Added ```skip_star(p)```
* Added ```skip_plus(p)```
* Added ```skip_exact(p,n)```

### v 0.4.3:
* Added ```string<A:Parser>(a:A)->impl Parser<String>``` to create a parser that reads the internal parser but returns the whole string it matched on

### v 0.4.2:
* Skip2Star added for skipping 2 parsers of different types

### v 0.4.1:
* derives Hash and Eq for Error

### v 0.4.0:
* Now uses "star" and "plus" for CharBool Repeats instead of "min_n" and "any"
* Now requires successes to declare if they could have continued with correct input
* Now has clearer errors containing info on how to find them, and what they expected next 


### v 0.3.0: Breaking Changes 
*Now Parser output is a trait associated type (Out)
use ```impl Parser<Out=V>``` instead of ```impl Parser<V>``` and most things should work
* read_fs removed - use CharBool.min_n(usize) instead
* Esc removed - see common::common_str for how to handle escapes


### v 0.2.1 :
* Added StringRepeat
* added SkipRepeat
* switched LCChars to use CharIndices
* now has index parser
* Added skip and skip_min to CharBool
* Added StrPos Parser str_pos


### v 0.2.0 -- Major update:
* created a new trait called CharBool
* removed is_alpha_num
* Added Character readers, that take use the CharBool trait to get what they want


### v 0.1.6:
* Added line_col wrappper to get the line and column of the result
* Added ```one_char(&str)```  Parser to check the next char is a member of that.

### v 0.1.5 :
* Added common_float method
* impl Parser for char and &'static str
* made tuples work as combinator parsers


### v 0.1.4: 
* Added keyword to make sure there are no alpha num characters on the end of the keyword
* Fixed the error display method to make them easier to read.
* added a 'common' module and ```common_int``` and ```common_bool``` parsers

### v 0.1.3:
* Added reflect functionality for when you need to count up and down again

### v 0.1.2 : 
* Added  ```sep_until(main,sep,close)```
* Added ```repeat_until(main,close)```
* Fixed Or Error to include both errors to make it easier to find the problems in branching iterators

### v 0.1.1 :

* Added ```eoi``` and ```to_end()``` functions for making sure you have the end of the input;
* Added ```common_str()``` for getting the most common form of string
