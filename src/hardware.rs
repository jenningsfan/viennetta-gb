use std::io::Write;

use self::{cpu::CPU, io::{IO, Cartridge}};

pub mod io;
pub mod cpu;
pub mod memory;
mod boot_rom;

//const CLOCKS_PER_FRAME: u16 = 1;
const CLOCKS_PER_FRAME: u16 = 17556;

#[derive(Debug)]
pub struct GameBoy {
    pub cpu: cpu::CPU,
    io: io::IO,
}

impl GameBoy {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            cpu: CPU::default(),
            io: IO::new(cart),
        }
    }

    pub fn run_frame(&mut self) -> io::LcdPixels {
        let mut clocks = 0;

        while clocks < CLOCKS_PER_FRAME {
            let cycles = self.cpu.execute_opcode(&mut self.memory);
            // commented out becauseit will be completly rewritten
            // TODO: don't forget this
            self.io.run_cycles(cycles * 4);
            clocks += cycles as u16;
        }

        self.io.get_frame()
    }
}