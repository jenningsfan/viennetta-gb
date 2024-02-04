use crate::hardware::io::ppu::Colour;
use crate::hardware::io::LcdPixels;
use crate::hardware::io::{HEIGHT, WIDTH};

pub const PIXEL_SIZE: u32 = 2;

pub fn convert_gameboy_to_rgb565(gameboy: LcdPixels) -> Vec<u8> {
    let mut result: Vec<u16> = vec![];

    for pixel in gameboy {
        result.push(match pixel {
            Colour::White => 0x0000,
            Colour::LightGrey => 0xB573,
            Colour::DarkGrey => 0x6B4B,
            Colour::Black => 0xFFFF,
        });
    }

    result.iter().flat_map(|&x| x.to_ne_bytes().to_vec()).collect()
}