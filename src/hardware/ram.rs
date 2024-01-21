use std::ops::{Index, IndexMut};

pub struct RAM {
    ram: [u8; 0xFFFF]
}

impl Index<usize> for RAM {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.ram[index]
    }
}

impl Index<u16> for RAM {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        self.index(index as usize)
    }
}

impl IndexMut<usize> for RAM {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.ram[index]
    }
}

impl IndexMut<u16> for RAM {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        self.index_mut(index as usize)
    }
}