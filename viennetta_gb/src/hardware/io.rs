pub mod joypad;
pub mod ppu;
pub mod cart;
pub mod apu;
mod serial;
mod timer;

use dbg_hex::dbg_hex;
use bitflags::bitflags;
use log::warn;
pub use ppu::{WIDTH, HEIGHT, LcdPixels};
use self::ppu::PPU;
use self::apu::APU;
use self::serial::Serial;
use self::timer::Timer;
use self::joypad::Joypad;
use super::boot_rom::{DMG_BOOT_ROM, CGB_BOOT_ROM};
use self::cart::Cartridge;

pub const T_CYCLES_RATE: u32 = 4 * 1024 * 1024;
pub const M_CYCLES_RATE: u32 = 1 * 1024 * 1024;

#[derive(Debug)]
struct RAM {
    wram: [u8; 0x8000],
    hram: [u8; 0x7F],
    pub wram_bank: u8,
}

impl RAM {
    pub fn read_wram(&self, mut address: u16) -> u8 {
        let bank = if self.wram_bank == 0 { 0 } else { self.wram_bank - 1};
        if address >= 0x1000 {
            address += bank as u16 * 0x1000;
        }

        self.wram[address as usize]
    }

    pub fn read_hram(&self, address: u16) -> u8 {
        self.hram[address as usize]
    }

    pub fn write_wram(&mut self, mut address: u16, value: u8) {
        let bank = if self.wram_bank == 0 { 0 } else { self.wram_bank - 1};
        if address >= 0x1000 {
            address += bank as u16 * 0x1000;
        }

        self.wram[address as usize] = value;
    }

    pub fn write_hram(&mut self, address: u16, value: u8) {
        self.hram[address as usize] = value;
    }
}

impl Default for RAM {
    fn default() -> Self {
        Self {
            wram: [0; 0x8000],
            hram: [0; 0x7F],
            wram_bank: 1,
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
    pub apu: APU,
    ram: RAM,
    serial: Serial,
    pub timer: Timer,
    pub cart: Cartridge,
    pub joypad: Joypad,
    pub int_enable: Interrupts,
    pub int_flag: Interrupts,
    boot_rom_enable: u8,
    last_dma_value: u8,
    ff72: u8,
    ff73: u8,
    ff74: u8,
    ff75: u8,
    vram_dma_source: u16,
    vram_dma_dest: u16,
    vram_dma_len: u8,
    pub speed_switch: u8,
}

impl MMU {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            ppu: PPU::default(),
            ram: RAM::default(),
            apu: APU::default(),
            serial: Serial::default(),
            timer: Timer::default(),
            joypad: Joypad::default(),
            cart,
            int_enable: Interrupts::empty(),
            int_flag: Interrupts::empty(),
            boot_rom_enable: 0,
            last_dma_value: 0,
            ff72: 0,
            ff73: 0,
            ff74: 0,
            ff75: 0,
            vram_dma_source: 0,
            vram_dma_dest: 0,
            vram_dma_len: 0,
            speed_switch: 0,
        }
    }
}

