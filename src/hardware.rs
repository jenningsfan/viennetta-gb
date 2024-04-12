use self::{cpu::CPU, io::{MMU, cart::Cartridge}};

pub mod io;
pub mod cpu;
mod boot_rom;

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
            total_cycles += self.run_instruction() as u16;
        }
        //println!("FRAME FRAME FRAMETY FRAME Y: {} X: {} CYCLES: {}", self.mmu.ppu.line_y, self.mmu.ppu.line_x, self.mmu.ppu.cycles_line);
        // println!("{}", self.mmu.apu.sample_buf.len());
        // self.mmu.apu.sample_buf = vec![];
        self.mmu.get_frame()
    }

    #[inline]
    pub fn run_instruction(&mut self) -> u8 {
        let cycles = self.cpu.tick(&mut self.mmu);
        self.mmu.run_cycles(cycles);

        cycles
    }

    pub fn get_save_data(&self) -> Option<&Vec<u8>> {
        self.mmu.cart.get_save_data()
    }
}