// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[prelude_import] use base::prelude::*;
use base::rmo::{AsRef, AsMut};
use core::{mem};
use str_one::{NoNullStr, AsNoNullStr, AsMutNoNullStr, AsMutCStr, CStr};
use vec::{Vec};
use fmt::{Debug, Write};
use alloc::{self, Allocator};

pub struct NoNullString<'a, Heap = alloc::Heap>
    where Heap: Allocator,
{
    data: Vec<'a, u8, Heap>,
}

impl<H> NoNullString<'static, H>
    where H: Allocator,
{
    pub fn new() -> NoNullString<'static, H> {
        NoNullString { data: Vec::new() }
    }
}

impl<'a> NoNullString<'a, alloc::NoHeap> {
    pub fn buffered(buf: &'a mut [u8]) -> NoNullString<'a, alloc::NoHeap> {
        NoNullString { data: Vec::buffered(buf) }
    }
}

impl<'a, H> NoNullString<'a, H>
    where H: Allocator,
{
    /// Casts the byte vector directly to a `NoNullString` without checking it for validity.
    pub unsafe fn from_bytes_unchecked(bytes: Vec<'a, u8, H>) -> NoNullString<'a, H> {
        NoNullString { data: bytes }
    }

    pub fn clear(&mut self) {
        self.data.truncate(0);
    }

    pub fn push_file<F>(&mut self, name: F) -> Result
        where F: AsNoNullStr,
    {
        let bytes = try!(name.as_no_null_str()).as_ref();
        try!(self.data.reserve(bytes.len() + 1));
        self.data.push(b'/');
        self.data.try_push_all(bytes)
    }

    pub fn set_path<F>(&mut self, path: F) -> Result
        where F: AsNoNullStr,
    {
        let bytes = try!(path.as_no_null_str()).as_ref();
        self.clear();
        try!(self.data.reserve(bytes.len()));
        self.data.try_push_all(bytes)
    }
}

impl<'a, H> AsRef<NoNullStr> for NoNullString<'a, H>
    where H: Allocator,
{
    fn as_ref(&self) -> &NoNullStr {
        unsafe { NoNullStr::from_bytes_unchecked(&self.data) }
    }
}

impl<'a, H> AsMut<NoNullStr> for NoNullString<'a, H>
    where H: Allocator,
{
    fn as_mut(&mut self) -> &mut NoNullStr {
        unsafe { NoNullStr::from_bytes_unchecked_mut(&mut self.data) }
    }
}

impl<'a, H> AsNoNullStr for NoNullString<'a, H>
    where H: Allocator,
{
    fn as_no_null_str(&self) -> Result<&NoNullStr> {
        unsafe { Ok(NoNullStr::from_bytes_unchecked(&self.data)) }
    }
}

impl<'a, H> AsMutNoNullStr for NoNullString<'a, H>
    where H: Allocator,
{
    fn as_mut_no_null_str(&mut self) -> Result<&mut NoNullStr> {
        unsafe { Ok(NoNullStr::from_bytes_unchecked_mut(&mut self.data)) }
    }
}

impl<'a, H> Debug for NoNullString<'a, H>
    where H: Allocator,
{
    fn fmt<W: Write>(&self, w: &mut W) -> Result {
        self.as_ref().fmt(w)
    }
}

impl<'a, H> AsMutCStr for NoNullString<'a, H>
    where H: Allocator,
{
    fn as_mut_cstr(&mut self) -> Result<&mut CStr> {
        // We push a 0 at the end, create a slice, then truncate without reallocating so
        // that after the reference is dropped the null is gone.

        try!(self.data.reserve(1));
        self.data.push(0);
        let cstr: &'static mut CStr = unsafe {
            mem::cast(CStr::from_bytes_unchecked_mut(&mut self.data[..]))
        };
        unsafe { self.data.set_len(cstr.len()); }
        Ok(cstr)
    }
}
