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
    /// CPUID leaf 1 feature bits.
    ///
    /// ECX is stored in bits 0..31 and EDX is stored in bits 32..63 so the
    /// trap path can adjust either register through one flag type.
    pub struct FeatureInfoFlags: u64 {
        // ECX feature word

        // SIMD / AVX
        /// SSE3 instruction support.
        const SSE3 = 1 << 0;
        /// Supplemental SSE3 instruction support.
        const SSSE3 = 1 << 9;
        /// Fused multiply-add instructions using YMM state.
        const FMA = 1 << 12;
        /// SSE4.1 instruction support.
        const SSE41 = 1 << 19;
        /// SSE4.2 instruction support.
        const SSE42 = 1 << 20;
        /// XSAVE/XRSTOR extended-state support, including XSETBV/XGETBV and XCR0.
        const XSAVE = 1 << 26;
        /// OS support for XSETBV/XGETBV and XSAVE-based extended-state management.
        const OSXSAVE = 1 << 27;
        /// AVX instruction support.
        const AVX = 1 << 28;
        /// Half-precision floating-point conversion instructions.
        const F16C = 1 << 29;

        // Security / crypto
        /// Carry-less multiplication instruction support.
        const PCLMULQDQ = 1 << 1;
        /// Safer Mode Extensions support.
        const SMX = 1 << 6;
        /// AES instruction-set extensions.
        const AESNI = 1 << 25;
        /// Hardware random number generation through RDRAND.
        const RDRAND = 1 << 30;

        // Virtualization
        /// Virtual Machine Extensions support.
        const VMX = 1 << 5;
        /// x2APIC support.
        const X2APIC = 1 << 21;
        /// The CPUID result is being mediated by a hypervisor.
        const HYPERVISOR = 1 << 31;

        // Power / thermal
        /// MONITOR and MWAIT instruction support.
        const MONITOR = 1 << 3;
        /// Enhanced SpeedStep support.
        const EIST = 1 << 7;
        /// Thermal Monitor 2 support.
        const TM2 = 1 << 8;

        // Misc
        /// 64-bit Debug Store area layout.
        const DTES64 = 1 << 2;
        /// CPL-qualified Debug Store support.
        const DSCPL = 1 << 4;
        /// Configurable L1 data-cache context mode.
        const CNXTID = 1 << 10;
        /// CMPXCHG16B instruction support.
        const CMPXCHG16B = 1 << 13;
        /// IA32_PERF_CAPABILITIES performance/debug MSR support.
        const PDCM = 1 << 15;
        /// Process-context identifiers usable through CR4.PCIDE.
        const PCID = 1 << 17;
        /// Direct cache access for memory-mapped device prefetching.
        const DCA = 1 << 18;
        /// MOVBE instruction support.
        const MOVBE = 1 << 22;
        /// POPCNT instruction support.
        const POPCNT = 1 << 23;
        /// APIC timer deadline mode driven by a TSC value.
        const TSC_DEADLINE = 1 << 24;

        // EDX feature word

        // SIMD / AVX
        /// On-chip x87 floating-point unit.
        const FPU = 1 << (32 + 0);
        /// MMX instruction support.
        const MMX = 1 << (32 + 23);
        /// FXSAVE/FXRSTOR support for fast floating-point context save and restore.
        const FXSR = 1 << (32 + 24);
        /// SSE instruction support.
        const SSE = 1 << (32 + 25);
        /// SSE2 instruction support.
        const SSE2 = 1 << (32 + 26);

        // Security / crypto
        /// Debug extensions, including I/O breakpoints controlled through CR4.DE.
        const DE = 1 << (32 + 2);
        /// Machine-check exception support.
        const MCE = 1 << (32 + 7);
        /// Machine-check architecture MSRs and reporting banks.
        const MCA = 1 << (32 + 14);
        /// Processor serial-number facility is present and enabled.
        const PSN = 1 << (32 + 18);

        // Virtualization
        /// Virtual-8086 mode enhancements such as CR4.VME and CR4.PVI.
        const VME = 1 << (32 + 1);
        /// On-chip local APIC.
        const APIC = 1 << (32 + 9);
        /// Hyper-threading topology field in CPUID.1.EBX is valid.
        const HTT = 1 << (32 + 28);

        // Power / thermal
        /// Thermal-monitor and software-controlled clock facilities.
        const ACPI = 1 << (32 + 22);
        /// Automatic thermal-control circuitry.
        const TM = 1 << (32 + 29);
        /// Pending-break-enable signaling while the processor is stopped.
        const PBE = 1 << (32 + 31);

        // Misc
        /// 4 MiB page-size extension.
        const PSE = 1 << (32 + 3);
        /// RDTSC instruction and CR4.TSD control support.
        const TSC = 1 << (32 + 4);
        /// RDMSR and WRMSR instruction support.
        const MSR = 1 << (32 + 5);
        /// Physical-address extension paging support.
        const PAE = 1 << (32 + 6);
        /// CMPXCHG8B instruction support.
        const CX8 = 1 << (32 + 8);
        /// SYSENTER/SYSEXIT and their associated MSRs.
        const SEP = 1 << (32 + 11);
        /// Memory type range registers.
        const MTRR = 1 << (32 + 12);
        /// Page-global entries controlled by CR4.PGE.
        const PGE = 1 << (32 + 13);
        /// Conditional-move instruction support.
        const CMOV = 1 << (32 + 15);
        /// Page attribute table support.
        const PAT = 1 << (32 + 16);
        /// 36-bit page-size extension for 4 MiB pages above 4 GiB.
        const PSE36 = 1 << (32 + 17);
        /// CLFLUSH instruction support.
        const CLFSH = 1 << (32 + 19);
        /// Debug Store buffer support.
        const DS = 1 << (32 + 21);
        /// Self-snoop cache-coherency support.
        const SS = 1 << (32 + 27);
    }

    /// CPUID leaf 7, subleaf 0 ECX feature bits exposed to the guest.
    pub struct ExtendedFeaturesEcx: u32 {
        // SIMD / AVX
        /// AVX-512 vector byte manipulation instructions.
        const AVX512VBMI = 1 << 1;
        /// Second AVX-512 vector byte manipulation instruction set.
        const AVX512VBMI2 = 1 << 6;
        /// AVX-512 vector neural-network instructions.
        const AVX512VNNI = 1 << 11;
        /// AVX-512 bit-algorithm instructions.
        const AVX512BITALG = 1 << 12;
        /// AVX-512 vector population-count instructions for D/Q elements.
        const AVX512VPOPCNTDQ = 1 << 14;

        // Security / crypto
        /// User-mode instruction prevention.
        const UMIP = 1 << 2;
        /// Protection keys for user-mode pages.
        const PKU = 1 << 3;
        /// Protection-key instructions are enabled by CR4.PKE.
        const OSPKE = 1 << 4;
        /// CET shadow-stack capability and related CET MSRs.
        const CETSS = 1 << 7;
        /// Galois-field instruction support.
        const GFNI = 1 << 8;
        /// Vector AES instruction support.
        const VAES = 1 << 9;
        /// Vector carry-less multiplication instruction support.
        const VPCLMULQDQ = 1 << 10;
        /// Total-memory-encryption control MSRs are available.
        const TMEEN = 1 << 13;
        /// SGX launch-configuration support.
        const SGX_LC = 1 << 30;

        // Virtualization
        // No leaf-7 ECX flags used here are categorized as virtualization controls.

        // Power / thermal
        /// WAITPKG instruction support.
        const WAITPKG = 1 << 5;

        // Misc
        /// PrefetchWT1 instruction support.
        const PREFETCHWT1 = 1 << 0;
        /// Five-level paging and 57-bit linear addresses.
        const LA57 = 1 << 16;
        /// RDPID and IA32_TSC_AUX support.
        const RDPID = 1 << 22;
    }
}
