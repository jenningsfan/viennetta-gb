use crate::hardware::io::LcdPixels;
use crate::hardware::io::{HEIGHT, WIDTH};

pub const PIXEL_SIZE: usize = 2;

const COLOURS: [u16; 4] = [0xFFFF, 0xB573, 0x6B4B, 0x0000];

pub fn convert_gameboy_to_rgb565(gameboy: LcdPixels) -> [u8; WIDTH * HEIGHT * PIXEL_SIZE] {
    let mut result = [0; WIDTH * HEIGHT * PIXEL_SIZE];

    for (i, pixel) in gameboy.iter().enumerate() {
        let colour = COLOURS[*pixel as usize];
        result[i * 2] = colour as u8; // truncates
        result[i * 2 + 1] = (colour >> 8) as u8;
    }

    result
}