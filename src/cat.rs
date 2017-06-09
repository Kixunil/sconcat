// Copyright © 2017 Trevor Spiteri

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::{self, Debug, Display};
use std::ops::{Add, AddAssign};

/// Trait for types that can be concatenated.
pub trait Cat {
    /// Length of item in bytes.
    fn size_hint(&self) -> usize;
    /// Append item to String.
    fn append_to(&self, s: &mut String);
    /// Converts item to a String.
    fn into_string(self, capacity: usize) -> String;
}

impl<'a> Cat for char {
    fn size_hint(&self) -> usize {
        self.len_utf8()
    }

    fn append_to(&self, s: &mut String) {
        s.push(*self);
    }

    fn into_string(self, capacity: usize) -> String {
        let mut s = String::with_capacity(capacity);
        s.push(self);
        s
    }
}

impl<'a> Cat for &'a str {
    fn size_hint(&self) -> usize {
        self.len()
    }

    fn append_to(&self, s: &mut String) {
        s.push_str(self);
    }

    fn into_string(self, capacity: usize) -> String {
        let mut s = String::with_capacity(capacity);
        s.push_str(self);
        s
    }
}

impl Cat for String {
    fn size_hint(&self) -> usize {
        self.len()
    }

    fn append_to(&self, s: &mut String) {
        s.push_str(self)
    }

    fn into_string(mut self, capacity: usize) -> String {
        let len = self.len();
        if capacity > len {
            self.reserve(capacity - len);
        }
        self
    }
}

#[derive(Clone)]
pub struct CatMany<L: Cat, R: Cat> {
    lhs: L,
    rhs: R,
}

impl<L: Cat + Copy, R: Cat + Copy> Copy for CatMany<L, R> {}

impl<L: Cat, R: Cat> Cat for CatMany<L, R> {
    fn size_hint(&self) -> usize {
        self.lhs
            .size_hint()
            .checked_add(self.rhs.size_hint())
            .expect("capacity overflow")
    }

    fn append_to(&self, s: &mut String) {
        self.lhs.append_to(s);
        self.rhs.append_to(s);
    }

    fn into_string(self, capacity: usize) -> String {
        let mut s = self.lhs.into_string(capacity);
        self.rhs.append_to(&mut s);
        s
    }
}

impl<L: Cat, R: Cat> Add<CatNone> for CatMany<L, R> {
    type Output = CatMany<L, R>;
    fn add(self, _rhs: CatNone) -> CatMany<L, R> {
        self
    }
}

impl<L: Cat, R: Cat, RR: Cat> Add<CatOne<RR>> for CatMany<L, R> {
    type Output = CatMany<CatMany<L, R>, RR>;
    fn add(self, rhs: CatOne<RR>) -> CatMany<CatMany<L, R>, RR> {
        CatMany {
            lhs: self,
            rhs: rhs.inner,
        }
    }
}

impl<L: Cat, R: Cat, RR: Cat> Add<RR> for CatMany<L, R> {
    type Output = CatMany<CatMany<L, R>, RR>;
    fn add(self, rhs: RR) -> CatMany<CatMany<L, R>, RR> {
        CatMany {
            lhs: self,
            rhs: rhs,
        }
    }
}

impl<L: Cat, R: Cat> AddAssign<CatMany<L, R>> for String {
    fn add_assign(&mut self, rhs: CatMany<L, R>) {
        self.reserve(rhs.size_hint());
        rhs.append_to(self);
    }
}

impl<L: Cat, R: Cat> From<CatMany<L, R>> for String {
    fn from(src: CatMany<L, R>) -> String {
        let capacity = src.size_hint();
        src.into_string(capacity)
    }
}

impl<L: Cat + Debug, R: Cat + Debug> Debug for CatMany<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.lhs, f)?;
        Display::fmt(" + ", f)?;
        Debug::fmt(&self.rhs, f)
    }
}

impl<L: Cat + Display, R: Cat + Display> Display for CatMany<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.lhs, f)?;
        Display::fmt(&self.rhs, f)
    }
}

#[derive(Clone)]
pub struct CatOne<T: Cat> {
    inner: T,
}

impl<T: Cat + Copy> Copy for CatOne<T> {}

