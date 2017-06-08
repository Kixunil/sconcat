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
//! If the concatenation contains at least one owned string, the
//! leftmost owned string will be resized to fit the whole
//! concatentation, and the result will be stored in this string. The
//! space is allocated once in the beginning, so at most one
//! reallocation takes place.
//!
//! ## Examples
//!
//! A concatenation can be converted to a `String`.
//!
//! ```rust
//! use scat::SCAT;
//!
//! let cat1 = SCAT + "hello, " + "world! " + '☺';
//! // One allocation and no following reallocations:
//! let s1 = String::from(cat1);
//! assert_eq!(s1, "hello, world! ☺");
//!
//! // The owned `String` will be reused.
//! let cat2 = SCAT + "hello, " + String::from("world! ") + '☺';
//! // At most one reallocation:
//! let s2 = String::from(cat2);
//! assert_eq!(s2, "hello, world! ☺");
//! ```
//!
//! A concatenation can also be appended to a `String`.
//!
//! ```rust
//! use scat::SCAT;
//!
//! let cat = SCAT + "hello, " + "world! " + '☺';
//! let mut s = String::new();
//! // At most one reallocation:
//! s += cat;
//! assert_eq!(s, "hello, world! ☺");
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
//! let cat = SCAT + "hello, " + "world! " + s1;
//! let s2 = String::from(cat);
//! assert_eq!(s2, "hello, world! ☺");
//! assert_eq!(s2.as_ptr(), ptr);
//! ```
//!
//! The concatenation also implements [`Display`][Display]. Using
//! `to_string()` will create a copy of the concatenation. If the
//! concatenation is not to be used again, prefer `String::from(cat)`
//! to `cat.to_string()`.
//!
//! ```rust
//! use scat::SCAT;
//!
//! let cat = SCAT + "hello, " + "world! " + '☺';
//! let s1 = cat.to_string();
//! assert_eq!(s1, "hello, world! ☺");
//! let s2 = String::from(cat);
//! assert_eq!(s2, "hello, world! ☺");
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
