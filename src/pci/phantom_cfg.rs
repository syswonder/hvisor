#[derive(Debug)]
pub struct PhantomCfg {
    pub bdf: usize,
    command: u16
}

impl PhantomCfg {
    pub fn new(bdf: usize, command: u16) -> Self{
        Self {
            bdf,
            command: command & !0x400, // set disable-intx to 0, the origin state
        }
    }

    pub fn set_cmd(&mut self, command: u16){
        self.command = command;
    }

    pub fn get_cmd(&self) -> u16 {
        self.command
    }
}

