use crate::{arch::device::PortIoDevice, device::irqchip::pic::hpet, error::HvResult};
use alloc::vec::Vec;
use bit_field::BitField;
use core::ops::Range;
use spin::Mutex;

const VIRT_PIT_FREQ_HZ: u64 = 1193182;

#[allow(non_snake_case)]
pub mod ReadWriteState {
    pub const LSB: u8 = 1;
    pub const MSB: u8 = 2;
    pub const WORD_0: u8 = 3;
    pub const WORD_1: u8 = 4;
}

#[derive(Debug, Default, Clone)]
struct VirtI8254Channel {
    count: i32,
    mode: u8,
    rw_mode: u8,
    read_state: u8,
    write_state: u8,
    write_latch: u32,
    count_set_time: u64,
}

impl VirtI8254Channel {
    fn get_count(&self) -> i32 {
        let delta =
            (hpet::current_time_nanos() - self.count_set_time) * VIRT_PIT_FREQ_HZ / 1_000_000_000;
        let mut count = self.count;
        match self.mode {
            0 => count = (self.count - (delta as i32)) & 0xffff,
            _ => {}
        }
        count
    }

    fn set_count(&mut self, mut value: u32) {
        if value == 0 {
            value = 0x1_0000;
        }
        self.count_set_time = hpet::current_time_nanos();
        self.count = value as _;
    }
}

pub struct VirtI8254 {
    base_port: u16,
    speaker_port: u16,
    port_range: Vec<Range<u16>>,
    channels: Vec<Mutex<VirtI8254Channel>>,
}

impl VirtI8254 {
    pub fn new(base_port: u16, speaker_port: u16) -> Self {
        Self {
            base_port,
            speaker_port,
            port_range: vec![base_port..base_port + 4, speaker_port..speaker_port + 1],
            channels: vec![
                Mutex::new(VirtI8254Channel::default()),
                Mutex::new(VirtI8254Channel::default()),
                Mutex::new(VirtI8254Channel::default()),
            ],
        }
    }
}

impl PortIoDevice for VirtI8254 {
    fn port_range(&self) -> &Vec<Range<u16>> {
        &self.port_range
    }

    fn read(&self, port: u16, msg: u8) -> HvResult<u32> {
        // info!("i8254 read: {:x}", port);

        /*if port == self.speaker_port {
            if let Some(channel) = self.channels.get(2) {
                let mut channel = channel.lock();
                let cnt = channel.get_count();
                return Ok(0);
            }
        }*/

        let chan_id = ((port - self.base_port) & 3) as usize;
        if let Some(channel) = self.channels.get(chan_id) {
            let mut channel = channel.lock();

            let ret = match channel.read_state {
                ReadWriteState::LSB => 0,
                ReadWriteState::MSB => 0,
                ReadWriteState::WORD_0 => {
                    channel.read_state = ReadWriteState::WORD_1;
                    channel.get_count() & 0xff
                }
                ReadWriteState::WORD_1 => {
                    channel.read_state = ReadWriteState::WORD_0;
                    (channel.get_count() >> 8) & 0xff
                }
                _ => 0,
            };
            return Ok(ret as u32);
        }

        Ok(0)
    }

    fn write(&self, port: u16, value: u32, msg: u8) -> HvResult {
        // info!("i8254 write: {:x}, {:x}", port, value);

        let offset: usize = (port - self.base_port) as _;
        match offset {
            3 => {
                let chan_id: usize = value.get_bits(6..=7) as _;
                if chan_id == 3 {
                } else if let Some(channel) = self.channels.get(chan_id) {
                    let mut channel = channel.lock();
                    let access: u8 = value.get_bits(4..=5) as _;

                    if access != 0 {
                        channel.rw_mode = access;
                        channel.read_state = access;
                        channel.write_state = access;

                        channel.mode = value.get_bits(1..=3) as _;
                    }
                }
            }
            0 | 1 | 2 => {
                if let Some(channel) = self.channels.get(offset) {
                    let mut channel = channel.lock();
                    match channel.write_state {
                        ReadWriteState::LSB => {}
                        ReadWriteState::MSB => {}
                        ReadWriteState::WORD_0 => {
                            channel.write_latch = value;
                            channel.write_state = ReadWriteState::WORD_1;
                        }
                        ReadWriteState::WORD_1 => {
                            let low = channel.write_latch;
                            channel.set_count(low | (value << 8));
                            channel.write_state = ReadWriteState::WORD_0;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        };

        Ok(())
    }
}
