pub mod joypad;
pub mod ppu;
pub mod cart;
mod apu;
mod serial;
mod timer;

use bitflags::bitflags;
pub use ppu::{WIDTH, HEIGHT, LcdPixels};
use self::ppu::PPU;
use self::serial::Serial;
use self::timer::Timer;
use self::joypad::Joypad;
use super::boot_rom::BOOT_ROM;
use self::cart::Cartridge;

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

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct Interrupts: u8 {
        const VBlank  = 1 << 0;
        const LcdStat = 1 << 1;
        const Timer   = 1 << 2;
        const Serial  = 1 << 3;
        const Joypad  = 1 << 4;
    }
}

#[derive(Debug)]
pub struct MMU {
    pub ppu: PPU,
    ram: RAM,
    serial: Serial,
    pub timer: Timer,
    pub cart: Cartridge,
    pub joypad: Joypad,
    pub int_enable: Interrupts,
    pub int_flag: Interrupts,
    boot_rom_enable: u8,
    last_dma_value: u8,
}

impl MMU {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            ppu: PPU::default(),
            ram: RAM::default(),
            serial: Serial::default(),
            timer: Timer::default(),
            joypad: Joypad::default(),
            cart,
            int_enable: Interrupts::empty(),
            int_flag: Interrupts::empty(),
            boot_rom_enable: 0,
            last_dma_value: 0,
        }
    }
}

impl MMU {
    pub fn run_cycles(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.int_flag |= self.ppu.run_cycles(4);
            self.int_flag |= self.timer.run_cycles(4);

            // if let Some(addr) = self.dma_transfer_offset {
            //     self.ppu.write_oam(addr & 0xFF, self.read_memory(addr));
            //     self.dma_transfer_offset = Some(addr + 1);
            //     if (addr + 1) & 0xA0 == 0xA0 {
            //         self.dma_transfer_offset = None;
            //     }
            // }
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
            0xA000..=0xBFFF => self.cart.read_ram(address - 0xA000),  // External RAM (MBC)
            0xC000..=0xDFFF => self.ram.read_wram(address - 0xC000),   // WRAM
            0xE000..=0xFDFF => self.ram.read_wram(address - 0xE000),   // Echo RAM
            0xFE00..=0xFE9F => self.ppu.read_oam(address - 0xFE00),    // OAM
            0xFF80..=0xFFFE => self.ram.read_hram(address - 0xFF80),   // HRAM
            0xFF00 => self.joypad.read(),                                       // Joypad
            0xFF01 => self.serial.read_data(),                                  // Serial Data
            0xFF02 => self.serial.read_control(),                               // Serial Control
            0xFF04..=0xFF07 => self.timer.read_io(address),                 // Timer
            0xFF46 => self.last_dma_value,                                  // OAM DMA
            0xFF40..=0xFF4B => self.ppu.read_io(address),                       // PPU
            0xFF0F => self.int_flag.bits() as u8,                               // Interrupt Enable
            0xFF50 => self.boot_rom_enable,                                     // Boot ROM Enable/Disable
            0xFFFF => self.int_enable.bits() as u8,                             // Interrupt Enable
            _ => 0xFF,
        }
    }

    pub fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.cart.write_rom(address, value),                     // ROM
            0x8000..=0x9FFF => self.ppu.write_vram(address - 0x8000, value),   // VRAM
            0xA000..=0xBFFF => self.cart.write_ram(address - 0xA000, value),  // External RAM (MBC)
            0xC000..=0xDFFF => self.ram.write_wram(address - 0xC000, value),   // WRAM
            0xE000..=0xFDFF => self.ram.write_wram(address - 0xE000, value),   // Echo RAM
            0xFE00..=0xFE9F => self.ppu.write_oam(address - 0xFE00, value),    // OAM
            0xFF80..=0xFFFE => self.ram.write_hram(address - 0xFF80, value),   // HRAM
            0xFF00 => self.joypad.write(value),                                         // Joypad
            0xFF01 => self.serial.write_data(value),                                    // Serial Data
            0xFF02 => self.serial.write_control(value),                                 // Serial Control
            0xFF04..=0xFF07 => self.timer.write_io(address, value),                 // Timer
            0xFF46 => self.oam_dma(value),                                      // OAM DMA
            0xFF40..=0xFF4B => self.ppu.write_io(address, value),                       // PPU
            0xFF0F => self.int_flag = Interrupts::from_bits(value & 0x1F).unwrap(),     // Interrupt Enable
            0xFF50 => self.boot_rom_enable = value,                                     // Boot ROM Enable/Disable
            0xFFFF => self.int_enable = Interrupts::from_bits(value & 0x1F)
                .expect(format!("{:02X} is not a valid IE value", value & 0x1F).as_str()),   // Interrupt Enable
            _ => {},
        }
    }

    fn oam_dma(&mut self, address: u8) {
        // self.last_dma_value = address;
        // self.dma_transfer_offset = Some((address as u16) << 8);

        for offset in 0..0xA0 {
            self.ppu.write_oam(offset, self.read_memory(((address as u16) << 8) | offset));
        }
    }
}