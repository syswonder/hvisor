// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//
use core::marker::PhantomData;
/// A struct definition that wraps around a bare pointer
/// author: ForeverYolo
/// references:
/// rust_shyper: https://gitee.com/openeuler/rust_shyper
use core::ops::Deref;
use core::ptr::NonNull;

#[derive(Debug)]
pub struct GicRef<'a, T> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a T>,
}

impl<T> GicRef<'_, T> {
    pub const unsafe fn new<'a>(ptr: *const T) -> GicRef<'a, T> {
        GicRef {
            ptr: NonNull::new_unchecked(ptr.cast_mut()),
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for GicRef<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for GicRef<'_, T> {}
unsafe impl<T: Sync> Send for GicRef<'_, T> {}
unsafe impl<T: Sync> Sync for GicRef<'_, T> {}

impl<T> Deref for GicRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}
