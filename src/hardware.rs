pub mod io;
pub mod cpu;
mod memory;
mod boot_rom;

#[derive(Default, Debug)]
pub struct GameBoy {
    io: io::IO,
    cpu: cpu::CPU,
    memory: memory::Memory,
}

impl GameBoy {
    pub fn run_frame(&mut self) -> io::LcdPixels {
        self.cpu.execute_opcode(&mut self.memory);
        self.io.run_cycles(1, &mut self.memory)
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.memory.load_rom(rom);
    }
}