use crate::hardware::io::ppu::Colour;
use crate::hardware::io::LcdPixels;
use crate::hardware::io::{HEIGHT, WIDTH};

pub const PIXEL_SIZE: usize = 2;

pub fn convert_gameboy_to_rgb565(gameboy: LcdPixels) -> Vec<u8> {
    let mut result: Vec<u16> = vec![];

    for pixel in gameboy {
        result.push(match pixel {
            Colour::White => 0xFFFF,
            Colour::LightGrey => 0xB573,
            Colour::DarkGrey => 0x6B4B,
            Colour::Black => 0x0000,
        });
    }

    result.iter().flat_map(|&x| x.to_ne_bytes().to_vec()).collect()
}

// pub fn convert_gameboy_to_rgb565(gameboy: LcdPixels) -> [u8; WIDTH * HEIGHT * PIXEL_SIZE] {
//     let mut result = [0; WIDTH * HEIGHT * PIXEL_SIZE];

//     for i in 0..gameboy.len() {
//         let pixel = gameboy[i];

//         result[i] = match pixel {
//             Colour::White => 0xFF,
//             Colour::LightGrey => 0x73,
//             Colour::DarkGrey => 0x4B,
//             Colour::Black => 0x00,
//         };

//         result[i + 1] = match pixel {
//             Colour::White => 0xFF,
//             Colour::LightGrey => 0xB5,
//             Colour::DarkGrey => 0x6B,
//             Colour::Black => 0x00,
//         };
//     }

//     result
// }