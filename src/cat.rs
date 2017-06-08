// Copyright © 2017 Trevor Spiteri

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cmp;
use std::fmt::{self, Display};
use std::ops::{Add, AddAssign};
use std::ptr;
use std::slice;

pub unsafe trait Cat: Display {
    fn max_len(&self) -> usize;
    fn has_capacity(&self) -> usize;
    unsafe fn write_to(self, ptr: *mut u8) -> usize;
    unsafe fn to_vec_with_uninit(self, before: usize, after: usize) -> Vec<u8>;
}

unsafe impl<'a> Cat for char {
    fn max_len(&self) -> usize {
        self.len_utf8()
    }

    fn has_capacity(&self) -> usize {
        0
    }

    unsafe fn write_to(self, dst: *mut u8) -> usize {
        let len = self.len_utf8();
        let dst_slice = slice::from_raw_parts_mut(dst, len);
        self.encode_utf8(dst_slice);
        len
    }

    unsafe fn to_vec_with_uninit(self, before: usize, after: usize) -> Vec<u8> {
        let capacity = self.len_utf8()
            .checked_add(before)
            .expect("overflow")
            .checked_add(after)
            .expect("overflow");
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

    fn has_capacity(&self) -> usize {
        0
    }

    unsafe fn write_to(self, dst: *mut u8) -> usize {
        let src = self.as_bytes().as_ptr();
        let count = self.len();
        ptr::copy_nonoverlapping(src, dst, count);
        count
    }

    unsafe fn to_vec_with_uninit(self, before: usize, after: usize) -> Vec<u8> {
        let capacity = self.len()
            .checked_add(before)
            .expect("overflow")
            .checked_add(after)
            .expect("overflow");
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

    fn has_capacity(&self) -> usize {
        self.capacity()
    }

    unsafe fn write_to(self, dst: *mut u8) -> usize {
        let src = self.as_bytes().as_ptr();
        let count = self.len();
        ptr::copy_nonoverlapping(src, dst, count);
        count
    }

    unsafe fn to_vec_with_uninit(self, before: usize, after: usize) -> Vec<u8> {
        let additional = before.checked_add(after).expect("overflow");
        let mut v = self.into_bytes();
        let count = v.len();
        v.reserve(additional);
        // reserve already makes sure we have enough space
        v.set_len(count + additional);
        // this copy can overlap
        let src = v.as_ptr();
        let dst = v.as_mut_ptr().wrapping_offset(before as isize);
        ptr::copy(src, dst, count);
        v
    }
}

pub struct CatMany<L: Cat, R: Cat> {
    lhs: L,
    rhs: R,
}

unsafe impl<L: Cat, R: Cat> Cat for CatMany<L, R> {
    fn max_len(&self) -> usize {
        self.lhs
            .max_len()
            .checked_add(self.rhs.max_len())
            .expect("overflow")
    }

    fn has_capacity(&self) -> usize {
        cmp::max(self.lhs.has_capacity(), self.rhs.has_capacity())
    }

    unsafe fn write_to(self, dst: *mut u8) -> usize {
        let len_lhs = self.lhs.write_to(dst);
        let dst_rhs = dst.wrapping_offset(len_lhs as isize);
        let len_rhs = self.rhs.write_to(dst_rhs);
        len_lhs + len_rhs
    }

    unsafe fn to_vec_with_uninit(self, before: usize, after: usize) -> Vec<u8> {
        let max_len_lhs = self.lhs.max_len();
        let max_len_rhs = self.rhs.max_len();
        let cap_lhs = self.lhs.has_capacity();
        let cap_rhs = self.rhs.has_capacity();
        let cap_req = before
            .checked_add(max_len_lhs)
            .and_then(|a| a.checked_add(max_len_rhs))
            .and_then(|a| a.checked_add(after))
            .expect("overflow");
        if cap_lhs >= cap_req || (cap_rhs < cap_req && cap_lhs > 0) ||
            cap_rhs == 0
        {
            let after_lhs = after.checked_add(max_len_rhs).expect("overflow");
            let mut v = self.lhs.to_vec_with_uninit(before, after_lhs);
            let before_rhs = v.len() - after_lhs;
            let dst_rhs = v.as_mut_ptr().wrapping_offset(before_rhs as isize);
            let len_rhs = self.rhs.write_to(dst_rhs);
            v.set_len(before_rhs + len_rhs + after);
            v
        } else {
            let before_rhs = before.checked_add(max_len_lhs).expect("overflow");
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

impl<L: Cat, R: Cat> Display for CatMany<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.lhs, self.rhs)
    }
}

impl<L: Cat, R: Cat> AddAssign<CatMany<L, R>> for String {
    fn add_assign(&mut self, rhs: CatMany<L, R>) {
        let len_lhs = self.len();
        let max_len_rhs = rhs.max_len();
        let v = unsafe { self.as_mut_vec() };
        v.reserve(max_len_rhs);
        unsafe {
            v.set_len(len_lhs + max_len_rhs);
            let dst_rhs = v.as_mut_ptr().wrapping_offset(len_lhs as isize);
            let len_rhs = rhs.write_to(dst_rhs);
            v.set_len(len_lhs + len_rhs);
        }
    }
}

pub struct CatOne<T: Cat> {
    inner: T,
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

impl<T: Cat> Display for CatOne<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<T: Cat> AddAssign<CatOne<T>> for String {
    fn add_assign(&mut self, rhs: CatOne<T>) {
        let len_lhs = self.len();
        let max_len_rhs = rhs.inner.max_len();
        let v = unsafe { self.as_mut_vec() };
        v.reserve(max_len_rhs);
        unsafe {
            v.set_len(len_lhs + max_len_rhs);
            let dst_rhs = v.as_mut_ptr().wrapping_offset(len_lhs as isize);
            let len_rhs = rhs.inner.write_to(dst_rhs);
            v.set_len(len_lhs + len_rhs);
        }
    }
}

pub struct Scat;

/// A term that is used to start a string concatenation.
///
/// See the [crate documentation](index.html).
///
/// # Examples
///
/// ```rust
/// use scat::SCAT;
///
/// let cat = SCAT + "hello, " + "world! " + '☺';
/// let s = String::from(cat);
/// assert_eq!(s, "hello, world! ☺");
///
/// let mut s2 = String::from("hello");
/// s2 += SCAT + ',' + " world" + String::from("! ") + '☺';
/// assert_eq!(s2, "hello, world! ☺");
/// ```
pub const SCAT: Scat = Scat;

impl<T: Cat> Add<T> for Scat {
    type Output = CatOne<T>;
    fn add(self, rhs: T) -> CatOne<T> {
        CatOne { inner: rhs }
    }
}

impl From<Scat> for String {
    fn from(_src: Scat) -> String {
        String::new()
    }
}

impl Display for Scat {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl AddAssign<Scat> for String {
    fn add_assign(&mut self, _rhs: Scat) {}
}

#[cfg(test)]
mod tests {
    use cat::SCAT;
    #[test]
    fn it_works() {
        let z = SCAT + "hello, " + String::from("world");
        assert_eq!(z.to_string(), "hello, world");
        assert_eq!(String::from(z), "hello, world");

        let mut s = String::new();
        s.reserve(20);
        let ptr = s.as_ptr();
        s += SCAT + "12345" + "67890" + '1' + String::from("2345") + "67890";
        assert_eq!(s, "12345678901234567890");
        assert_eq!(s.as_ptr(), ptr);
    }
}
