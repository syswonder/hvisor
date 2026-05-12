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
#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[cfg(target_arch = "loongarch64")]
pub mod loongarch64;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

// export modules for external use
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
pub use loongarch64::*;

#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

// Unified architecture interface
pub mod interface {
    /// Get the secondary entry point for CPU startup
    /// For architectures that need a special secondary entry (like LoongArch64),
    /// this returns the secondary entry function. For other architectures,
    /// this returns the same as arch_entry.
    pub fn arch_secondary_entry() -> unsafe extern "C" fn() -> i32 {
        #[cfg(target_arch = "loongarch64")]
        {
            use super::loongarch64::entry::arch_secondary_entry;
            arch_secondary_entry
        }

        #[cfg(not(target_arch = "loongarch64"))]
        {
            // For non-LoongArch64 architectures, we need to create a wrapper
            // that matches the expected signature
            unsafe extern "C" fn wrapper() -> i32 {
                // Call the actual arch_entry, which may return ! or i32
                // For architectures where arch_entry returns !, this code is unreachable
                // For architectures where arch_entry returns i32, we return its value
                #[cfg(target_arch = "aarch64")]
                {
                    use super::aarch64::entry::arch_entry;
                    arch_entry();
                    // arch_entry never returns, so this is unreachable
                    unreachable!()
                }

                #[cfg(any(target_arch = "riscv64", target_arch = "x86_64"))]
                {
                    use super::entry::arch_entry;
                    arch_entry()
                }
            }
            wrapper
        }
    }

    /// Architecture-specific initialization after heap is set up
    pub fn arch_post_heap_init(host_dtb: usize) {
        #[cfg(target_arch = "loongarch64")]
        {
            use super::loongarch64::mm::arch_post_heap_init as la_arch_post_heap_init;
            la_arch_post_heap_init(host_dtb);
        }

        #[cfg(not(target_arch = "loongarch64"))]
        {
            use super::mm::arch_post_heap_init as generic_arch_post_heap_init;
            generic_arch_post_heap_init(host_dtb);
        }
    }

    /// Architecture-specific timer initialization
    pub fn timer_init() {
        #[cfg(target_arch = "loongarch64")]
        {
            use super::loongarch64::timer::timer_init as la_timer_init;
            la_timer_init();
        }

        #[cfg(not(target_arch = "loongarch64"))]
        {
            // Other architectures may have their own timer init or none at all
        }
    }

    /// Architecture-specific print logo function
    pub fn print_logo() {
        #[cfg(target_arch = "loongarch64")]
        {
            use super::loongarch64::print_logo as la_print_logo;
            la_print_logo();
        }

        #[cfg(not(target_arch = "loongarch64"))]
        {
            // Other architectures may not have a special logo
            println!("Hello, HVISOR!");
        }
    }

    /// Architecture-specific clear BSS function
    pub fn clear_bss() {
        #[cfg(target_arch = "loongarch64")]
        {
            use crate::clear_bss;
            clear_bss();
        }

        #[cfg(not(target_arch = "loongarch64"))]
        {
            // BSS is cleared in arch_entry for other architectures
        }
    }
}
