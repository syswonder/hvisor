use crate::{
    arch::device::{
        all_virt_devices, DeviceMsg, PortIoDevice, PIC_MASTER_BASE_PORT, PIC_SLAVE_BASE_PORT,
    },
    device::irqchip::inject_vector,
    error::HvResult,
    percpu::{get_cpu_data, this_cpu_data},
};
use alloc::vec::Vec;
use core::ops::Range;
use spin::{Mutex, MutexGuard};

pub const SEND_IRQ_KEY: u8 = 0x82;
pub const SEND_VECTOR_KEY: u8 = 0x59;

struct VirtI8259aUnlocked {
    base_port: u16,
    isr: u8,
    irr: u8,
    imr: u8,
    init_state: u8,
    vector_base: u8,
    auto_eoi: bool,
    icw4_needed: bool,
    is_master: bool,
    poll_cmd: bool,
    read_isr: bool,
    special_fully_nested_mode: bool,
    special_mask: bool,
}

impl VirtI8259aUnlocked {
    fn new(base_port: u16, is_master: bool) -> Self {
        Self {
            base_port,
            isr: 0,
            irr: 0,
            imr: 0,
            init_state: 0,
            vector_base: 0,
            auto_eoi: false,
            icw4_needed: false,
            is_master,
            poll_cmd: false,
            read_isr: false,
            special_fully_nested_mode: false,
            special_mask: false,
        }
    }

    fn ack_irq(&mut self, irq: i32) {
        // TODO: auto eoi
        self.isr |= (1 << irq);
        // TODO: elcr
        self.irr &= !(1 << irq);
        // VirtDualI8259aUnlocked::update_irq(self, pic2);
    }

    fn get_priority(&mut self, mask: u8) -> u8 {
        if mask == 0 {
            return 8;
        }

        let mut priority: u8 = 0;
        // TODO: priority add
        while mask & (1 << (priority & 7)) == 0 {
            priority += 1;
        }

        priority
    }

    fn get_irq(&mut self) -> i32 {
        let mut mask = self.irr & !self.imr;
        let priority = self.get_priority(mask);
        if priority == 8 {
            return -1;
        }

        mask = self.isr;
        if self.special_mask {
            mask &= !self.imr;
        }
        if self.special_fully_nested_mode && self.is_master {
            mask &= !(1u8 << 2); // ignore in service slave irq
        }
        let cur_priority = self.get_priority(mask);
        if priority < cur_priority {
            return (priority as i32) & 7;
        }

        -1
    }

    fn set_irq(&mut self, irq: i32) {
        let mask: u8 = 1 << irq;
        self.irr |= mask;
    }
}

pub struct VirtDualI8259aUnlocked {
    pics: Vec<VirtI8259aUnlocked>,
}

impl VirtDualI8259aUnlocked {
    fn new(master_base_port: u16, slave_base_port: u16) -> Self {
        Self {
            pics: vec![
                VirtI8259aUnlocked::new(master_base_port, true),
                VirtI8259aUnlocked::new(slave_base_port, false),
            ],
        }
    }

    fn ack_irq(&mut self, irq: i32, id: usize) {
        self.pics[id].ack_irq(irq);
        self.update_irq(id);
    }

    fn set_irq(&mut self, irq: i32, id: usize) {
        self.pics[id].set_irq(irq);
        // isr bit empty, can serve
        if self.pics[id].isr & (1 << irq) == 0 {
            self.update_irq(id);
        }
    }

    fn update_irq(&mut self, id: usize) {
        let mut pic = &mut self.pics[id];
        let irq = pic.get_irq();
        if irq < 0 {
            return;
        }

        if pic.is_master {
            let mut vector = pic.vector_base + (irq as u8);
            if irq == 2 {
                let mut irq2 = self.pics[1].get_irq();
                if irq2 >= 0 {
                    self.ack_irq(irq2, 1);
                } else {
                    irq2 = 7; // spurious irq
                }
                vector = self.pics[1].vector_base + (irq2 as u8);
            }
            self.ack_irq(irq, 0);
            // TODO: single core? smp?
            inject_vector(0, vector, None, true);
        } else {
            self.set_irq(2, 0);
        }
    }

