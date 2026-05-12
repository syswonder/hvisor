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
// Authors: Jingyu Liu <liujingyu24s@ict.ac.cn>
//

use alloc::vec::Vec;

use crate::consts::PAGE_SIZE;
use crate::device::iommu::iommu_msi_pt_tlb_invalid;
use crate::memory::Frame;
use crate::memory::GuestPhysAddr;
use crate::memory::MemFlags;
use crate::memory::MemoryRegion;
use crate::platform::__board::{IMSIC_GUEST_INDEX, IMSIC_GUEST_NUM, IMSIC_S_BASE};
use crate::zone::Zone;
use core::sync::atomic::{fence, Ordering};

/// RISC-V IOMMU MSI PTE size (two 64-bit doublewords).
pub const MSI_PTE_BYTES: usize = 16;
/// One 4 KiB page holds at most this many MSI PTEs (spec: table ≤256 entries uses one 4K page).
pub const MSI_PTE_COUNT: usize = PAGE_SIZE / MSI_PTE_BYTES;

/// Encode one MSI PTE in basic translation mode (V=1, C=0, M=3): replace GPA bits ≥12 with `PPN`.
///
/// DW0: bit0 V=1, bits 2:1 M=3, bits 53:10 PPN, bit 63 C=0. DW1 reserved (0).
#[inline]
pub fn msi_pte_encode_basic(hpa: usize) -> [u8; MSI_PTE_BYTES] {
    let ppn = (hpa >> 12) as u64;
    const PPN_MASK: u64 = (1u64 << 44) - 1;
    // V=1, M=3, C=0, PPN=ppn
    let dw0: u64 = 1 | (3 << 1) | ((ppn & PPN_MASK) << 10);
    let mut out = [0u8; MSI_PTE_BYTES];
    out[..8].copy_from_slice(&dw0.to_le_bytes());
    out
}

/// Fill a 4 KiB MSI page table: PTE index is `vcpu_id = cpu_id - first_cpu` (guest ordinal).
fn msi_pt_fill(
    frame: &mut Frame,
    zone_id: usize,
    cpu_ids: &[usize],
    first_cpu: usize,
    imsic_base: usize,
    guest_num: usize,
) {
    frame.clear();
    for &cpu_id in cpu_ids {
        let vcpu_id = cpu_id - first_cpu;
        assert!(
            vcpu_id < MSI_PTE_COUNT,
            "MSI PTE index (vcpu_id) {} exceeds table capacity {}",
            vcpu_id,
            MSI_PTE_COUNT
        );
        // IMSIC VS-file for global view.
        let imsic_hpa = imsic_base + PAGE_SIZE * ((1 + guest_num) * cpu_id + IMSIC_GUEST_INDEX);
        // VIMSIC S-file for guest view.
        let imsic_gpa = imsic_base + PAGE_SIZE * vcpu_id;
        // Construct MSI PTE.
        let pte = msi_pte_encode_basic(imsic_hpa);
        let off = vcpu_id * MSI_PTE_BYTES;
        info!(
            "vIMSIC map vcpu {} imsic hpa {:#x} gpa {:#x}",
            vcpu_id, imsic_hpa, imsic_gpa
        );
        // Fill MSI PTE to MSI PT.
        frame.as_slice_mut()[off..off + MSI_PTE_BYTES].copy_from_slice(&pte);
        // After change the MSI PTE, invalidate the related TLBs in IOMMU.
        iommu_msi_pt_tlb_invalid(zone_id as u16, imsic_gpa as usize);
    }
}

/**
 * For imsic's guest_num = 1
 * its interrupt files:
 *     S-file CPU0      base + 0x0000
 *    VS-file CPU0      base + 0x1000
 *     S-file CPU1      base + 0x2000
 *    VS-file CPU1      base + 0x3000
 *     ...
 */

/// Maps guest IMSIC interrupt files and builds a one-page IOMMU MSI translation table for this zone.
///
/// Returns the physical frame backing the MSI PTE array (`start_paddr()` for IOMMU programming).
pub fn vimsic_init(zone: &mut Zone, imsic_base: usize, guest_num: usize) -> Frame {
    let size = crate::memory::PAGE_SIZE;

    let cpu_ids: Vec<_> = zone.cpu_set().iter().collect();
    let first_cpu = zone
        .cpu_set()
        .first_cpu()
        .expect("vimsic_init: zone has no CPUs");
    let mut inner = zone.write();

    // 1. Insert regions to zone.gpm, this is used by cpu core (for example, send ipi through vimsic).
    cpu_ids.iter().for_each(|cpu_id| {
        let vcpu_id = cpu_id - first_cpu;
        let imsic_hpa = imsic_base + PAGE_SIZE * ((1 + guest_num) * cpu_id + IMSIC_GUEST_INDEX);
        // For VM, it couldn't see VS-files. Guest uses contiguous indices from 0.
        let imsic_gpa = imsic_base + PAGE_SIZE * vcpu_id;
        info!(
            "Zone {} vIMSIC map phys_hart {} vcpu {} imsic hpa {:#x} gpa {:#x}",
            zone.id(),
            cpu_id,
            vcpu_id,
            imsic_hpa,
            imsic_gpa
        );
        inner.gpm_mut().insert(MemoryRegion::new_with_offset_mapper(
            imsic_gpa as GuestPhysAddr,
            imsic_hpa,
            size,
            MemFlags::READ | MemFlags::WRITE,
        ));
    });
    drop(inner);

    // 2. Construct MSI PT, this is used by devices that send MSI.
    let mut msi_frame = Frame::new_zero().expect("MSI page table frame allocation");
    msi_pt_fill(
        &mut msi_frame,
        zone.id(),
        &cpu_ids,
        first_cpu,
        imsic_base,
        guest_num,
    );
    info!(
        "Zone {} MSI page table at HPA {:#x} ({} valid PTEs, basic translation)",
        zone.id(),
        msi_frame.start_paddr(),
        cpu_ids.len()
    );
    msi_frame
}

pub fn imsic_vs_file_addr(hart_id: usize) -> usize {
    IMSIC_S_BASE + PAGE_SIZE * ((1 + IMSIC_GUEST_NUM) * hart_id + IMSIC_GUEST_INDEX)
}
