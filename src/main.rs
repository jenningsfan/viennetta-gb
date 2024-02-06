use std::{fs, env};
use std::ops::Index;
use std::io::stdin;
use viennetta_gb::hardware::GameBoy;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = fs::read(&args[1]).unwrap();
    let mut gameboy = GameBoy::default();
    gameboy.load_rom(&rom);

    let mut breakpoint: Vec<u16> = vec![];
    let mut stepping = true;

    loop {
        if gameboy.cpu.regs.pc == 0x4750 {
            println!("YES YES YES YES YES");
        }
        if breakpoint.contains(&gameboy.cpu.regs.pc) || stepping {
            stepping = false;
            println!("{:04X}", gameboy.cpu.regs.pc);
            loop {
                let mut command = String::new();
                stdin().read_line(&mut command).unwrap();
                let command: Vec<_> = command.trim().split(" ").collect();

                match command[0] {
                    "c" => break,
                    "i" => {
                        println!("{:02x}", gameboy.memory[gameboy.cpu.regs.pc])
                    }
                    "s" => {
                        stepping = true;
                        break;
                    }
                    "b" => {
                        breakpoint.push(u16::from_str_radix(command[1], 16).unwrap());
                    }
                    "rb" => {
                        let offset = u16::from_str_radix(command[1], 16).unwrap();
                        breakpoint.retain(|x| *x != offset);
                    }
                    "r" => {
                        println!("AF: {:02x}{:02x}", gameboy.cpu.regs.a, gameboy.cpu.regs.flags);
                        println!("BC: {:02x}{:02x}", gameboy.cpu.regs.b, gameboy.cpu.regs.c);
                        println!("DE: {:02x}{:02x}", gameboy.cpu.regs.d, gameboy.cpu.regs.e);
                        println!("HL: {:02x}{:02x}", gameboy.cpu.regs.h, gameboy.cpu.regs.l);
                        println!("HL: {:02x}{:02x}", gameboy.cpu.regs.h, gameboy.cpu.regs.l);
                        println!("SP: {:04x}", gameboy.cpu.regs.sp);
                        println!("PC: {:04x}", gameboy.cpu.regs.pc);

                    }
                    _ => println!("Not a valid command"),
                }
            }
        }
        
        gameboy.run_frame();
    }
}