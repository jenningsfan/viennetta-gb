use std::collections::HashSet;
use std::{env, fs, thread};
use std::io::stdin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use viennetta_gb::hardware::io::cart::Cartridge;
use viennetta_gb::hardware::GameBoy;
use viennetta_gb::hardware::io::joypad::Buttons;

const PIXEL_SIZE: usize = 2;
const COLOURS: [u32; 4] = [0xFFFFFFFF, 0X706869, 0xaba5a8, 0x0000];

pub fn convert_gameboy_to_rgb(gameboy: LcdPixels) -> [u8; WIDTH * HEIGHT * PIXEL_SIZE] {
    let mut result = [0; WIDTH * HEIGHT * PIXEL_SIZE];

    for (i, pixel) in gameboy.iter().enumerate() {
        let colour = COLOURS[*pixel as usize];
        result[i * 2] = colour as u8; // truncates
        result[i * 2 + 1] = (colour >> 8) as u8;
    }

    result
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = fs::read(&args[1]).expect(format!("{} is not a valid path\n", args[1]).as_str());
    let mut gameboy = GameBoy::new(Cartridge::new(&rom));

    let mut fb: linuxfb::Framebuffer = linuxfb::Framebuffer::new("/dev/fb0").unwrap();
    let mut buffer = linuxfb::double::Buffer::new(fb).unwrap();
    let frame: &mut[u8] = buffer.as_mut_slice();


    for i in 0..frame.len() {
        frame[i] = 0;
    }
    buffer.flip().unwrap();

    gameboy.mmu.joypad.update_state(Buttons::from_bits(0).unwrap());
    loop {
        let image = gameboy.run_frame();
        let frame: &mut[u8] = buffer.as_mut_slice();
        let (prefix, pixels, suffix) = unsafe { frame.align_to_mut::<u32>() };
        assert_eq!(prefix.len(), 0);
        assert_eq!(suffix.len(), 0);
        *pixels = convert_gameboy_to_rgb(image);

        thread::sleep_ms(1000 / 60);
    }
}