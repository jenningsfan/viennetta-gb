mod apu;
mod joypad;
pub mod ppu;
mod timer;

pub use ppu::{WIDTH, HEIGHT, LcdPixels};

#[derive(Default, Debug)]
pub struct IO {
    pub ppu: ppu::PPU,
}

impl IO {
    pub fn run_cycles(&mut self, cycles: u8) -> LcdPixels {
        for _ in 0..cycles {
            self.ppu.run_cycle();
        }
        self.ppu.get_frame()
    }
}