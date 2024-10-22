#[derive(Debug, Default, Clone, Copy)]
pub struct PciBar {
    val: u32,
    bar_type: BarType,
    size: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct BarRegion{
    pub start: usize,
    pub size: usize,
    pub bar_type: BarType
}

#[derive(Default, Debug, Copy, Clone)]
pub enum BarType {
    Mem32,
    Mem64,
    IO,
    #[default]
    Unknown,
}

impl PciBar{
    // origin_val: the register value written by vm
    // val: write !0u64 to the BAR to get the size this BAR need
    pub fn init(&mut self, origin_val: u32, val: u32){
        self.val = origin_val;

        if let Some(fix_bit) = (0..32).rev().find(|&off| val & (1 << off) == 0){
            if fix_bit != 31 {
                self.size = 1 << (fix_bit + 1);
            }else{
                // fix_bit == 31, indicates that all the bits are read-only
                self.size = 0;
            }
        }else{
            // all the bits are rw, indicates this BAR's value is the upper 32 bits of a region's address
            // so the size depends on the next BAR, set self.size to 1, or the value will overflow
            self.size = 1;
        }

        self.bar_type = match self.val & 0b1{
            0b1 => BarType::IO,
            _ => {
                match self.val & 0b110{
                    0b000 => BarType::Mem32,
                    0b100 => BarType::Mem64,
                    _ => BarType::Unknown,
                }
            }
        };
    }

    pub fn is_mutable(&self) -> bool{
        match self.size{
            0 => false,
            _ => true,
        }
    }

    pub fn mem_type_64(&self) -> bool{
        match self.bar_type{
            BarType::Mem64 => true,
            _ => false
        }
    }

    pub fn get_32b_region(&self) -> BarRegion{
        BarRegion{
            start: (self.val & 0xfffffff0) as _,
            size: self.size,
            bar_type: self.bar_type,
        }
    }

    pub fn get_upper_mem64_32b(&self) -> BarRegion{
        BarRegion{
            start: self.val as _, // upper 32bits are all mutable
            size: self.size,
            bar_type: self.bar_type,
        }
    }

    pub fn get_64b_region(&self, lower_region: BarRegion) -> BarRegion{
        let higher_region = self.get_upper_mem64_32b();
        // info!("mm64, high: {:#x}, low: {:#x}", higher_region.start, lower_region.start);
        BarRegion {
            start: (higher_region.start << 32) + lower_region.start,
            size: higher_region.size * lower_region.size,
            bar_type: BarType::Mem64
        }
    }
}