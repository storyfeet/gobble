#Gobble

## A not too generic parser for strings in Rust

This parser exists take some of the generics and macros pain out of parsing.  It is surprisingly declarative for something that is just a bunch of ```then()```s and ```or()```s

A parser is anything that implements the Parser Trait

```rust

pub type ParseRes<'a, V> = Result<(LCChars<'a>, V), ParseError>;

pub trait Parser<V>: Sized {
    fn parse<'a>(&self, i: &LCChars<'a>) -> ParseRes<'a, V>;
    //...
}
```
This is automatically implemented for any function with the same signature.


LCChars is a wrapper around the "Chars" iterator which tracks line number and column number.
This is to help return the correct errors.

the main.rs file is an example parser.

Mostly you will be combining functions with ```then()```, ```ig_then()```, ```then_ig()```, and ```or()```

## Example 1:

```rust
use gobble::*;
pub fn ident()=>impl Parser<String>{
    read_fs(is_alpha,1)
        .then(read_fs(is_alpha_num,0))
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


## Changelog:

### v 0.1.4: --wip
Added keyword to make sure there are no alpha num characters on the end of the keyword

### v 0.1.3:
Added reflect functionality for when you need to count up and down again

### v 0.1.2 : 
Added  ```sep_until(main,sep,close)```
Added ```repeat_until(main,close)```
Fixed Or Error to include both errors to make it easier to find the problems in branching iterators

### v 0.1.1 :

Added ```eoi``` and ```to_end()``` functions for making sure you have the end of the input;
Also added ```common_str()``` for getting the most common form of string