impl MMU {
    pub fn run_cycles(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.int_flag |= self.ppu.run_cycles(4);
            self.int_flag |= self.timer.run_cycles(4);
            self.apu.run_cycles(4);

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
        if self.boot_rom_enable == 0 {
            if address < 0x100 {
                return CGB_BOOT_ROM[address as usize];
            }
            else if address >= 0x200 && address < 0x8FF {
                return CGB_BOOT_ROM[address as usize];                
            }
        }

        match address {
            0x0000..=0x7FFF => self.cart.read_rom(address),                     // ROM
            0x8000..=0x9FFF => self.ppu.read_vram(address - 0x8000),   // VRAM
            0xA000..=0xBFFF => self.cart.read_ram(address - 0xA000),   // External RAM (MBC)
            0xC000..=0xDFFF => self.ram.read_wram(address - 0xC000),   // WRAM
            0xE000..=0xFDFF => self.ram.read_wram(address - 0xE000),   // Echo RAM
            0xFE00..=0xFE9F => self.ppu.read_oam(address - 0xFE00),    // OAM
            0xFF80..=0xFFFE => self.ram.read_hram(address - 0xFF80),   // HRAM
            0xFF00 => self.joypad.read(),                                       // Joypad
            0xFF01 => self.serial.read_data(),                                  // Serial Data
            0xFF02 => self.serial.read_control(),                               // Serial Control
            0xFF04..=0xFF07 => self.timer.read_io(address),                 // Timer
            0xFF10..=0xFF26 => self.apu.read_io(address),                       // APU
            0xFF30..=0xFF3F => self.apu.read_wave(address - 0xFF30),   // APU Wave Pattern
            0xFF46 => self.last_dma_value,                                      // OAM DMA
            0xFF40..=0xFF4B => self.ppu.read_io(address),                       // PPU
            0xFF0F => self.int_flag.bits() as u8,                               // Interrupt Enable
            0xFF4D => self.speed_switch,                                        // speed switch
            0xFF4F => self.ppu.read_io(address),                                // PPU
            0xFF50 => self.boot_rom_enable,                                     // Boot ROM Enable/Disable
            0xFF51 => (self.vram_dma_source >> 8) as u8,                        // VRAM DMA
            0xFF52 => (self.vram_dma_source & 0xFF) as u8,                       // VRAM DMA
            0xFF53 => (self.vram_dma_dest >> 8) as u8,                          // VRAM DMA
            0xFF54 => (self.vram_dma_dest & 0xFF) as u8,                         // VRAM DMA
            0xFF55 => { warn!("TODO: proper VRAM DMA transfer"); 0xFF },        // VRAM DMA
            0xFF56 => { warn!("TODO: IR port read"); 0x0 },                     // IR port
            0xFF68..0xFF6C => self.ppu.read_io(address),                        // PPU
            0xFF70 => self.ram.wram_bank,                                       // WRAM bank
            0xFF72 => self.ff72,                                                // FF72
            0xFF73 => self.ff73,                                                // FF73
            0xFF74 => self.ff74,                                                // FF74
            0xFF75 => self.ff75,                                                // FF75
            0xFFFF => self.int_enable.bits() as u8,                             // Interrupt Enable
            _ => 0xFF,
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
            0xFF00 => self.joypad.write(value),                                         // Joypad
            0xFF01 => self.serial.write_data(value),                                    // Serial Data
            0xFF02 => self.serial.write_control(value),                                 // Serial Control
            0xFF04..=0xFF07 => self.timer.write_io(address, value),                // Timer
            0xFF10..=0xFF26 => self.apu.write_io(address, value),                       // APU
            0xFF30..=0xFF3F => self.apu.write_wave(address - 0xFF30, value),   // APU Wave Pattern
            0xFF4D => self.speed_switch = value & 0x81,                                 // speed switch
            0xFF46 => self.oam_dma(value),                                      // OAM DMA
            0xFF40..=0xFF4B => self.ppu.write_io(address, value),                       // PPU
            0xFF4F => self.ppu.write_io(address, value),                                // PPU
            0xFF0F => self.int_flag = Interrupts::from_bits(value & 0x1F).unwrap(),     // Interrupt Enable
            0xFF50 => self.boot_rom_enable = value,                                     // Boot ROM Enable/Disable
            0xFF51 => self.vram_dma_source = (self.vram_dma_dest & 0xFF) | (value as u16) << 8, // VRAM DMA
            0xFF52 => self.vram_dma_source = (self.vram_dma_source & 0xFF00) | (value as u16),  // VRAM DMA
            0xFF53 => self.vram_dma_dest = (self.vram_dma_dest & 0xFF) | (value as u16) << 8,   // VRAM DMA
            0xFF54 => self.vram_dma_dest = (self.vram_dma_dest & 0xFF00) | (value as u16),      // VRAM DMA
            0xFF55 => { self.vram_dma_len = value & 0x7F; self.vram_dma() },                             // VRAM DMA
            0xFF56 => warn!("TODO: IR port write"),                                     // IR port
            0xFF68..=0xFF6C => self.ppu.write_io(address, value),                       // PPU
            0xFF70 => self.ram.wram_bank = value & 0x7,                                 // WRAM bank
            0xFF72 => self.ff72 = value,                                                // FF72
            0xFF73 => self.ff73 = value,                                                // FF73
            0xFF74 => self.ff74 = value,                                                // FF74
            0xFF75 => self.ff75 = value & 0x70,                                         // FF75
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

    fn vram_dma(&mut self) {
        let source = self.vram_dma_source;
        let dest = (self.vram_dma_dest & 0x1FF0) + 0x8000;
        let len = (self.vram_dma_len as u16 + 1) * 0x10;

        for i in 0..len {
            self.write_memory(dest + i, self.read_memory(source + i));
        }

        // dbg_hex!(source);
        // dbg_hex!(dest);
        // dbg_hex!(len);
    }
}