impl<T: Cat> Add<CatNone> for CatOne<T> {
    type Output = CatOne<T>;
    fn add(self, _rhs: CatNone) -> CatOne<T> {
        self
    }
}

impl<L: Cat, R: Cat> Add<CatOne<R>> for CatOne<L> {
    type Output = CatMany<L, R>;
    fn add(self, rhs: CatOne<R>) -> CatMany<L, R> {
        CatMany {
            lhs: self.inner,
            rhs: rhs.inner,
        }
    }
}

impl<L: Cat, R: Cat> Add<R> for CatOne<L> {
    type Output = CatMany<L, R>;
    fn add(self, rhs: R) -> CatMany<L, R> {
        CatMany {
            lhs: self.inner,
            rhs: rhs,
        }
    }
}

impl<T: Cat> AddAssign<CatOne<T>> for String {
    fn add_assign(&mut self, rhs: CatOne<T>) {
        self.reserve(rhs.inner.size_hint());
        rhs.inner.append_to(self);
    }
}

impl<T: Cat> From<CatOne<T>> for String {
    fn from(src: CatOne<T>) -> String {
        let capacity = src.inner.size_hint();
        src.inner.into_string(capacity)
    }
}

impl<T: Cat + Debug> Debug for CatOne<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl<T: Cat + Display> Display for CatOne<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

#[derive(Clone, Copy)]
pub struct CatNone;

/// A term that is used to start a string concatenation.
///
/// See the [crate documentation](index.html).
///
/// # Examples
///
/// ```rust
/// use sconcat::CAT;
///
/// let cat = CAT + "Hello, " + "world! " + '☺';
/// let s = String::from(cat);
/// assert_eq!(s, "Hello, world! ☺");
///
/// let mut s2 = String::from("Hello");
/// s2 += CAT + ',' + " world" + String::from("! ") + '☺';
/// assert_eq!(s2, "Hello, world! ☺");
/// ```
pub const CAT: CatNone = CatNone;

impl Add<CatNone> for CatNone {
    type Output = CatNone;
    fn add(self, _rhs: CatNone) -> CatNone {
        self
    }
}

impl<T: Cat> Add<CatOne<T>> for CatNone {
    type Output = CatOne<T>;
    fn add(self, rhs: CatOne<T>) -> CatOne<T> {
        rhs
    }
}

impl<T: Cat> Add<T> for CatNone {
    type Output = CatOne<T>;
    fn add(self, rhs: T) -> CatOne<T> {
        CatOne { inner: rhs }
    }
}

impl AddAssign<CatNone> for String {
    fn add_assign(&mut self, _rhs: CatNone) {}
}

impl From<CatNone> for String {
    fn from(_src: CatNone) -> String {
        String::new()
    }
}

impl Debug for CatNone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt("\"\"", f)
    }
}

impl Display for CatNone {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use CAT;

    #[test]
    fn it_works() {
        let cat = CAT + "Hello, " + String::from("world");
        assert_eq!(cat.to_string(), "Hello, world");
        assert_eq!(String::from(cat), "Hello, world");

        let mut s = String::new();
        s.reserve(20);
        let ptr = s.as_ptr();
        s += CAT + "12345" + "67890" + '1' + String::from("2345") + "67890";
        assert_eq!(s, "12345678901234567890");
        assert_eq!(s.as_ptr(), ptr);
    }

    #[test]
    fn formatting() {
        let cat0 = CAT;
        assert_eq!(format!("{}", cat0), "");
        assert_eq!(format!("{:?}", cat0), "\"\"");
        let cat1 = cat0 + "Hello, ";
        assert_eq!(format!("{}", cat1), "Hello, ");
        assert_eq!(format!("{:?}", cat1), "\"Hello, \"");
        let cat2 = cat1 + "world! ";
        assert_eq!(format!("{}", cat2), "Hello, world! ");
        assert_eq!(format!("{:?}", cat2), "\"Hello, \" + \"world! \"");
        let cat3 = cat2 + '☺';
        assert_eq!(format!("{}", cat3), "Hello, world! ☺");
        assert_eq!(format!("{:?}", cat3), "\"Hello, \" + \"world! \" + '☺'");
    }
}
