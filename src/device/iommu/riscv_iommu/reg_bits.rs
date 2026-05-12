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
//      Jingyu Liu <liujingyu24s@ict.ac.cn>

use tock_registers::register_bitfields;

register_bitfields! {
    u64,
    pub(super) IOMMU_CAPS [    // RISC-V IOMMU Spec Chap6.3 IOMMU capabilities
        VERSION OFFSET(0) NUMBITS(8) [
            VERSION_1_0 = 0x10,
        ],
        SV32 OFFSET(8) NUMBITS(1) [],
        SV39 OFFSET(9) NUMBITS(1) [],
        SV48 OFFSET(10) NUMBITS(1) [],
        SV57 OFFSET(11) NUMBITS(1) [],
        SVRSW60T59B OFFSET(14) NUMBITS(1) [],
        SVPBMT OFFSET(15) NUMBITS(1) [],
        SV32X4 OFFSET(16) NUMBITS(1) [],
        SV39X4 OFFSET(17) NUMBITS(1) [],
        SV48X4 OFFSET(18) NUMBITS(1) [],
        SV57X4 OFFSET(19) NUMBITS(1) [],
        AMO_MRIF OFFSET(21) NUMBITS(1) [],
        MSI_FLAT OFFSET(22) NUMBITS(1) [],
        MSI_MRIF OFFSET(23) NUMBITS(1) [],
        AMO_HWAD OFFSET(24) NUMBITS(1) [],
        ATS OFFSET(25) NUMBITS(1) [],
        T2GPA OFFSET(26) NUMBITS(1) [],
        END OFFSET(27) NUMBITS(1) [],
        IGS OFFSET(28) NUMBITS(2) [
            MSI = 0,
            WSI = 1,
            BOTH = 2,
        ],
        HPM OFFSET(30) NUMBITS(1) [],
        DBG OFFSET(31) NUMBITS(1) [],
        PAS OFFSET(32) NUMBITS(6) [],
        PD8 OFFSET(38) NUMBITS(1) [],
        PD17 OFFSET(39) NUMBITS(1) [],
        PD20 OFFSET(40) NUMBITS(1) [],
        QOSID OFFSET(41) NUMBITS(1) [],
        NL OFFSET(42) NUMBITS(1) [],
        S OFFSET(43) NUMBITS(1) [],
    ],
    pub(super) IOMMU_DDTP [ // RISC-V IOMMU Spec Chap6.5 Device-directory table pointer
        MODE OFFSET(0) NUMBITS(4) [
            OFF = 0,
            BARE = 1,
            DDT_1LVL = 2,
            DDT_2LVL = 3,
            DDT_3LVL = 4
        ],
        BUSY OFFSET(4) NUMBITS(1) [],
        PPN OFFSET(10) NUMBITS(44) []
    ],
    pub(super) DDT_TC [ // RISC-V IOMMU Spec Chap3.1.3.1 Translation Control
        V OFFSET(0) NUMBITS(1) [],
        EN_ATS OFFSET(1) NUMBITS(1) [],
        EN_PRI OFFSET(2) NUMBITS(1) [],
        T2GPA OFFSET(3) NUMBITS(1) [],
        DT2GPA OFFSET(4) NUMBITS(1) [],
        PDTV OFFSET(5) NUMBITS(1) [],
        PRP OFFSET(6) NUMBITS(1) [],
        GADEV OFFSET(7) NUMBITS(1) [],
        SADEV OFFSET(8) NUMBITS(1) [],
        DPE OFFSET(9) NUMBITS(1) [],
        SBE OFFSET(10) NUMBITS(1) [],
        SXL OFFSET(11) NUMBITS(1) []
    ],
    pub(super) DDT_IOHGATP [ // RISC-V IOMMU Spec Chap3.1.3.2 IO hypervisor guest address translation and protection
        PPN OFFSET(0) NUMBITS(44) [],
        GSCID OFFSET(44) NUMBITS(16) [],
        MODE OFFSET(60) NUMBITS(4) [
            SV39X4 = 8,
            SV48X4 = 9,
            SV57X4 = 10
        ]
    ],
    pub(super) DDT_TA [ // RISC-V IOMMU Spec Chap3.1.3.3 Translation attributes
        PS_CID OFFSET(12) NUMBITS(20) [],
        RCID OFFSET(40) NUMBITS(12) [],
        MTYPE OFFSET(52) NUMBITS(12) [],
    ],
    pub(super) DDT_FSC [ // RISC-V IOMMU Spec Chap3.1.3.4 First-stage context
        MODE OFFSET(60) NUMBITS(4) [
            BARE = 0,
            SV39 = 8,
            SV48 = 9,
            SV57 = 10
        ],
        PPN OFFSET(0) NUMBITS(44) []
    ],
    pub(super) DDT_DIR [ // RISC-V IOMMU Spec Chap3.1.1 Non-leaf DDT entry
        V OFFSET(0) NUMBITS(1) [],
        PPN OFFSET(10) NUMBITS(44) []
    ],
    pub(super) IOMMU_XQB [ // RISC-V IOMMU Spec Chap6.6 Command-queue base
                           // RISC-V IOMMU Spec Chap6.9 Fault queue base
        LOG2SZ_1 OFFSET(0) NUMBITS(5) [],
        PPN OFFSET(10) NUMBITS(44) []
    ],
    pub(super) IOMMU_FQ_TAG [ // RISC-V IOMMU Spec Chap4.2 Fault/Event-Queue
        CAUSE OFFSET(0) NUMBITS(12) [],
        PID OFFSET(12) NUMBITS(20) [],
        PV OFFSET(32) NUMBITS(1) [],
        PRIV OFFSET(33) NUMBITS(1) [],
        TYPE OFFSET(34) NUMBITS(6) [],
        DID OFFSET(40) NUMBITS(24) []
    ]
}

register_bitfields! {
    u32,
    pub(super) IOMMU_FCTL [ // RISC-V IOMMU Spec Chap6.4 Features-control register
        BE OFFSET(0) NUMBITS(1) [],
        WSI OFFSET(1) NUMBITS(1) [],
        GXL OFFSET(2) NUMBITS(1) [],
    ],
    pub(super) IOMMU_CQCSR [ // RISC-V IOMMU Spec Chap6.15 Command-queue CSR
        CQEN OFFSET(0) NUMBITS(1) [],
        CIE OFFSET(1) NUMBITS(1) [],
        CQMF OFFSET(8) NUMBITS(1) [],
        CMDTO OFFSET(9) NUMBITS(1) [],
        CMDILL OFFSET(10) NUMBITS(1) [],
        FENCEWIP OFFSET(11) NUMBITS(1) [],
        CQON OFFSET(16) NUMBITS(1) [],
        BUSY OFFSET(17) NUMBITS(1) [],
    ],
    pub(super) IOMMU_FQCSR [ // RISC-V IOMMU Spec Chap6.16 Fault-queue CSR
        FQEN OFFSET(0) NUMBITS(1) [],
        FIE OFFSET(1) NUMBITS(1) [],
        FQMF OFFSET(8) NUMBITS(1) [],
        FQOF OFFSET(9) NUMBITS(1) [],
        FQON OFFSET(16) NUMBITS(1) [],
        BUSY OFFSET(17) NUMBITS(1) [],
    ],
    pub(super) IOMMU_IPSR [ // RISC-V IOMMU Spec Chap6.18 Interrupt pending status register
        CIP OFFSET(0) NUMBITS(1) [],
        FIP OFFSET(1) NUMBITS(1) [],
        PMIP OFFSET(2) NUMBITS(1) [],
        PIP OFFSET(3) NUMBITS(1) []
    ]
}