    fn read(&mut self, id: usize, port: u16) -> HvResult<u32> {
        let mut pic = &mut self.pics[id];
        if pic.poll_cmd {
            pic.poll_cmd = false;
            let mut irq = pic.get_irq();
            if irq >= 0 {
                self.ack_irq(irq, id);
                irq |= 0x80;
            } else {
                irq = 0;
            }
            return Ok(irq as u32);
        }

        let mut ret = 0;
        let offset = port - pic.base_port;
        if offset == 0 {
            if pic.read_isr {
                ret = pic.isr;
            } else {
                ret = pic.irr;
            }
        } else {
            ret = pic.imr;
        }

        Ok(ret as u32)
    }

    fn write(&mut self, id: usize, port: u16, value: u32) -> HvResult {
        let mut pic = &mut self.pics[id];
        let offset = port - pic.base_port;
        let value: u8 = value as u8;
        if offset == 0 {
            if value & 0x10 != 0 {
                // ICW1
                if value & 0x08 != 0 {
                    error!("I8259A: level-triggered not supported!");
                }
                if value & 0x02 != 0 {
                    error!("I8259A: single pic not supported!");
                }
                pic.init_state = 1;
                pic.icw4_needed = (value & 0x01) != 0;
            } else if value & 0x08 != 0 {
                // OCW3
                if value & 0x02 != 0 {
                    pic.read_isr = value & 1 != 0;
                }
                if value & 0x04 != 0 {
                    pic.poll_cmd = true;
                }
                if value & 0x40 != 0 {
                    pic.special_mask = (value >> 5) & 1 != 0;
                }
            } else {
                // OCW2
                let cmd = value >> 5;
                match cmd {
                    0 | 4 => {
                        // TODO: rotate auto eoi
                    }
                    1 | 5 => {
                        // non specific eoi
                        let isr = pic.isr;
                        let priority = pic.get_priority(isr);
                        if priority != 8 {
                            let irq = priority & 7;
                            pic.isr &= !(1 << irq);
                            self.update_irq(id);
                        }
                    }
                    3 | 7 => {
                        // specific eoi
                        let irq = value & 7;
                        pic.isr &= !(1 << irq);
                        self.update_irq(id);
                    }
                    _ => {}
                }
            }
        } else {
            match pic.init_state {
                0 => {
                    pic.imr = value;
                    self.update_irq(id);
                }
                1 => {
                    // ICW2
                    pic.vector_base = value & 0xf8;
                    // info!("I8259A: vector base: {:x}", pic.vector_base);
                    pic.init_state = 2;
                }
                2 => {
                    // ICW3
                    // master: 0x4, slave: 0x2
                    pic.init_state = match pic.icw4_needed {
                        true => 3,
                        false => 0,
                    }
                }
                3 => {
                    // ICW4
                    pic.special_fully_nested_mode = (value >> 4) & 1 != 0;
                    pic.auto_eoi = (value >> 1) & 1 != 0;
                    pic.init_state = 0;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub struct VirtDualI8259a {
    port_range: Vec<Range<u16>>,
    dual_pic: Mutex<VirtDualI8259aUnlocked>,
}

impl VirtDualI8259a {
    pub fn new(master_base_port: u16, slave_base_port: u16) -> Self {
        Self {
            port_range: vec![
                master_base_port..master_base_port + 2,
                slave_base_port..slave_base_port + 2,
            ],
            dual_pic: Mutex::new(VirtDualI8259aUnlocked::new(
                master_base_port,
                slave_base_port,
            )),
        }
    }
}

impl PortIoDevice for VirtDualI8259a {
    fn port_range(&self) -> &Vec<Range<u16>> {
        &self.port_range
    }

    fn read(&self, port: u16, msg: u8) -> HvResult<u32> {
        // info!("I8259A read, port: {:x}", port);
        let mut dual_pic = self.dual_pic.lock();

        if self.port_range[0].contains(&port) {
            dual_pic.read(0, port)
        } else {
            dual_pic.read(1, port)
        }
    }

    fn write(&self, port: u16, value: u32, msg: u8) -> HvResult {
        // info!("I8259A write, port: {:x} value: {:x}", port, value);
        let mut dual_pic = self.dual_pic.lock();
        let is_master = self.port_range[0].contains(&port);
        let id = match is_master {
            true => 0,
            false => 1,
        };

        if msg != 0 {
            match msg {
                DeviceMsg::UPDATE_IRQ_HIGH => {
                    dual_pic.set_irq(value as i32, id);
                }
                _ => {}
            }
            return Ok(());
        }

        dual_pic.write(id, port, value)
    }
}
