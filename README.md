# String concatenation

The `sconcat` crate provides concatenation of characters, string
slices and owned strings.

A concatenation is started with the `CAT` constant, and any number of
characters, string slices or strings can be concatenated using the `+`
operator. The concatenation can be converted or appended to a
`String`.

If the concatenation contains at least one owned string, the
leftmost owned string will be resized to fit the whole
concatentation, and the result will be stored in this string. The
space is allocated once in the beginning, so at most one
reallocation takes place.

This crate is free software licensed under the
[Apache License, Version 2.0][apache] or the [MIT license][mit], at
your option.

## Basic use

[Documentation][doc] for this crate is available. The crate provides
one constant, `CAT`, that can be used to start a concatenation
expression. The concatenation can then be converted or appended to a
`String`. The final length is computed in the beginning so that at
most one allocation or reallocation takes place.

## Examples

```rust
use sconcat::CAT;

let cat1 = CAT + "Hello, " + "world! " + '☺';
let s1 = String::from(cat1);
assert_eq!(s1, "Hello, world! ☺");

let mut s2 = String::from("Hello");
s2 += CAT + ',' + " world" + String::from("! ") + '☺';
assert_eq!(s2, "Hello, world! ☺");

let mut buf = String::from("Hello, ");
// 7 bytes for "world! " and 3 bytes for '☺'
buf.reserve(10);
let ptr = buf.as_ptr();
// buf is large enough, so no reallocations take place
let cat3 = CAT + buf + "world! " + '☺';
let s3 = String::from(cat3);
assert_eq!(s3, "Hello, world! ☺");
assert_eq!(s3.as_ptr(), ptr);
```

## Usage

To use `sconcat` in your crate, add `extern crate sconcat;` to the
crate root and add `sconcat` as a dependency in `Cargo.toml`:

```toml
[dependencies]
sconcat = "0.1"
```

[apache]: https://www.apache.org/licenses/LICENSE-2.0
[doc]:    https://docs.rs/sconcat/
[mit]:    https://opensource.org/licenses/MIT
