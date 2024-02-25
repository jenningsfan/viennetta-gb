use crate::hardware::io::LcdPixels;
use crate::hardware::io::{HEIGHT, WIDTH};

pub const PIXEL_SIZE: usize = 2;

pub fn convert_gameboy_to_rgb565(gameboy: LcdPixels) -> [u8; WIDTH * HEIGHT * PIXEL_SIZE] {
    let mut result = [0; WIDTH * HEIGHT * PIXEL_SIZE];

    for (i, pixel) in gameboy.iter().enumerate() {
        result[i * 2] = *pixel as u8; // truncates
        result[i * 2 + 1] = (*pixel >> 8) as u8;
    }

    result
}