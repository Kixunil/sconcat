// Copyright © 2017 Trevor Spiteri

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # String concatenation
//!
//! Concatenatation of characters ([`char`][char]), string slices
//! ([`&str`][str]) and owned strings ([`String`][String]).
//!
//! A concatenation is started with the [`CAT`][CAT] constant, and any
//! number of characters, string slices or strings can be concatenated
//! using the `+` operator. The concatenation can be converted or
//! appended to a `String`.
//!
//! If the concatenation is converted to a `String`, and it starts
//! with an owned string with enough capacity to store the result, no
//! allocations or reallocations take place. If the concatenation is
//! appended to a `String` with enough capacity, no allocations or
//! reallocations take place. Otherwise, one allocation or
//! reallocation takes place.
//!
//! ## Examples
//!
//! A concatenation can be converted to a `String`.
//!
//! ```rust
//! use sconcat::CAT;
//!
//! let cat1 = CAT + "Hello, " + "world! " + '☺';
//! // One allocation:
//! let s1 = String::from(cat1);
//! assert_eq!(s1, "Hello, world! ☺");
//!
//! let cat2 = CAT + String::from("Hello, ") + "world! " + '☺';
//! // At most one reallocation as the initial `String` is resized:
//! let s2 = String::from(cat2);
//! assert_eq!(s2, "Hello, world! ☺");
//! ```
//!
//! A concatenation can also be appended to a `String`.
//!
//! ```rust
//! use sconcat::CAT;
//!
//! let cat = CAT + "world! " + '☺';
//! let mut s = String::from("Hello, ");
//! // At most one reallocation as the initial `s` is resized:
//! s += cat;
//! assert_eq!(s, "Hello, world! ☺");
//! ```
//!
//! If the concatenation starts with a `String` that has enough
//! reserved space, no reallocations will take place.
//!
//! ```rust
//! use sconcat::CAT;
//!
//! let mut buf = String::from("Hello, ");
//! // 7 bytes for "world! " and 3 bytes for '☺'
//! buf.reserve(10);
//! let ptr = buf.as_ptr();
//! let cat = CAT + buf + "world! " + '☺';
//! let s2 = String::from(cat);
//! assert_eq!(s2, "Hello, world! ☺");
//! assert_eq!(s2.as_ptr(), ptr);
//! ```
//!
//! The concatenation also implements [`Display`][Display] and
//! [`Debug`][Debug]. However, using `to_string()` can result in
//! multiple reallocations, so `String::from(cat)` is preferred over
//! `cat.to_string()` where possible.
//!
//! ```rust
//! use sconcat::CAT;
//!
//! let cat = CAT + "Hello, " + "world! " + '☺';
//! // `s1` can be resized up to three times:
//! let s1 = cat.to_string();
//! assert_eq!(s1, "Hello, world! ☺");
//! // Only one allocation here:
//! let s2 = String::from(cat);
//! assert_eq!(s2, "Hello, world! ☺");
//! // The following would fail as now `cat` has been moved:
//! // let s3 = String::from(cat);
//! ```
//!
//! ## Usage
//!
//! To use `sconcat` in your crate, add `extern crate sconcat;` to the
//! crate root and add `sconcat` as a dependency in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! sconcat = "0.1"
//! ```
//!
//! [CAT]:     constant.CAT.html
//! [Debug]:   https://doc.rust-lang.org/std/fmt/trait.Debug.html
//! [Display]: https://doc.rust-lang.org/std/fmt/trait.Display.html
//! [String]:  https://doc.rust-lang.org/std/string/struct.String.html
//! [char]:    https://doc.rust-lang.org/std/primitive.char.html
//! [str]:     https://doc.rust-lang.org/std/primitive.str.html

#[cfg(feature = "fast_fmt")]
extern crate fast_fmt;

mod cat;
pub use cat::CAT;

#[cfg(test)]
mod tests {
    use CAT;

    #[test]
    fn readme_example_works() {
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
    }
}
