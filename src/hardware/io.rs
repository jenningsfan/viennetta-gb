mod apu;
mod joypad;
pub mod ppu;
mod timer;

pub use ppu::{WIDTH, HEIGHT, LcdPixels};

use self::ppu::PPU;

#[derive(Debug)]
struct RAM {
    wram: [u8; 0x2000],
    hram: [u8; 0x7E],
}

impl RAM {
    pub fn write_wram(&mut self, address: u16, value: u8) {
        self.wram[address as usize] = value;
    }

    pub fn write_hram(&mut self, address: u16, value: u8) {
        self.hram[address as usize] = value;
    }
}

impl Default for RAM {
    fn default() -> Self {
        Self {
            wram: [0; 0x2000],
            hram: [0; 0x7E],
        }
    }
}

#[derive(Debug)]
pub struct IO {
    pub ppu: ppu::PPU,
    ram: RAM,
}

impl Default for IO {
    fn default() -> Self {
        Self {
            ppu: PPU::default(),
            ram: RAM::default(),
        }
    }
}

impl IO {
    pub fn run_cycles(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.ppu.run_cycle();
        }
    }

    pub fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => self.ppu.write_vram(address - 0x8000, value),   // VRAM
            0xC000..=0xDFFF => self.ram.write_wram(address - 0xC000, value),   // WRAM
            0xE000..=0xFDFF => self.ram.write_wram(address - 0xE000, value),   // Echo RAM
            0xFF80..=0xFFFE => self.ram.write_hram(address - 0xE000, value),   // HRAM
            _ => {},
        }
    }
}