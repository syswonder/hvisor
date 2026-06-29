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

bitflags::bitflags! {
    pub struct FeatureInfoFlags: u64 {
        // CPUID.01H ECX bits.
        const SSE3 = 1 << 0; // bit 0: SSE3
        const PCLMULQDQ = 1 << 1; // bit 1: PCLMULQDQ
        const DTES64 = 1 << 2; // bit 2: 64-bit DS area
        const MONITOR = 1 << 3; // bit 3: MONITOR/MWAIT
        const DSCPL = 1 << 4; // bit 4: CPL debug store
        const VMX = 1 << 5; // bit 5: VMX
        const SMX = 1 << 6; // bit 6: SMX
        const EIST = 1 << 7; // bit 7: SpeedStep
        const TM2 = 1 << 8; // bit 8: thermal monitor 2
        const SSSE3 = 1 << 9; // bit 9: SSSE3
        const CNXTID = 1 << 10; // bit 10: L1 context ID
        const FMA = 1 << 12; // bit 12: FMA
        const CMPXCHG16B = 1 << 13; // bit 13: CMPXCHG16B
        const PDCM = 1 << 15; // bit 15: perf/debug capability
        const PCID = 1 << 17; // bit 17: PCID
        const DCA = 1 << 18; // bit 18: DCA
        const SSE41 = 1 << 19; // bit 19: SSE4.1
        const SSE42 = 1 << 20; // bit 20: SSE4.2
        const X2APIC = 1 << 21; // bit 21: x2APIC
        const MOVBE = 1 << 22; // bit 22: MOVBE
        const POPCNT = 1 << 23; // bit 23: POPCNT
        const TSC_DEADLINE = 1 << 24; // bit 24: TSC deadline
        const AESNI = 1 << 25; // bit 25: AESNI
        const XSAVE = 1 << 26; // bit 26: XSAVE
        const OSXSAVE = 1 << 27; // bit 27: OSXSAVE
        const AVX = 1 << 28; // bit 28: AVX
        const F16C = 1 << 29; // bit 29: F16C
        const RDRAND = 1 << 30; // bit 30: RDRAND
        const HYPERVISOR = 1 << 31; // bit 31: hypervisor

        // CPUID.01H EDX bits.
        const FPU = 1 << 32; // bit 32: x87 FPU
        const VME = 1 << (32 + 1); // bit 33: VME
        const DE = 1 << (32 + 2); // bit 34: debug extensions
        const PSE = 1 << (32 + 3); // bit 35: page size extension
        const TSC = 1 << (32 + 4); // bit 36: TSC
        const MSR = 1 << (32 + 5); // bit 37: MSR
        const PAE = 1 << (32 + 6); // bit 38: PAE
        const MCE = 1 << (32 + 7); // bit 39: machine check
        const CX8 = 1 << (32 + 8); // bit 40: CMPXCHG8B
        const APIC = 1 << (32 + 9); // bit 41: APIC
        const SEP = 1 << (32 + 11); // bit 43: SYSENTER/SYSEXIT
        const MTRR = 1 << (32 + 12); // bit 44: MTRR
        const PGE = 1 << (32 + 13); // bit 45: page global
        const MCA = 1 << (32 + 14); // bit 46: machine check arch
        const CMOV = 1 << (32 + 15); // bit 47: CMOV
        const PAT = 1 << (32 + 16); // bit 48: PAT
        const PSE36 = 1 << (32 + 17); // bit 49: PSE-36
        const PSN = 1 << (32 + 18); // bit 50: serial number
        const CLFSH = 1 << (32 + 19); // bit 51: CLFLUSH
        const DS = 1 << (32 + 21); // bit 53: debug store
        const ACPI = 1 << (32 + 22); // bit 54: thermal MSRs
        const MMX = 1 << (32 + 23); // bit 55: MMX
        const FXSR = 1 << (32 + 24); // bit 56: FXSAVE/FXRSTOR
        const SSE = 1 << (32 + 25); // bit 57: SSE
        const SSE2 = 1 << (32 + 26); // bit 58: SSE2
        const SS = 1 << (32 + 27); // bit 59: self snoop
        const HTT = 1 << (32 + 28); // bit 60: hyper-threading
        const TM = 1 << (32 + 29); // bit 61: thermal monitor
        const PBE = 1 << (32 + 31); // bit 63: pending break
    }

    pub struct ExtendedFeaturesEcx: u32 {
        const PREFETCHWT1 = 1 << 0; // bit 0: PREFETCHWT1
        const AVX512VBMI = 1 << 1; // bit 1: AVX512_VBMI
        const UMIP = 1 << 2; // bit 2: UMIP
        const PKU = 1 << 3; // bit 3: PKU
        const OSPKE = 1 << 4; // bit 4: OSPKE
        const WAITPKG = 1 << 5; // bit 5: WAITPKG
        const AVX512VBMI2 = 1 << 6; // bit 6: AVX512_VBMI2
        const CETSS = 1 << 7; // bit 7: CET shadow stack
        const GFNI = 1 << 8; // bit 8: GFNI
        const VAES = 1 << 9; // bit 9: VAES
        const VPCLMULQDQ = 1 << 10; // bit 10: VPCLMULQDQ
        const AVX512VNNI = 1 << 11; // bit 11: AVX512_VNNI
        const AVX512BITALG = 1 << 12; // bit 12: AVX512_BITALG
        const TMEEN = 1 << 13; // bit 13: TME enable
        const AVX512VPOPCNTDQ = 1 << 14; // bit 14: AVX512_VPOPCNTDQ
        const LA57 = 1 << 16; // bit 16: 57-bit linear address
        const RDPID = 1 << 22; // bit 22: RDPID
        const SGX_LC = 1 << 30; // bit 30: SGX launch config
    }
}

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

pub(crate) fn emulate_tsc_cpuid_leaf(
    tsc_freq_mhz: Option<u32>,
    fallback: raw_cpuid::CpuIdResult,
) -> raw_cpuid::CpuIdResult {
    match tsc_freq_mhz.and_then(|mhz| mhz.checked_mul(1_000_000)) {
        Some(crystal_hz) => raw_cpuid::CpuIdResult {
            eax: 1,
            ebx: 1,
            ecx: crystal_hz,
            edx: 0,
        },
        None => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn cpuid_leaf_0x15_uses_measured_tsc_frequency() {
        let fallback = raw_cpuid::CpuIdResult {
            eax: 7,
            ebx: 8,
            ecx: 9,
            edx: 10,
        };
        let result = emulate_tsc_cpuid_leaf(Some(2_400), fallback);

        assert_eq!(result.eax, 1);
        assert_eq!(result.ebx, 1);
        assert_eq!(result.ecx, 2_400_000_000);
        assert_eq!(result.edx, 0);
    }

    #[test_case]
    fn cpuid_leaf_0x15_falls_back_without_safe_frequency() {
        let fallback = raw_cpuid::CpuIdResult {
            eax: 7,
            ebx: 8,
            ecx: 9,
            edx: 10,
        };
        let result = emulate_tsc_cpuid_leaf(Some(u32::MAX), fallback);

        assert_eq!(result.eax, 7);
        assert_eq!(result.ebx, 8);
        assert_eq!(result.ecx, 9);
        assert_eq!(result.edx, 10);
    }
}
