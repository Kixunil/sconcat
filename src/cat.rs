// Copyright © 2017 Trevor Spiteri

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cmp;
use std::fmt::{self, Debug, Display};
use std::mem;
use std::ops::{Add, AddAssign};
use std::ptr;
use std::slice;

/// Trait for types that can be concatenated.
///
/// # Safety
///
/// This trait is unsafe because returning a value which is too small
/// from `max_len()` can lead to writing to unallocated memory.
pub unsafe trait Cat {
    /// Maximum number of bytes that will be written.
    fn max_len(&self) -> usize;
    /// Maximum capacity of available Vec<u8>.
    fn largest_owned(&self) -> usize;
    /// Write up to `self.max_len()` bytes at `ptr`, returning the
    /// real length.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it writes to a pointer.
    unsafe fn write_to(&self, ptr: *mut u8) -> usize;
    /// Convert to Vec<u8> with `before` uninitialized bytes at the
    /// beginning and `after` uninitialized bytes at the end.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it return uninitialized
    /// memory.
    unsafe fn to_vec_with_uninit(self, before: usize, after: usize) -> Vec<u8>;
}

unsafe impl<'a> Cat for char {
    fn max_len(&self) -> usize {
        self.len_utf8()
    }

    fn largest_owned(&self) -> usize {
        0
    }

    unsafe fn write_to(&self, dst: *mut u8) -> usize {
        let len = self.len_utf8();
        let dst_slice = slice::from_raw_parts_mut(dst, len);
        self.encode_utf8(dst_slice);
        len
    }

    unsafe fn to_vec_with_uninit(self, before: usize, after: usize) -> Vec<u8> {
        let capacity = self.len_utf8()
            .checked_add(before)
            .and_then(|a| a.checked_add(after))
            .expect("capacity overflow");
        let mut v = Vec::<u8>::with_capacity(capacity);
        v.set_len(capacity);
        let dst = v.as_mut_ptr().wrapping_offset(before as isize);
        self.write_to(dst);
        v
    }
}

unsafe impl<'a> Cat for &'a str {
    fn max_len(&self) -> usize {
        self.len()
    }

    fn largest_owned(&self) -> usize {
        0
    }

    unsafe fn write_to(&self, dst: *mut u8) -> usize {
        let src = self.as_bytes().as_ptr();
        let count = self.len();
        ptr::copy_nonoverlapping(src, dst, count);
        count
    }

    unsafe fn to_vec_with_uninit(self, before: usize, after: usize) -> Vec<u8> {
        let capacity = self.len()
            .checked_add(before)
            .and_then(|a| a.checked_add(after))
            .expect("capacity overflow");
        let mut v = Vec::<u8>::with_capacity(capacity);
        v.set_len(capacity);
        let dst = v.as_mut_ptr().wrapping_offset(before as isize);
        self.write_to(dst);
        v
    }
}

unsafe impl Cat for String {
    fn max_len(&self) -> usize {
        self.len()
    }

    fn largest_owned(&self) -> usize {
        self.capacity()
    }

    unsafe fn write_to(&self, dst: *mut u8) -> usize {
        let src = self.as_bytes().as_ptr();
        let count = self.len();
        ptr::copy_nonoverlapping(src, dst, count);
        count
    }

    unsafe fn to_vec_with_uninit(self, before: usize, after: usize) -> Vec<u8> {
        let additional = before.checked_add(after).expect("capacity overflow");
        let mut v = self.into_bytes();
        let count = v.len();
        v.reserve(additional);
        // reserve() already makes sure we have enough space
        v.set_len(count + additional);
        // this copy can overlap
        let src = v.as_ptr();
        let dst = v.as_mut_ptr().wrapping_offset(before as isize);
        ptr::copy(src, dst, count);
        v
    }
}

#[derive(Clone)]
pub struct CatMany<L: Cat, R: Cat> {
    lhs: L,
    rhs: R,
}

impl<L: Cat + Copy, R: Cat + Copy> Copy for CatMany<L, R> {}

unsafe impl<L: Cat, R: Cat> Cat for CatMany<L, R> {
    fn max_len(&self) -> usize {
        self.lhs
            .max_len()
            .checked_add(self.rhs.max_len())
            .expect("capacity overflow")
    }

