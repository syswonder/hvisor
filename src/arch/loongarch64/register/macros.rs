/*
    this file is forked from extern crate loongArch64::register::macros;
    wheatfox
*/
#![allow(unused)]
macro_rules! impl_tlbelo {
    ($ident:ident,$number:expr) => {
        impl $ident {
            fn valid(&self) -> bool {
                self.bits.get_bit(0)
            }
            fn dirty(&self) -> bool {
                self.bits.get_bit(1)
            }
            fn plv(&self) -> usize {
                self.bits.get_bits(2..=3)
            }
            fn mat(&self) -> MemoryAccessType {
                self.bits.get_bits(4..=5).into()
            }
            fn global(&self) -> bool {
                self.bits.get_bit(6)
            }
            fn ppn(&self) -> usize {
                self.bits.get_bits(12..PALEN)
            }
            fn not_readable(&self) -> bool {
                self.bits.get_bit(61)
            }
            fn not_executable(&self) -> bool {
                self.bits.get_bit(62)
            }
            fn rplv(&self) -> bool {
                self.bits.get_bit(63)
            }
        }
        pub fn set_valid(valid: bool) {
            set_csr_loong_bit!($number, 0, valid);
        }
        pub fn set_dirty(dirty: bool) {
            set_csr_loong_bit!($number, 1, dirty);
        }
        pub fn set_plv(plv: usize) {
            set_csr_loong_bits!($number, 2..=3, plv);
        }
        pub fn set_mat(mem_access_type: MemoryAccessType) {
            set_csr_loong_bits!($number, 4..=5, mem_access_type as usize);
        }
        pub fn set_global(global_flag: bool) {
            set_csr_loong_bit!($number, 6, global_flag);
        }
        pub fn set_ppn(ppn: usize) {
            set_csr_loong_bits!($number, 14..PALEN, ppn);
        }
        pub fn set_not_readable(not_readable: bool) {
            set_csr_loong_bit!($number, 61, not_readable);
        }
        pub fn set_not_executable(not_executable: bool) {
            set_csr_loong_bit!($number, 62, not_executable);
        }
        pub fn set_rplv(rplv: bool) {
            set_csr_loong_bit!($number, 63, rplv);
        }
    };
}
macro_rules! impl_dwm {
    ($ident:ident,$number:expr) => {
        impl $ident {
            fn plv0(&self) -> bool {
                self.bits.get_bit(0)
            }
            fn plv1(&self) -> bool {
                self.bits.get_bit(1)
            }
            fn plv2(&self) -> bool {
                self.bits.get_bit(2)
            }
            fn plv3(&self) -> bool {
                self.bits.get_bit(3)
            }
            fn mat(&self) -> MemoryAccessType {
                self.bits.get_bits(4..=5).into()
            }
            fn vseg(&self) -> usize {
                self.bits.get_bits(60..=63)
            }
        }
        pub fn set_plv0(plv0: bool) {
            set_csr_loong_bit!($number, 0, plv0);
        }
        pub fn set_plv1(plv1: bool) {
            set_csr_loong_bit!($number, 1, plv1);
        }
        pub fn set_plv2(plv2: bool) {
            set_csr_loong_bit!($number, 2, plv2);
        }
        pub fn set_plv3(plv3: bool) {
            set_csr_loong_bit!($number, 3, plv3);
        }
        pub fn set_mat(mat: MemoryAccessType) {
            set_csr_loong_bits!($number, 4..=5, mat as usize);
        }
        pub fn set_vseg(vseg: usize) {
            set_csr_loong_bits!($number, 60..=63, vseg);
        }
        impl Debug for $ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("DMW0")
                    .field("MAT", &self.mat())
                    .field("vseg", &self.vseg())
                    .field("plv0", &self.plv0())
                    .field("plv1", &self.plv1())
                    .field("plv2", &self.plv2())
                    .field("plv3", &self.plv3())
                    .finish()
            }
        }
    };
}

macro_rules! impl_read_csr {
    ($csr_number:literal,$csr_ident:ident) => {
            #[inline(always)]
            pub fn read() -> $csr_ident {
                $csr_ident {
                    bits: unsafe {
                        let bits:usize;
                        core::arch::asm!("csrrd {},{}", out(reg) bits, const $csr_number);
                        bits
                },
            }
        }
    };
}
macro_rules! impl_define_csr {
    ($csr_ident:ident,$doc:expr) => {
        #[doc = $doc]
        #[derive(Copy, Clone)]
        pub struct $csr_ident {
            bits: usize,
        }
    };
}

// csr
macro_rules! read_csr_loong {
    ($csr_number:literal) => {
        unsafe{
            let bits:usize;
            core::arch::asm!("csrrd {},{}", out(reg) bits, const $csr_number);
            bits
        }
    };
}
macro_rules! write_csr_loong {
    ($csr_number:literal,$value:expr) => {
      unsafe {core::arch::asm!("csrwr {},{}", in(reg) $value, const $csr_number);}
    };
}

macro_rules! set_csr_loong_bits {
    ($csr_number:literal,$range:expr,$value:expr) => {
        let mut tmp = read_csr_loong!($csr_number);
        tmp.set_bits($range, $value);
        write_csr_loong!($csr_number, tmp);
    };
}

macro_rules! set_csr_loong_bit {
    ($csr_number:literal,$range:expr,$value:expr) => {
        let mut tmp = read_csr_loong!($csr_number);
        tmp.set_bit($range, $value);
        write_csr_loong!($csr_number, tmp);
    };
}

// gcsr

macro_rules! impl_read_gcsr {
    ($csr_number:literal,$csr_ident:ident) => {
            #[inline(always)]
            pub fn read() -> $csr_ident {
                $csr_ident {
                    bits: unsafe {
                        let bits:usize;
                        core::arch::asm!("gcsrrd {},{}", out(reg) bits, const $csr_number);
                        bits
                },
            }
        }
    };
}

macro_rules! impl_define_gcsr {
    ($csr_ident:ident,$doc:expr) => {
        #[doc = $doc]
        #[derive(Copy, Clone)]
        pub struct $csr_ident {
            bits: usize,
        }
    };
}

#[macro_export]
macro_rules! read_gcsr_loong {
    ($csr_number:literal) => {
        unsafe{
            let bits:usize;
            core::arch::asm!("gcsrrd {},{}", out(reg) bits, const $csr_number);
            bits
        }
    };
}

#[macro_export]
macro_rules! write_gcsr_loong {
    ($csr_number:literal,$value:expr) => {
      unsafe {core::arch::asm!("gcsrwr {},{}", in(reg) $value, const $csr_number);}
    };
}

#[macro_export]
macro_rules! set_gcsr_loong_bits {
    ($csr_number:literal,$range:expr,$value:expr) => {
        let mut tmp = read_gcsr_loong!($csr_number);
        tmp.set_bits($range, $value);
        write_gcsr_loong!($csr_number, tmp);
    };
}

#[macro_export]
macro_rules! set_gcsr_loong_bit {
    ($csr_number:literal,$range:expr,$value:expr) => {
        let mut tmp = read_gcsr_loong!($csr_number);
        tmp.set_bit($range, $value);
        write_gcsr_loong!($csr_number, tmp);
    };
}
