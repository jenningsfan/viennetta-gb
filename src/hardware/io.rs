mod apu;
mod joypad;
pub mod ppu;
mod timer;

pub use ppu::{WIDTH, HEIGHT, LcdPixels};

use super::memory::Memory;

#[derive(Default, Debug)]
pub struct IO {
    pub ppu: ppu::PPU,
}

impl IO {
    pub fn run_cycles(&mut self, cycles: u8, memory: &mut Memory) -> LcdPixels {
        for _ in 0..cycles {
            self.ppu.run_cycle(memory);
        }
        self.ppu.get_frame()
    }
}