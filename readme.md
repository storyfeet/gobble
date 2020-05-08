#Gobble

## A parser for strings in Rust that is just Code. No Macros, and easy to use generics

This parser exists take some of the generics and macros pain out of parsing.  It is surprisingly declarative for something that is just a bunch of ```then()```s and ```or()```s

To Gobble : A parser is anything that implements it's Parser Trait

```rust

pub type ParseRes<'a, V> = Result<(LCChars<'a>, V), ParseError>;

pub trait Parser<V>: Sized {
    // LCChars is a chars iterator that tracks line and column
    // a non mut pointer to LCChars means if it fails the caller knows for sure it hasnt changed.
    //cloning the iterator is cheap
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, V>;
    //...
}
```
Parser is already implemented for :

* Any function matching ```Fn<'a>(&LCChars<'a>)->ParseRes<'a,V>```
* ```&'static str``` if matched exactly returns itself.
* ```char``` if matched exactly returns itself.
* Tuples of Parsers up to 6 items. Matching each member in turn, and returning a tuple of the results. (if you need more than 6 you can always nest them)

The clearest example is in examples/json.rs 

Mostly you will be combining functions with ```then()```, ```ig_then()```, ```then_ig()```, and ```or()```

## Example 1:

```rust
use gobble::*;
pub fn ident()=>impl Parser<String>{
    read_fs(is_alpha,1)
        .then(read_fs(is_alpha_num,0))
        //map converts the result to the correct type for the function
        .map(|(mut a, b)| {
            a.push_str(&b);
            a
    })
}
```

## Example 2:
Or more lazily with closures

```rust
use gobble::*;
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

 assert!(fsig.parse_s("23file(fname,ref)").is_err());
 
 ```
## How the combinators work
 
The function combinators work by returning a Struct that is generically typed to match the parsers it is given.
for example if 'a:A' and 'b:B' are parsers that both return a value of type 'V'. ```a.or(b)``` will return an ```Or<A,B,V>{a,b}```

```Or<A,B,V> implements ```Parser<V>```, and will try a first, but then b second if a fails

This means that writing a.or(b).or(c).or(d) Will return a Fixed size struct ```Or<Or<Or<A,B,V>,C>,D>```  It's not pretty but the compiler checks it an makes sure it works.

This does of course create a problem for recursive types such as Json, as the structure created would be an infinite size.

the solution is to write the definition of one part of your recursive loop yourself, but you can still use the other tricks to build it.

```
// An Excerpt from example/json.rs 
use gobble::*;
use examples::json::{json_string,wsn_,map_item}; //ish

pub fn json_value<'a>(it: &LCChars<'a>) -> ParseRes<'a, Value> {
    //create the parse using the builders combinators
    let p = or6(
        "null".map(|_| Value::Null),
        common_bool.map(|b| Value::Bool(b)),
        or(
            common_float.map(|f| Value::Num(f)),
            common_int.map(|i| Value::Num(i as f64)),
        ),
        json_string().map(|s| Value::Str(s)),
        // here we use json_value so returning a parser would create an infinite size object
        "[".ig_then(sep_until(wsn_(json_value), ",", "]"))
            .map(|a| Value::Array(a)),
        "{".ig_then(sep_until(wsn_(map_item()), ",", "}")).map(|a| {
            let mut m = HashMap::new();
            for (k, v) in a {
                m.insert(k, v);
            }
            Value::Object(m)
        }),
    );
    //by creating the parser inside the function we avoid having infinitely sized objects but we can still have it look PEG enough to read easily
    p.parse(it)
}
```

## Changelog:

### v 0.1.5 :
Added common_float method
impl Parser for char and &'static str
made tuples work as combinator parsers


### v 0.1.4: 
Added keyword to make sure there are no alpha num characters on the end of the keyword
Fixed the error display method to make them easier to read.
added a 'common' module and ```common_int``` and ```common_bool``` parsers

### v 0.1.3:
Added reflect functionality for when you need to count up and down again

### v 0.1.2 : 
Added  ```sep_until(main,sep,close)```
Added ```repeat_until(main,close)```
Fixed Or Error to include both errors to make it easier to find the problems in branching iterators

### v 0.1.1 :

Added ```eoi``` and ```to_end()``` functions for making sure you have the end of the input;
Also added ```common_str()``` for getting the most common form of string
