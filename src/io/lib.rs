// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![crate_name = "linux_io"]
#![crate_type = "lib"]
#![feature(plugin, no_std)]
#![plugin(linux_core_plugin)]
#![no_std]

extern crate linux_core as core;
extern crate linux_ty_one as ty_one;

#[prelude_import] use ty_one::prelude::*;
use core::{mem};
use ty_one::error::{Errno, DeviceFull};

pub type Result<T> = ty_one::result::Result<T, Errno>;

pub trait Read {
    fn scatter_read(&mut self, buf: &mut [&mut [u8]]) -> Result<usize>;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.scatter_read(&mut [buf])
    }

    fn read_all(&mut self, mut buf: &mut [u8]) -> Result<usize> {
        let mut read = 0;
        while buf.len() > 0 {
            match self.read(buf) {
                e @ Err(_) => return e,
                Ok(0) => break,
                Ok(n) => {
                    read += n;
                    buf = &mut {buf}[n..];
                }
            }
        }
        Ok(read)
    }
}

pub trait Write {
    fn gather_write(&mut self, buf: &[&[u8]]) -> Result<usize>;

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.gather_write(&[buf])
    }

    fn write_all(&mut self, mut buf: &[u8]) -> Result<usize> {
        let mut written = 0;
        while buf.len() > 0 {
            match self.write(buf) {
                e @ Err(_) => return e,
                Ok(0) => return Err(DeviceFull),
                Ok(n) => {
                    written += n;
                    buf = &buf[n..];
                }
            }
        }
        Ok(written)
    }

    fn write_str(&mut self, buf: &str) -> Result<usize> {
        self.write(buf.as_bytes())
    }
}

impl<'a> Read for &'a [u8] {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = mem::copy(buf, *self);
        *self = &self[n..];
        Ok(n)
    }

    fn scatter_read(&mut self, mut buf: &mut [&mut [u8]]) -> Result<usize> {
        let mut sum = 0;
        while self.len() > 0 && buf.len() > 0 {
            sum += self.read(&mut buf[0]).unwrap();
            let b = buf;
            buf = &mut b[1..];
        }
        Ok(sum)
    }
}

impl<'a> Write for &'a mut [u8] {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = mem::copy(*self, buf);
        unsafe {
            // Compiler bug.
            let slf: &mut &'static mut [u8] = mem::cast::<&mut &mut [u8], _>(self);
            *slf = &mut slf[n..];
        }
        Ok(n)
    }

    fn gather_write(&mut self, mut buf: &[&[u8]]) -> Result<usize> {
        let mut sum = 0;
        while self.len() > 0 && buf.len() > 0 {
            sum += self.write(&buf[0]).unwrap();
            buf = &buf[1..];
        }
        Ok(sum)
    }
}

impl<'a, T: Read+?Sized> Read for &'a mut T {
    fn scatter_read(&mut self, buf: &mut [&mut [u8]]) -> Result<usize> {
        (**self).scatter_read(buf)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (**self).read(buf)
    }

    fn read_all(&mut self, mut buf: &mut [u8]) -> Result<usize> {
        (**self).read_all(buf)
    }
}

impl<'a, T: Write+?Sized> Write for &'a mut T {
    fn gather_write(&mut self, buf: &[&[u8]]) -> Result<usize> {
        (**self).gather_write(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (**self).write(buf)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<usize> {
        (**self).write_all(buf)
    }

    fn write_str(&mut self, buf: &str) -> Result<usize> {
        (**self).write_str(buf)
    }
}
