use std::ops::{Index, IndexMut};
use crate::hardware::boot_rom::BOOT_ROM;

#[derive(Debug)]
pub struct Memory {
    memory: Vec<u8>
}

impl Memory {
    pub fn load_rom(&mut self, rom: &[u8]) {
        self.memory[0x100..0x100 + rom.len()].copy_from_slice(rom);
    }
}

impl Default for Memory {
    fn default() -> Self {
        let mut ram = vec![0; 0xFFFF];
        ram[0..0x100].copy_from_slice(&BOOT_ROM);

        Self {
            memory: ram
        }
    }
}

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        self.index(index as usize)
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.memory[index]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        self.index_mut(index as usize)
    }
}