use std::collections::HashSet;
use std::{fs, env};
use std::ops::Index;
use std::io::stdin;
use viennetta_gb::hardware::GameBoy;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = fs::read(&args[1]).unwrap();
    let mut gameboy = GameBoy::default();
    gameboy.load_rom(&rom);

    let mut breakpoint: HashSet<u16> = HashSet::new();
    let mut stepping = true;

    loop {
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
                        breakpoint.insert(u16::from_str_radix(command[1], 16).unwrap());
                    }
                    "rb" => {
                        let offset = u16::from_str_radix(command[1], 16).unwrap();
                        breakpoint.retain(|x| *x != offset);
                    }
                    "r" => {
                        gameboy.cpu.dump_regs();

                    }
                    "q" => {
                        std::process::exit(0);
                    }
                    _ => println!("Not a valid command"),
                }
            }
        }
        
        gameboy.run_frame();
    }
}