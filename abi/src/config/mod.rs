// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! Zone configuration ABI.
//!
//! The common leaf structures and constants live in `common`. Each
//! architecture gets a concrete zone-configuration type generated from the
//! same `define_zone_config!` macro, with its own architecture-specific
//! `HvArchZoneConfig`.

pub mod aarch64;
pub mod common;
pub mod loongarch64;
pub mod pci;
pub mod riscv64;
pub mod x86_64;
pub mod zone;
pub mod zone_list;

pub use common::*;
pub use pci::*;
pub use zone_list::*;
