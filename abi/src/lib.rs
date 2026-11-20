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
//! Stable ABI shared by hvisor, its kernel modules, and user-space tools.
//!
//! This crate intentionally has no dependencies outside core and alloc. All
//! structures are repr(C) and use fixed-width integer types where possible so
//! they can also be exported to C with cbindgen.

#![no_std]

extern crate alloc;

pub mod config;
pub mod hypercall;
pub mod ioctl;
pub mod ivc;
pub mod scmi;
pub mod virtio;

pub use config::*;
