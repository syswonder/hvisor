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
//  Solicey <lzoi_lth@163.com>

numeric_enum_macro::numeric_enum! {
    #[repr(u32)]
    #[derive(Debug)]
    pub enum CpuIdEax {
        VendorInfo = 0x0,
        FeatureInfo = 0x1,
        StructuredExtendedFeatureInfo = 0x7,
        TimeStampCounterInfo = 0x15,
        ProcessorFrequencyInfo = 0x16,
        HypervisorInfo = 0x4000_0000,
        HypervisorFeatures = 0x4000_0001,
    }
}

bitflags::bitflags! {
    /// CPUID leaf 7 subleaf 0 ECX feature bits visible to guests.
    pub struct ExtendedFeaturesEcx: u32 {
        const SGX_LC = 1_u32 << 30;
        const RDPID = 1_u32 << 22;
        const LA57 = 1_u32 << 16;

        const AVX512VPOPCNTDQ = 1_u32 << 14;
        const TMEEN = 1_u32 << 13;
        const AVX512BITALG = 1_u32 << 12;
        const AVX512VNNI = 1_u32 << 11;
        const VPCLMULQDQ = 1_u32 << 10;
        const VAES = 1_u32 << 9;
        const GFNI = 1_u32 << 8;
        const CETSS = 1_u32 << 7;
        const AVX512VBMI2 = 1_u32 << 6;
        const WAITPKG = 1_u32 << 5;
        const OSPKE = 1_u32 << 4;
        const PKU = 1_u32 << 3;
        const UMIP = 1_u32 << 2;
        const AVX512VBMI = 1_u32 << 1;
        const PREFETCHWT1 = 1_u32 << 0;
    }

    /// CPUID leaf 1 feature mask: ECX occupies low bits, EDX high bits.
    pub struct FeatureInfoFlags: u64 {
        const PBE = 1_u64 << (32 + 31);
        const TM = 1_u64 << (32 + 29);
        const HTT = 1_u64 << (32 + 28);
        const SS = 1_u64 << (32 + 27);
        const SSE2 = 1_u64 << (32 + 26);
        const SSE = 1_u64 << (32 + 25);
        const FXSR = 1_u64 << (32 + 24);
        const MMX = 1_u64 << (32 + 23);
        const ACPI = 1_u64 << (32 + 22);
        const DS = 1_u64 << (32 + 21);
        const CLFSH = 1_u64 << (32 + 19);
        const PSN = 1_u64 << (32 + 18);
        const PSE36 = 1_u64 << (32 + 17);
        const PAT = 1_u64 << (32 + 16);
        const CMOV = 1_u64 << (32 + 15);
        const MCA = 1_u64 << (32 + 14);
        const PGE = 1_u64 << (32 + 13);
        const MTRR = 1_u64 << (32 + 12);
        const SEP = 1_u64 << (32 + 11);
        const APIC = 1_u64 << (32 + 9);
        const CX8 = 1_u64 << (32 + 8);
        const MCE = 1_u64 << (32 + 7);
        const PAE = 1_u64 << (32 + 6);
        const MSR = 1_u64 << (32 + 5);
        const TSC = 1_u64 << (32 + 4);
        const PSE = 1_u64 << (32 + 3);
        const DE = 1_u64 << (32 + 2);
        const VME = 1_u64 << (32 + 1);
        const FPU = 1_u64 << (32 + 0);

        const HYPERVISOR = 1_u64 << 31;
        const RDRAND = 1_u64 << 30;
        const F16C = 1_u64 << 29;
        const AVX = 1_u64 << 28;
        const OSXSAVE = 1_u64 << 27;
        const XSAVE = 1_u64 << 26;
        const AESNI = 1_u64 << 25;
        const TSC_DEADLINE = 1_u64 << 24;
        const POPCNT = 1_u64 << 23;
        const MOVBE = 1_u64 << 22;
        const X2APIC = 1_u64 << 21;
        const SSE42 = 1_u64 << 20;
        const SSE41 = 1_u64 << 19;
        const DCA = 1_u64 << 18;
        const PCID = 1_u64 << 17;
        const PDCM = 1_u64 << 15;
        const CMPXCHG16B = 1_u64 << 13;
        const FMA = 1_u64 << 12;
        const CNXTID = 1_u64 << 10;
        const SSSE3 = 1_u64 << 9;
        const TM2 = 1_u64 << 8;
        const EIST = 1_u64 << 7;
        const SMX = 1_u64 << 6;
        const VMX = 1_u64 << 5;
        const DSCPL = 1_u64 << 4;
        const MONITOR = 1_u64 << 3;
        const DTES64 = 1_u64 << 2;
        const PCLMULQDQ = 1_u64 << 1;
        const SSE3 = 1_u64 << 0;
    }
}
