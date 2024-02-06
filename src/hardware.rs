use std::io::Read;

pub mod io;
pub mod cpu;
mod memory;
mod boot_rom;

#[derive(Default, Debug)]
pub struct GameBoy {
    pub breakpoint: bool, 
    pub cpu: cpu::CPU,
    io: io::IO,
    pub memory: memory::Memory,
}

impl GameBoy {
    pub fn run_frame(&mut self) -> io::LcdPixels {
        let cycles = self.cpu.execute_opcode(&mut self.memory);
        if self.memory[0xFF02] == 0x81_u8 {
            println!("{}", self.memory[0xFF01] as char);
            self.memory[0xFF02] = 0_u8;
        }
        self.io.run_cycles(1 * 4, &mut self.memory)
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.memory.load_rom(rom);
    }
}