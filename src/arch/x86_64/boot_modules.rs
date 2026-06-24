// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FITNESS FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//

pub const MAX_MODULES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootModule {
    /// Address grub loaded the module at.
    pub src: usize,
    /// Size of the module in bytes.
    pub size: usize,
    /// Final address of the module. Equal to `src` when the module's command
    /// line is `0`, i.e. it is left where grub placed it.
    pub dst: usize,
    /// Whether the module still has to be copied to `dst`.
    pub pending: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelocationError {
    DestinationOverlap,
    CyclicDependency,
}

/// Whether two half-open byte ranges intersect.
pub fn ranges_overlap(base_a: usize, len_a: usize, base_b: usize, len_b: usize) -> bool {
    base_a < base_b + len_b && base_b < base_a + len_a
}

/// Copy pending modules in an order that does not overwrite another pending
/// module's source range.
pub fn relocate_modules<F>(
    modules: &mut [BootModule],
    mut copy_module: F,
) -> Result<usize, RelocationError>
where
    F: FnMut(BootModule),
{
    for i in 0..modules.len() {
        for j in (i + 1)..modules.len() {
            if ranges_overlap(
                modules[i].dst,
                modules[i].size,
                modules[j].dst,
                modules[j].size,
            ) {
                return Err(RelocationError::DestinationOverlap);
            }
        }
    }

    let mut remaining = modules.iter().filter(|module| module.pending).count();
    let moved = remaining;
    while remaining > 0 {
        let mut progressed = false;
        for i in 0..modules.len() {
            if !modules[i].pending {
                continue;
            }
            let blocked = modules.iter().enumerate().any(|(j, module)| {
                j != i
                    && module.pending
                    && ranges_overlap(modules[i].dst, modules[i].size, module.src, module.size)
            });
            if blocked {
                continue;
            }

            let module = modules[i];
            copy_module(module);
            modules[i].pending = false;
            remaining -= 1;
            progressed = true;
        }
        if !progressed {
            return Err(RelocationError::CyclicDependency);
        }
    }

    Ok(moved)
}
