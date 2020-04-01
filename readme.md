#Gobble

## A not too generic parser for strings in Rust

This parser exists take some of the generics and macros pain out of parsing. 

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

Mostly you will be combining functions with ```then()```, ```ig_then()```, ``then_ig()```, and ```or()```

## Example:

```rust
/// A simple parser for a function call
fn parse_f_call<'a>(LCChars<'a>)->ParseRes<'a,(String,Vec<String>)>{
    let ident = read_fs(is_alpha).then(read_fs(is_alpha))
    read_fs(is_alphanum)
}
```


