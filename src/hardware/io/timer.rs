use super::Interrupts;

#[derive(Debug, Default)]
pub struct Timer {
    div: u16,
    counter: u8,
    modulo: u8,
    control: u8,
    intern_counter: u16,
}

impl Timer {
    pub fn run_cycles(&mut self, cycles: u8) -> Interrupts {
        let mut int = false;

        for _ in 0..cycles {
            self.div = self.div.wrapping_add(1);

            if self.control & 0x4 == 0x4 {
                let tima_cycles = match self.control & 0x2 {
                    0 => 256,
                    1 => 4,
                    2 => 16,
                    3 => 64,
                    _ => panic!("impossible")
                };

                while self.intern_counter >= tima_cycles {
                    self.intern_counter -= tima_cycles;
                    self.counter = self.counter.wrapping_add(1);

                    if self.counter == 0 {  // i.e. overflown
                        self.counter = self.modulo;
                        int = true;
                    }
                }
            }
        }

        if int {
            Interrupts::Timer
        }
        else {
            Interrupts::empty()
        }
    }

    pub fn read_io(&self, reg: u16) -> u8 {
        match reg {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            0xFF07 => self.control,
            _ => panic!("{reg} is not a valid timer register")
        }
    }

    pub fn write_io(&mut self, reg: u16, value: u8) {
        match reg {
            0xFF04 => self.div = 0,
            0xFF05 => self.counter = value,
            0xFF06 => self.modulo = value,
            0xFF07 => self.control = value,
            _ => panic!("{reg} is not a valid timer register")
        }
    }
}