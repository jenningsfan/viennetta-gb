use super::Interrupts;

#[derive(Debug, Default)]
pub struct Timer {
    div: u16,
    tima: u8,
    modulo: u8,
    control: u8,
}

impl Timer {
    pub fn run_cycles(&mut self, cycles: u8) -> Interrupts {
        let mut int = false;

        for _ in 0..cycles {
            let tima_bit_pos = match self.control & 0x3 {
                0 => 9,
                1 => 3,
                2 => 5,
                3 => 7,
                _ => panic!("impossible")
            };
            let mut prev_tima_bit = (self.div >> tima_bit_pos) & 1;
            prev_tima_bit &= (self.control as u16 >> 2) & 1; // get enable bit

            self.div = self.div.wrapping_add(1);

            let mut post_tima_bit = (self.div >> tima_bit_pos) & 1;
            post_tima_bit &= (self.control as u16 >> 2) & 1; // get enable bit

            if prev_tima_bit == 1 && post_tima_bit == 0 { // falling edge
                self.tima = self.tima.wrapping_add(1);
                
                if self.tima == 0 {  // i.e. overflown
                    self.tima = self.modulo;
                    int = true;
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
            0xFF05 => self.tima,
            0xFF06 => self.modulo,
            0xFF07 => self.control,
            _ => panic!("{reg} is not a valid timer register")
        }
    }

    pub fn write_io(&mut self, reg: u16, value: u8) {
        match reg {
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.modulo = value,
            0xFF07 => self.control = value,
            _ => panic!("{reg} is not a valid timer register")
        }
    }
}