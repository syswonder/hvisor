// File:          gcfg.rs
// Description:   this is the register definition of GCFG
// Authors:       wheatfox(enkerewpo@hotmail.com)
// Created:       2023-12-25
use bit_field::BitField;

impl_define_csr!(
  Gcfg,
  "Guest Config register(GCFG)
  this register is used to control guest machine
  "
);
impl_read_csr!(0x51, Gcfg);

impl Gcfg {
  /// RO, MATC's available value
  /// bit i set to 1 means MATC can be set to i(one-hot)
  pub fn matp(&self) -> usize {
    self.bits.get_bits(0..=3)
  }
  /// RW, VM's MAT
  /// 0 -> from GVA->GPA's MAT
  /// 1 -> from GPA->HPA's MAT
  /// 2 -> from GVA->GPA and GVA->HPA's weakest MAT
  /// weakness: WUC weak than CC, CC weak than SUC
  pub fn matc(&self) -> usize {
    self.bits.get_bits(4..=5)
  }
  /// RO, trap on priviledge instruction available or not
  /// 1 -> available, then TOPI can be written
  /// 0 -> not available, then TOPI is read-only zero
  pub fn topip(&self) -> bool {
    self.bits.get_bit(6)
  }
  /// RW, trap on priviledge instruction
  /// 1 -> guest run priviledge instruction will trap a
  /// GSPR(Guest Sensitive Priviledge Resource) exception and trap into host
  pub fn topi(&self) -> bool {
    self.bits.get_bit(7)
  }
  /// RO, trap on timer interrupt available or not
  pub fn totip(&self) -> bool {
    self.bits.get_bit(8)
  }
  /// RW, trap on timer interrupt
  /// 1 -> guest run rdtime, or access TID, TCFG, TVAL, CNTC, TICLR
  /// will trap a GSPR exception and trap into host
  pub fn toti(&self) -> bool {
    self.bits.get_bit(9)
  }
  /// RO, trap on exception available or not
  pub fn toep(&self) -> bool {
    self.bits.get_bit(10)
  }
  /// RW, trap on exception
  /// 1 -> guest run ertn, or triggered a exception that guest os kernel should handle
  /// will trap a GCHC(Guest CSR Hardware Change) exception and trap into host
  pub fn toe(&self) -> bool {
    self.bits.get_bit(11)
  }
  /// RO, trap on PLV available or not
  pub fn topp(&self) -> bool {
    self.bits.get_bit(12)
  }
  /// RW, trap on PLV
  /// when in Guest-PLV0, software change GCSR.CRMD.PLV will trap a GCSC(Guest CSR Software Change) exception and trap into host
  pub fn top(&self) -> bool {
    self.bits.get_bit(13)
  }
  /// RO, trap on host unimplmented CSR available or not
  pub fn tohup(&self) -> bool {
    self.bits.get_bit(14)
  }
  /// RW, trap on host unimplmented CSR
  /// 1 -> triggered the GSCR exception and into host
  pub fn tohu(&self) -> bool {
    self.bits.get_bit(15)
  }
  /// RO, TOCI's available value
  /// if i set to 1, then TOCI can be set to i
  pub fn tocip(&self) -> usize {
    self.bits.get_bits(16..=19)
  }
  /// RW, trap on cache op instruction, config which CACOP will trigger the GSPR
  /// 0 -> all CACOP inst
  /// 1 -> all except Hit inst
  /// 2 -> all except Hit and Index Invalidate Writeback inst
  pub fn toci(&self) -> usize {
    self.bits.get_bits(20..=21)
  }
  /// RO, 1 -> whether support guest access the performance monitor
  pub fn gpmp(&self) -> bool {
    self.bits.get_bit(23)
  }
  /// RW, allocate how many PM starting from PM0 to guest
  pub fn gpm_num(&self) -> usize {
    self.bits.get_bits(24..=26)
  }
}

pub fn set_matc(matc: usize) {
  set_csr_loong_bits!(0x51, 4..=5, matc);
}

pub fn set_topi(topi: bool) {
  set_csr_loong_bit!(0x51, 7, topi);
}

pub fn set_toti(toti: bool) {
  set_csr_loong_bit!(0x51, 9, toti);
}

pub fn set_toe(toe: bool) {
  set_csr_loong_bit!(0x51, 11, toe);
}

pub fn set_top(top: bool) {
  set_csr_loong_bit!(0x51, 13, top);
}

pub fn set_tohu(tohu: bool) {
  set_csr_loong_bit!(0x51, 15, tohu);
}

pub fn set_toci(toci: usize) {
  set_csr_loong_bits!(0x51, 20..=21, toci);
}

pub fn set_gpm_num(gpm_num: usize) {
  set_csr_loong_bits!(0x51, 24..=26, gpm_num);
}
