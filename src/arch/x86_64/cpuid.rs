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
        FeatureInfo = 0x0000_0001,
        StructuredExtendedFeatureInfo = 0x0000_0007,

        VendorInfo = 0x0000_0000,
        TscInfo = 0x0000_0015,
        ProcessorFrequencyInfo = 0x0000_0016,

        HypervisorInfo = 0x4000_0000,
        HypervisorFeatures = 0x4000_0001,
    }
}

bitflags::bitflags! {
    /// CPUID.1 feature bits, following the raw-cpuid 8.1.2 names.
    pub struct FeatureInfoFlags: u64 {
        // ECX: isolation, crypto, and random sources.

        /// ECX[5]: VMX virtualization controls.
        const VMX = 1 << 5;
        /// ECX[6]: Safer Mode Extensions.
        const SMX = 1 << 6;
        /// ECX[17]: Process-context identifiers.
        const PCID = 1 << 17;
        /// ECX[25]: AES instructions.
        const AESNI = 1 << 25;
        /// ECX[1]: Carry-less multiply.
        const PCLMULQDQ = 1 << 1;
        /// ECX[30]: RDRAND instruction.
        const RDRAND = 1 << 30;
        /// ECX[31]: Hypervisor-present bit.
        const HYPERVISOR = 1 << 31;

        // ECX: execution state and platform plumbing.

        /// ECX[26]: XSAVE/XRSTOR state support.
        const XSAVE = 1 << 26;
        /// ECX[27]: OS enabled XGETBV/XSETBV state handling.
        const OSXSAVE = 1 << 27;
        /// ECX[21]: x2APIC support.
        const X2APIC = 1 << 21;
        /// ECX[24]: APIC timer can use TSC deadlines.
        const TSC_DEADLINE = 1 << 24;
        /// ECX[3]: MONITOR/MWAIT support.
        const MONITOR = 1 << 3;
        /// ECX[10]: L1 context ID support.
        const CNXTID = 1 << 10;
        /// ECX[2]: 64-bit Debug Store area.
        const DTES64 = 1 << 2;
        /// ECX[4]: Debug Store can be CPL-qualified.
        const DSCPL = 1 << 4;
        /// ECX[15]: IA32_PERF_CAPABILITIES is available.
        const PDCM = 1 << 15;
        /// ECX[18]: Direct cache access prefetch.
        const DCA = 1 << 18;

        // ECX: SIMD, scalar instructions, and power controls.

        /// ECX[0]: SSE3 instructions.
        const SSE3 = 1 << 0;
        /// ECX[9]: SSSE3 instructions.
        const SSSE3 = 1 << 9;
        /// ECX[19]: SSE4.1 instructions.
        const SSE41 = 1 << 19;
        /// ECX[20]: SSE4.2 instructions.
        const SSE42 = 1 << 20;
        /// ECX[12]: FMA with YMM state.
        const FMA = 1 << 12;
        /// ECX[28]: AVX instructions.
        const AVX = 1 << 28;
        /// ECX[29]: F16C conversion instructions.
        const F16C = 1 << 29;
        /// ECX[22]: MOVBE instruction.
        const MOVBE = 1 << 22;
        /// ECX[23]: POPCNT instruction.
        const POPCNT = 1 << 23;
        /// ECX[13]: CMPXCHG16B instruction.
        const CMPXCHG16B = 1 << 13;
        /// ECX[7]: Enhanced SpeedStep.
        const EIST = 1 << 7;
        /// ECX[8]: Thermal Monitor 2.
        const TM2 = 1 << 8;

        // EDX: privilege, paging, and memory controls.

        /// EDX[1]: Virtual 8086 mode extensions.
        const VME = 1 << (32 + 1);
        /// EDX[2]: Debugging extensions.
        const DE = 1 << (32 + 2);
        /// EDX[3]: Page Size Extension.
        const PSE = 1 << (32 + 3);
        /// EDX[6]: Physical Address Extension.
        const PAE = 1 << (32 + 6);
        /// EDX[7]: Machine Check Exception.
        const MCE = 1 << (32 + 7);
        /// EDX[12]: Memory Type Range Registers.
        const MTRR = 1 << (32 + 12);
        /// EDX[13]: Page global bit.
        const PGE = 1 << (32 + 13);
        /// EDX[14]: Machine Check Architecture.
        const MCA = 1 << (32 + 14);
        /// EDX[16]: Page Attribute Table.
        const PAT = 1 << (32 + 16);
        /// EDX[17]: 36-bit page-size extension.
        const PSE36 = 1 << (32 + 17);

        // EDX: core ISA, topology, and platform status.

        /// EDX[0]: x87 FPU.
        const FPU = 1 << (32 + 0);
        /// EDX[4]: Time Stamp Counter.
        const TSC = 1 << (32 + 4);
        /// EDX[5]: RDMSR/WRMSR.
        const MSR = 1 << (32 + 5);
        /// EDX[8]: CMPXCHG8B instruction.
        const CX8 = 1 << (32 + 8);
        /// EDX[9]: local APIC.
        const APIC = 1 << (32 + 9);
        /// EDX[11]: SYSENTER/SYSEXIT.
        const SEP = 1 << (32 + 11);
        /// EDX[15]: conditional move instructions.
        const CMOV = 1 << (32 + 15);
        /// EDX[18]: processor serial number.
        const PSN = 1 << (32 + 18);
        /// EDX[19]: CLFLUSH instruction.
        const CLFSH = 1 << (32 + 19);
        /// EDX[21]: Debug Store.
        const DS = 1 << (32 + 21);
        /// EDX[22]: thermal monitor and software clock control.
        const ACPI = 1 << (32 + 22);
        /// EDX[23]: MMX instructions.
        const MMX = 1 << (32 + 23);
        /// EDX[24]: FXSAVE/FXRSTOR instructions.
        const FXSR = 1 << (32 + 24);
        /// EDX[25]: SSE instructions.
        const SSE = 1 << (32 + 25);
        /// EDX[26]: SSE2 instructions.
        const SSE2 = 1 << (32 + 26);
        /// EDX[27]: self-snoop.
        const SS = 1 << (32 + 27);
        /// EDX[28]: hyper-threading APIC ID field is valid.
        const HTT = 1 << (32 + 28);
        /// EDX[29]: Thermal Monitor.
        const TM = 1 << (32 + 29);
        /// EDX[31]: pending-break enable.
        const PBE = 1 << (32 + 31);
    }

    pub struct ExtendedFeaturesEcx: u32 {
        // Leaf 7 ECX: protection and crypto first.

        /// ECX[3]: user-mode protection keys.
        const PKU = 1 << 3;
        /// ECX[4]: CR4.PKE is enabled by the OS.
        const OSPKE = 1 << 4;
        /// ECX[7]: CET shadow stacks.
        const CETSS = 1 << 7;
        /// ECX[13]: Total Memory Encryption MSRs.
        const TMEEN = 1 << 13;
        /// ECX[30]: SGX launch configuration.
        const SGX_LC = 1 << 30;
        /// ECX[2]: user-mode instruction prevention.
        const UMIP = 1 << 2;
        /// ECX[8]: Galois field instructions.
        const GFNI = 1 << 8;
        /// ECX[9]: vector AES instructions.
        const VAES = 1 << 9;
        /// ECX[10]: vector carry-less multiply.
        const VPCLMULQDQ = 1 << 10;

        // Leaf 7 ECX: AVX-512 extensions.

        /// ECX[1]: AVX-512 VBMI.
        const AVX512VBMI = 1 << 1;
        /// ECX[6]: AVX-512 VBMI2.
        const AVX512VBMI2 = 1 << 6;
        /// ECX[11]: AVX-512 VNNI.
        const AVX512VNNI = 1 << 11;
        /// ECX[12]: AVX-512 bit algorithms.
        const AVX512BITALG = 1 << 12;
        /// ECX[14]: AVX-512 vector popcount.
        const AVX512VPOPCNTDQ = 1 << 14;

        // Leaf 7 ECX: address, wait, and ID helpers.

        /// ECX[0]: PrefetchWT1.
        const PREFETCHWT1 = 1 << 0;
        /// ECX[5]: WAITPKG instructions.
        const WAITPKG = 1 << 5;
        /// ECX[16]: 57-bit linear addresses.
        const LA57 = 1 << 16;
        /// ECX[22]: RDPID and IA32_TSC_AUX.
        const RDPID = 1 << 22;
    }
}
