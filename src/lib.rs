// Copyright © 2017 Trevor Spiteri

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # String concatenation
//!
//! Concatenatation of [characters][char], [string slices][str] and
//! [owned strings][String].
//!
//! A concatenation is started with the [`SCAT`][SCAT] constant, and
//! any number of characters, string slices or strings can be
//! concatenated using the `+` operator. The concatenation can be
//! converted or appended to a [`String`][String].
//!
//! If the concatenation contains at least one owned string with
//! enough capacity to store the result, the leftmost such string will
//! be resized to hold the result and no allocations or reallocations
//! take place.
//!
//! If no term is an owned string with enough capacity, one allocation
//! or reallocation takes place: if the first term is an owned string,
//! it will be resized to hold the result, otherwise a new owned
//! string with enough capacity is created.
//!
//! ## Examples
//!
//! A concatenation can be converted to a `String`.
//!
//! ```rust
//! use scat::SCAT;
//!
//! let cat1 = SCAT + "Hello, " + "world! " + '☺';
//! // One allocation and no following reallocations:
//! let s1 = String::from(cat1);
//! assert_eq!(s1, "Hello, world! ☺");
//!
//! let cat2 = SCAT + String::from("Hello, ") + "world! " + '☺';
//! // At most one reallocation as the initial `String` is resized:
//! let s2 = String::from(cat2);
//! assert_eq!(s2, "Hello, world! ☺");
//! ```
//!
//! A concatenation can also be appended to a `String`.
//!
//! ```rust
//! use scat::SCAT;
//!
//! let cat = SCAT + "world! " + '☺';
//! let mut s = String::from("Hello, ");
//! // At most one reallocation as the initial `s` is resized:
//! s += cat;
//! assert_eq!(s, "Hello, world! ☺");
//! ```
//!
//! If a `String` has enough reserved space, no reallocations will
//! take place.
//!
//! ```rust
//! use scat::SCAT;
//!
//! let mut s1 = String::from("☺");
//! s1.reserve(14);
//! let ptr = s1.as_ptr();
//! let cat = SCAT + "Hello, " + "world! " + s1;
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
//! use scat::SCAT;
//!
//! let cat = SCAT + "Hello, " + "world! " + '☺';
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
//! To use `scat` in your crate, add `extern crate scat;` to the crate
//! root and add `scat` as a dependency in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! scat = "0.1"
//! ```
//!
//! [char]:    https://doc.rust-lang.org/std/primitive.char.html
//! [str]:     https://doc.rust-lang.org/std/primitive.str.html
//! [String]:  https://doc.rust-lang.org/std/string/struct.String.html
//! [Display]: https://doc.rust-lang.org/std/fmt/trait.Display.html
//! [SCAT]:    constant.SCAT.html

mod cat;
pub use cat::SCAT;

#[cfg(test)]
mod tests {
    use SCAT;

    #[test]
    fn readme_example_works() {
        let cat1 = SCAT + "Hello, " + "world! " + '☺';
        let s1 = String::from(cat1);
        assert_eq!(s1, "Hello, world! ☺");

        let mut s2 = String::from("Hello");
        s2 += SCAT + ',' + " world" + String::from("! ") + '☺';
        assert_eq!(s2, "Hello, world! ☺");

        let mut buf = String::from("☺");
        buf.reserve(14);
        let ptr = buf.as_ptr();
        // buf is large enough, so no reallocations take place
        let cat3 = SCAT + "Hello, " + "world! " + buf;
        let s3 = String::from(cat3);
        assert_eq!(s3, "Hello, world! ☺");
        assert_eq!(s3.as_ptr(), ptr);
    }
}
