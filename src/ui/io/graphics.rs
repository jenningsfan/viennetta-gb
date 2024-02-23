use crate::hardware::io::ppu::Colour;
use crate::hardware::io::LcdPixels;
use crate::hardware::io::{HEIGHT, WIDTH};

pub const PIXEL_SIZE: usize = 2;

pub fn convert_gameboy_to_rgb565(gameboy: LcdPixels) -> [u8; WIDTH * HEIGHT * PIXEL_SIZE] {
    let mut result = [0; WIDTH * HEIGHT * PIXEL_SIZE];

     for (i, pixel) in gameboy.iter().enumerate() {
        let i = i * 2;

        // result[i] = match pixel {
        //     Colour::White => 0xFF,
        //     Colour::LightGrey => 0x73,
        //     Colour::DarkGrey => 0x4B,
        //     Colour::Black => 0x00,
        // };

        // result[i + 1] = match pixel {
        //     Colour::White => 0xFF,
        //     Colour::LightGrey => 0xB5,
        //     Colour::DarkGrey => 0x6B,
        //     Colour::Black => 0x00,
        // };
    }

    result
}