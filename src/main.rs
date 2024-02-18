use std::collections::HashSet;
use std::{fs, env};
use std::io::stdin;
use viennetta_gb::hardware::io::Cartridge;
use viennetta_gb::hardware::GameBoy;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = fs::read(&args[1]).expect(format!("{} is not a valid path\n", args[1]).as_str());
    let mut gameboy = GameBoy::new(Cartridge::new(&rom));

    let mut breakpoint: HashSet<u16> = HashSet::new();
    let mut stepping = false;

    if args.contains(&"--debugger".to_string()) {
        stepping = true;
    }

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
                        println!("{:02x}", gameboy.mmu.read_memory(gameboy.cpu.regs.pc))
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
                    "ch" => {
                        for i in 0..4 {
                            print!("{:02X}", gameboy.mmu.read_memory(0xFF80 + i));
                        }
                        println!();
                    }
                    _ => println!("Not a valid command"),
                }
            }
        }
        //println!("{:04X}", gameboy.cpu.regs.pc);
        gameboy.run_frame();
    }
}