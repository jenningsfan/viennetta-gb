use std::ops::{Index, IndexMut};
use crate::hardware::boot_rom::BOOT_ROM;

const BOOT_ROM_REGISTER: u16 = 0xFF50;

#[derive(Debug)]
pub struct Memory {
    game_rom: [u8; 0x8000],
    boot_rom: [u8; 0x100],
    vram: [u8; 0x2000],
    external_ram: [u8; 0x2000],
    wram: [u8; 0x2000],
    oam_ram: [u8; 0x100],
    io_ram: [u8; 0x80],
    hram: [u8; 0x7F],
    ie: u8,
}

impl Memory {
    pub fn load_rom(&mut self, rom: &[u8]) {
        self.game_rom[0x000..rom.len()].copy_from_slice(rom);
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            game_rom: [0; 0x8000],
            boot_rom: BOOT_ROM,
            vram: [0; 0x2000],
            external_ram: [0; 0x2000],
            wram: [0; 0x2000],
            oam_ram: [0; 0x100],
            io_ram: [0; 0x80],
            hram: [0; 0x7F],
            ie: 0,
        }
    }
}

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        if index < 0x100 && self[BOOT_ROM_REGISTER as usize] == 0 {
            return &self.boot_rom[index];
        }
        else if index < 0x8000 {
            return &self.game_rom[index];
        }
        else if index < 0xA000 {
            return &self.vram[index - 0x8000];
        }
        else if index < 0xC000 {
            return &self.external_ram[index - 0xA000];
        }
        else if index < 0xE000 {
            return &self.wram[index - 0xC000];
        }
        else if index < 0xFE00 {
            // will fall through to end
        }
        else if index < 0xFEA0 {
            return &self.oam_ram[index- 0xFE00];
        }
        else if index < 0xFF00 {
            // will fall through to end
        }
        else if index < 0xFF80 {
            return &self.io_ram[index - 0xFF00];
        }
        else if index < 0xFFFF {
            return &self.hram[index - 0xFF80];
        }
        else if index == 0xFFFF {
            return &self.ie;
        }
        panic!("{index:04X} is an invalid address");
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        self.index(index as usize)
    }
}

impl Index<i32> for Memory {
    type Output = u8;

    fn index(&self, index: i32) -> &Self::Output {
        self.index(index as usize)
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < 0x100 && self[BOOT_ROM_REGISTER as usize] == 0 {
            return &mut self.boot_rom[index];
        }
        else if index < 0x8000 {
            return &mut self.game_rom[index];
        }
        else if index < 0xA000 {
            return &mut self.vram[index - 0x8000];
        }
        else if index < 0xC000 {
            return &mut self.external_ram[index - 0xA000];
        }
        else if index < 0xE000 {
            return &mut self.wram[index - 0xC000];
        }
        else if index < 0xFE00 {
            // will fall through to end
        }
        else if index < 0xFEA0 {
            return &mut self.oam_ram[index - 0xFE00];
        }
        else if index < 0xFF00 {
            // will fall through to end
        }
        else if index < 0xFF80 {
            return &mut self.io_ram[index - 0xFF00];
        }
        else if index < 0xFFFF {
            return &mut self.hram[index - 0xFF80];
        }
        else if index == 0xFFFF {
            return &mut self.ie;
        }
        panic!("{index:04X} is an invalid address");
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        self.index_mut(index as usize)
    }
}

impl IndexMut<i32> for Memory {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        self.index_mut(index as usize)
    }
}