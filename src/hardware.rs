use std::io::Read;
use std::io::Write;

pub mod io;
pub mod cpu;
pub mod memory;
mod boot_rom;

const CLOCKS_PER_FRAME: u16 = 1;
//const CLOCKS_PER_FRAME: u16 = 17556;

#[derive(Default, Debug)]
pub struct GameBoy {
    pub breakpoint: bool,
    pub cpu: cpu::CPU,
    io: io::IO,
    pub memory: memory::Memory,
}

impl GameBoy {
    pub fn run_frame(&mut self) -> io::LcdPixels {
        let mut clocks = 0;

        while clocks < CLOCKS_PER_FRAME {
            let cycles = self.cpu.execute_opcode(&mut self.memory);
            if self.memory[0xFF02] == 0x81_u8 {
                print!("{}", self.memory[0xFF01] as char);
                std::io::stdout().flush().unwrap();
                self.memory[0xFF02] = 0x01_u8;
            }
            // commented out becauseit will be completly rewritten
            // TODO: don't forget this
            //self.io.run_cycles(cycles * 4, &mut self.memory);
            clocks += cycles as u16;
        }

        self.io.ppu.get_frame()
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.memory.load_rom(rom);
    }
}