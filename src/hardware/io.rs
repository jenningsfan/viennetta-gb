mod apu;
mod joypad;
mod serial;
pub mod ppu;
mod timer;

pub use ppu::{WIDTH, HEIGHT, LcdPixels};
use self::ppu::PPU;
use self::serial::Serial;
use super::boot_rom::BOOT_ROM;

#[derive(Debug)]
struct RAM {
    wram: [u8; 0x2000],
    hram: [u8; 0x7F],
}

impl RAM {
    pub fn read_wram(&self, address: u16) -> u8 {
        self.wram[address as usize]
    }

    pub fn read_hram(&self, address: u16) -> u8 {
        self.hram[address as usize]
    }

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
            hram: [0; 0x7F],
        }
    }
}

#[derive(Debug)]
pub struct Cartridge {
    rom: [u8; 0x8000],
}

impl Cartridge {
    pub fn new(game_rom: &[u8]) -> Self {
        let mut rom = [0; 0x8000];
        rom.copy_from_slice(game_rom);

        Self {
            rom,
        }
    }

    fn read_rom(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    fn write_rom(&mut self, _address: u16, _value: u8) {
        // Add mapper support here later
        // It is left empty on purpose
        // TODO: MBC
    }

    fn read_ram(&self, _address: u16) -> u8 {
        // This would be external ram
        // Left empty on purpose
        // TODO: MBC
        0xFF
    }

    fn write_ram(&mut self, _address: u16, _value: u8) {
        // Add mapper support here later
        // It is left empty on purpose
        // TODO: MBC
    }
}

#[derive(Debug)]
pub struct MMU {
    ppu: PPU,
    ram: RAM,
    serial: Serial,
    cart: Cartridge,
    int_enable: u8,
    boot_rom_enable: u8,
}

impl MMU {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            ppu: PPU::default(),
            ram: RAM::default(),
            serial: Serial::default(),
            cart,
            int_enable: 0,
            boot_rom_enable: 0,
        }
    }
}

impl MMU {
    pub fn run_cycles(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.ppu.run_cycle();
        }
    }

    pub fn get_frame(&self) -> LcdPixels {
        self.ppu.get_frame()
    }

    pub fn read_memory(&self, address: u16) -> u8 {
        if address < 0x100 && self.boot_rom_enable == 0 {
            return BOOT_ROM[address as usize];
        }

        match address {
            0x0000..=0x7FFF => self.cart.read_rom(address),                     // ROM
            0x8000..=0x9FFF => self.ppu.read_vram(address - 0x8000),   // VRAM
            0xA000..=0xBFFF => self.cart.read_ram(address - 0xA000),   // External RAM (MBC)
            0xC000..=0xDFFF => self.ram.read_wram(address - 0xC000),   // WRAM
            0xE000..=0xFDFF => self.ram.read_wram(address - 0xE000),   // Echo RAM
            0xFE00..=0xFE9F => self.ppu.read_oam(address - 0xFE00),    // OAM
            0xFF80..=0xFFFE => self.ram.read_hram(address - 0xFF80),   // HRAM
            0xFF01 => self.serial.read_data(),      // Serial Data
            0xFF02 => self.serial.read_control(),   // Serial Control
            0xFF50 => self.boot_rom_enable,         // Boot ROM Enable/Disable
            0xFFFF => self.int_enable,              // Interrupt Enable
            _ => 0x00,  
        }
    }

    pub fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.cart.write_rom(address, value),                     // ROM
            0x8000..=0x9FFF => self.ppu.write_vram(address - 0x8000, value),   // VRAM
            0xA000..=0xBFFF => self.cart.write_ram(address - 0xA000, value),   // External RAM (MBC)
            0xC000..=0xDFFF => self.ram.write_wram(address - 0xC000, value),   // WRAM
            0xE000..=0xFDFF => self.ram.write_wram(address - 0xE000, value),   // Echo RAM
            0xFE00..=0xFE9F => self.ppu.write_oam(address - 0xFE00, value),    // OAM
            0xFF80..=0xFFFE => self.ram.write_hram(address - 0xFF80, value),   // HRAM
            0xFF01 => self.serial.write_data(value),      // Serial Data
            0xFF02 => self.serial.write_control(value),   // Serial Control
            0xFF50 => self.boot_rom_enable = value,       // Boot ROM Enable/Disable
            0xFFFF => self.int_enable = value,            // Interrupt Enable
            _ => {},
        }
    }
}