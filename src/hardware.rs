use self::{cpu::CPU, io::{MMU, Cartridge}};

pub mod io;
pub mod cpu;
mod boot_rom;

//const CYCLES_PER_FRAME: u16 = 1;
const CYCLES_PER_FRAME: u16 = 17556;

#[derive(Debug)]
pub struct GameBoy {
    pub cpu: cpu::CPU,
    pub mmu: io::MMU,
}

impl GameBoy {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            cpu: CPU::default(),
            mmu: MMU::new(cart),
        }
    }

    pub fn run_frame(&mut self) -> io::LcdPixels {
        let mut total_cycles = 0;

        while total_cycles < CYCLES_PER_FRAME {
            let cycles = self.cpu.execute_opcode(&mut self.mmu);
            self.mmu.run_cycles(cycles * 4);
            total_cycles += cycles as u16;
        }

        self.mmu.get_frame()
    }
}