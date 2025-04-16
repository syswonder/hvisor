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
//
// This used for transfering the software interrupt to the target hart

//  Offset	    Width	Attr	Name	    Description
//  0x0000_0000 4B      RW      SETSSIP0    HART index 0 set supervisor-level IPI register
//  0x0000_0004 4B      RW      SETSSIP1    HART index 1 set supervisor-level IPI register
//  ...
//  0x0000_3FFC 4B              RESERVED    Reserved for future use.

use crate::consts::MAX_CPU_NUM;
use log::info;
use riscv_pac::result::{Error, Result};
use riscv_pac::HartIdNumber;
use riscv_peripheral::aclint::sswi::SSWI;
use spin::Once;

// Only init at boot time.
// Don't spend more time considering the concurrency.
static ACLINT_BASE: Once<usize> = Once::new();

// HartId is needed for riscv_peripheral::aclint.
#[derive(Debug, Clone, Copy)]
pub struct HartId(usize);

// sswi.setssip needs a type which implement HartIdNumber.
unsafe impl HartIdNumber for HartId {
    const MAX_HART_ID_NUMBER: usize = MAX_CPU_NUM as usize;
    #[inline]
    fn number(self) -> usize {
        self.0
    }
    #[inline]
    fn from_number(number: usize) -> Result<Self> {
        if number > Self::MAX_HART_ID_NUMBER {
            return Err(Error::InvalidVariant(number));
        }
        Ok(unsafe { core::mem::transmute::<usize, HartId>(number) })
    }
}

/// Init the aclint's base address.
pub fn aclint_init(base_addr: usize) {
    info!("ACLINT: base address is {:#x?}", base_addr);
    ACLINT_BASE.call_once(|| base_addr);
}

/// Send a software interrupt to the target hart.
pub fn aclint_send_ipi(hart_id: usize) {
    assert!(hart_id < MAX_CPU_NUM, "hart_id is out of range");

    debug!("ACLINT: addr {:#x}", *ACLINT_BASE.get().unwrap());

    let sswi = unsafe { SSWI::new(*ACLINT_BASE.get().unwrap() as usize) };
    let setssip = sswi.setssip(HartId::from_number(hart_id).unwrap());

    // Write the software interrupt to the target hart.
    setssip.pend();
}
