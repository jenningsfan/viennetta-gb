use std::{fs, env};
use viennetta_gb::hardware::GameBoy;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = fs::read(args[1]).unwrap();
    let mut gameboy = GameBoy::default();
    gameboy.load_rom(&rom);

    loop {
        gameboy.run_frame();
    }
}