    fn largest_owned(&self) -> usize {
        cmp::max(self.lhs.largest_owned(), self.rhs.largest_owned())
    }

    unsafe fn write_to(&self, dst: *mut u8) -> usize {
        let len_lhs = self.lhs.write_to(dst);
        let dst_rhs = dst.wrapping_offset(len_lhs as isize);
        let len_rhs = self.rhs.write_to(dst_rhs);
        len_lhs + len_rhs
    }

    unsafe fn to_vec_with_uninit(self, before: usize, after: usize) -> Vec<u8> {
        let max_len_lhs = self.lhs.max_len();
        let max_len_rhs = self.rhs.max_len();
        let largest_lhs = self.lhs.largest_owned();
        let largest_rhs = self.rhs.largest_owned();
        let req_cap = before
            .checked_add(max_len_lhs)
            .and_then(|a| a.checked_add(max_len_rhs))
            .and_then(|a| a.checked_add(after))
            .expect("capacity overflow");

        // * If lhs has enough capacity, use lhs: no reallocations,
        //   and no need to move bytes.
        // * Else if rhs has enough capacity, use rhs: no
        //   reallocations, but may need to move bytes up to two
        //   times: once so that the lhs won't overwrite the rhs, and
        //   once if the actual lenght of lhs < max_len_lhs.
        // * Else use lhs: do *not* use rhs, otherwise we may pay the
        //   penalty above and still need to reallocate memory.
        if largest_lhs >= req_cap || largest_rhs < req_cap {
            let after_lhs =
                after.checked_add(max_len_rhs).expect("capacity overflow");
            let mut v = self.lhs.to_vec_with_uninit(before, after_lhs);
            let before_rhs = v.len() - after_lhs;
            let dst_rhs = v.as_mut_ptr().wrapping_offset(before_rhs as isize);
            let len_rhs = self.rhs.write_to(dst_rhs);
            v.set_len(before_rhs + len_rhs + after);
            v
        } else {
            let before_rhs =
                before.checked_add(max_len_lhs).expect("capacity overflow");
            let mut v = self.rhs.to_vec_with_uninit(before_rhs, after);
            let len_rhs = v.len() - before_rhs - after;
            let dst_lhs = v.as_mut_ptr().wrapping_offset(before as isize);
            let len_lhs = self.lhs.write_to(dst_lhs);
            let excess = max_len_lhs - len_lhs;
            if excess > 0 {
                let src = v.as_ptr().wrapping_offset(before_rhs as isize);
                let dst = v.as_mut_ptr()
                    .wrapping_offset((before_rhs - excess) as isize);
                ptr::copy(src, dst, len_rhs);
            }
            v.set_len(before_rhs - excess + len_rhs + after);
            v
        }
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

impl<L: Cat, R: Cat> From<CatMany<L, R>> for String {
    fn from(src: CatMany<L, R>) -> String {
        unsafe {
            let v = src.to_vec_with_uninit(0, 0);
            String::from_utf8_unchecked(v)
        }
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

impl<L: Cat, R: Cat> AddAssign<CatMany<L, R>> for String {
    fn add_assign(&mut self, rhs: CatMany<L, R>) {
        // steal String from borrow
        let mut steal = String::new();
        mem::swap(self, &mut steal);
        let cat = CatMany::<String, CatMany<L, R>> {
            lhs: steal,
            rhs: rhs,
        };
        steal = cat.into();
        mem::swap(self, &mut steal);
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

impl<T: Cat> From<CatOne<T>> for String {
    fn from(src: CatOne<T>) -> String {
        unsafe {
            let v = src.inner.to_vec_with_uninit(0, 0);
            String::from_utf8_unchecked(v)
        }
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

impl<T: Cat> AddAssign<CatOne<T>> for String {
    fn add_assign(&mut self, rhs: CatOne<T>) {
        // steal String from borrow
        let mut steal = String::new();
        mem::swap(self, &mut steal);
        let cat = CatMany::<String, T> {
            lhs: steal,
            rhs: rhs.inner,
        };
        steal = cat.into();
        mem::swap(self, &mut steal);
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

impl AddAssign<CatNone> for String {
    fn add_assign(&mut self, _rhs: CatNone) {}
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
