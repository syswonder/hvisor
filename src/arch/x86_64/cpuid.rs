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

        HypervisorInfo = 0x4000_0000,
        HypervisorFeatures = 0x4000_0001,

        StructuredExtendedFeatureInfo = 0x7,
        TscInfo = 0x15,
        ProcessorFrequencyInfo = 0x16,
    }
}

bitflags::bitflags! {
    pub struct FeatureInfoFlags: u64 {
        // CPUID.01H:ECX virtualization, platform and context-management bits.
        const VMX = 1 << 5;
        const SMX = 1 << 6;
        const HYPERVISOR = 1 << 31;
        const MONITOR = 1 << 3;
        const EIST = 1 << 7;
        const TM2 = 1 << 8;
        const DTES64 = 1 << 2;
        const DSCPL = 1 << 4;
        const CNXTID = 1 << 10;
        const PDCM = 1 << 15;
        const PCID = 1 << 17;
        const X2APIC = 1 << 21;
        const TSC_DEADLINE = 1 << 24;

        // CPUID.01H:ECX SIMD, arithmetic, crypto and random-number features.
        const SSE3 = 1 << 0;
        const PCLMULQDQ = 1 << 1;
        const SSSE3 = 1 << 9;
        const FMA = 1 << 12;
        const CMPXCHG16B = 1 << 13;
        const DCA = 1 << 18;
        const SSE41 = 1 << 19;
        const SSE42 = 1 << 20;
        const MOVBE = 1 << 22;
        const POPCNT = 1 << 23;
        const AESNI = 1 << 25;
        const XSAVE = 1 << 26;
        const OSXSAVE = 1 << 27;
        const AVX = 1 << 28;
        const F16C = 1 << 29;
        const RDRAND = 1 << 30;

        // CPUID.01H:EDX system, memory-management and APIC-related features.
        const APIC = 1 << (32 + 9);
        const SEP = 1 << (32 + 11);
        const MTRR = 1 << (32 + 12);
        const PGE = 1 << (32 + 13);
        const MCA = 1 << (32 + 14);
        const PAT = 1 << (32 + 16);
        const PAE = 1 << (32 + 6);
        const PSE = 1 << (32 + 3);
        const PSE36 = 1 << (32 + 17);
        const VME = 1 << (32 + 1);
        const DE = 1 << (32 + 2);
        const MCE = 1 << (32 + 7);
        const TSC = 1 << (32 + 4);
        const MSR = 1 << (32 + 5);
        const FPU = 1 << (32 + 0);
        const CX8 = 1 << (32 + 8);
        const CMOV = 1 << (32 + 15);
        const PSN = 1 << (32 + 18);
        const CLFSH = 1 << (32 + 19);

        // CPUID.01H:EDX cache, trace, thermal and legacy SIMD features.
        const DS = 1 << (32 + 21);
        const ACPI = 1 << (32 + 22);
        const MMX = 1 << (32 + 23);
        const FXSR = 1 << (32 + 24);
        const SSE = 1 << (32 + 25);
        const SSE2 = 1 << (32 + 26);
        const SS = 1 << (32 + 27);
        const HTT = 1 << (32 + 28);
        const TM = 1 << (32 + 29);
        const PBE = 1 << (32 + 31);
    }

    pub struct ExtendedFeaturesEcx: u32 {
        // CPUID.07H:ECX byte 0 extension bits.
        const CETSS = 1 << 7;
        const AVX512VBMI2 = 1 << 6;
        const WAITPKG = 1 << 5;
        const OSPKE = 1 << 4;
        const PKU = 1 << 3;
        const UMIP = 1 << 2;
        const AVX512VBMI = 1 << 1;
        const PREFETCHWT1 = 1 << 0;

        // CPUID.07H:ECX byte 1 vector and memory-encryption extension bits.
        const GFNI = 1 << 8;
        const VAES = 1 << 9;
        const VPCLMULQDQ = 1 << 10;
        const AVX512VNNI = 1 << 11;
        const AVX512BITALG = 1 << 12;
        const TMEEN = 1 << 13;
        const AVX512VPOPCNTDQ = 1 << 14;

        // CPUID.07H:ECX high extension bits referenced by the emulator.
        const SGX_LC = 1 << 30;
        const RDPID = 1 << 22;
        const LA57 = 1 << 16;
    }
}
