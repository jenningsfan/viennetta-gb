use crate::hardware::memory::{self, Memory};

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
pub type LcdPixels = [Colour; WIDTH * HEIGHT];

#[derive(Debug, Clone, Copy)]
pub enum Colour {
    White = 0,
    LightGrey = 1,
    DarkGrey = 2,
    Black = 3,
}

impl From<u8> for Colour {
    fn from(value: u8) -> Self {
        match value {
            0 => Colour::White,
            1 => Colour::LightGrey,
            2 => Colour::DarkGrey,
            3 => Colour::Black,
            _ => panic!("Invalid u8 value for Colour: {value}"),
        }
    }
}

#[derive(Debug)]
pub struct PPU {
    lcd: LcdPixels,
}

impl Default for PPU {
    fn default() -> Self {
        Self {
            lcd: [Colour::White; WIDTH * HEIGHT]
        }
    }
}

impl PPU {
    pub fn get_frame(&self) -> LcdPixels {
        self.lcd
    }

    pub fn run_cycle(&mut self, memory: &Memory) {
        let tile = self.get_tile(0, memory);
        for i in 0..8 {
            for j in 0..8 {
                self.lcd[i * WIDTH + j] = tile[i * 8 + j];
            }
        }
    }

    fn get_tile(&self, offset: usize, memory: &Memory) -> [Colour; 64] {
        // TODO: this isn't very nice. redo to use a range
        let mut tiles_bytes = vec![];
        for i in 0..16 {
            tiles_bytes.push(memory[0x8000 + offset + i]);
        }
        
        let mut tiles = [Colour::Black; 64];
        for (i, tile) in tiles_bytes.chunks(2).enumerate() {
            let high = tile[0];
            let low = tile[1];
            
            for j in 0..8 {
                let tile = (((high >> j) & 1) << 1) | (low >> j) & 1;
                tiles[i * 8 + (7 - j)] = Colour::from(tile);
            }
        }

        tiles
    }
}