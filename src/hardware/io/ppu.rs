use crate::hardware::memory::Memory;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
pub type LcdPixels = [Colour; WIDTH * HEIGHT];
type Tile = [Colour; 64];

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
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
            lcd: [Colour::Black; WIDTH * HEIGHT]
        }
    }
}

impl PPU {
    pub fn get_frame(&self) -> LcdPixels {
        self.lcd
    }

    pub fn run_cycle(&mut self, memory: &mut Memory) {
        for i in 0..1 {
            memory[0x8000 + 16_usize * i] = 0x3C;
            memory[0x8000 + 16_usize * i + 1_usize] = 0x7E;
            memory[0x8000 + 16_usize * i + 2_usize] = 0x42;
            memory[0x8000 + 16_usize * i + 3_usize] = 0x42;
            memory[0x8000 + 16_usize * i + 4_usize] = 0x42;
            memory[0x8000 + 16_usize * i + 5_usize] = 0x42;
            memory[0x8000 + 16_usize * i + 6_usize] = 0x42;
            memory[0x8000 + 16_usize * i + 7_usize] = 0x42;
            memory[0x8000 + 16_usize * i + 8_usize] = 0x7E;
            memory[0x8000 + 16_usize * i + 9_usize] = 0x5E;
            memory[0x8000 + 16_usize * i + 10_usize] = 0x7E;
            memory[0x8000 + 16_usize * i + 11_usize] = 0x0A;
            memory[0x8000 + 16_usize * i + 12_usize] = 0x7C;
            memory[0x8000 + 16_usize * i + 13_usize] = 0x56;
            memory[0x8000 + 16_usize * i + 14_usize] = 0x38;
            memory[0x8000 + 16_usize * i + 15_usize] = 0x7C;
        }

        for i in 0..256 {
            let tile = self.get_tile(i as u8, memory);
            let x = (i * 8) % WIDTH;
            let y = (i / 20) * 8;
            self.draw_tile(&tile, x, y);
        }
    }

    fn draw_tile(&mut self, tile: &Tile, x: usize, y: usize) {
        for i in 0..8 {
            let start = (i + y) * WIDTH + x;
            self.lcd[start..start + 8].copy_from_slice(&tile[i * 8.. i * 8 + 8]);
        }
    }

    fn get_tile(&self, offset: u8, memory: &Memory) -> Tile {
        // TODO: this isn't very nice. redo to use a range
        let mut tiles_bytes = [0; 16];

        if memory[0xFF40] & 0x40_u8 == 0_u8 {
            let offset = (offset as i8 as isize) * 16;
            for i in 0..16 {
                tiles_bytes[i] = memory[(0x9000 + offset + i as isize) as usize];
            }
        }
        else {
            for i in 0..16 {
                let offset = offset as usize * 16;
                tiles_bytes[i] = memory[0x8000 + offset + i];
            }
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