use std::collections::HashSet;
use std::os::windows::process;
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
    let mut debugging = false;
    let mut blaargs = false;

    if args.contains(&"--debugger".to_string()) {
        //stepping = true;
        debugging = true;
        breakpoint.insert(0x100);
    }

    if args.contains(&"--blaargs".to_string()) {
        blaargs = true;
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
                    "ppu" => {
                        let stat = gameboy.mmu.ppu.status;
                        println!("LCDC: {:02X}", gameboy.mmu.ppu.lcdc);
                        println!("STAT: {:02X}", stat);
                        println!("LY: {:02X}", gameboy.mmu.ppu.line_y);
                        println!("LYC: {:02X}", gameboy.mmu.ppu.line_compare);
                        println!("line cycles: {}", gameboy.mmu.ppu.cycles_line);
                        println!("BG pal: {:02X}", gameboy.mmu.ppu.palettes.bg_palette);
                        println!("OBJ0 pal: {:02X}", gameboy.mmu.ppu.palettes.obj0_palette);
                        println!("OBJ1 pal: {:02X}", gameboy.mmu.ppu.palettes.obj1_palette);

                        let mode = match stat & 0x3 {
                            0 => "H-Blank",
                            1 => "V-Blank",
                            2 => "OAM Scan",
                            3 => "Drawing",
                            _ => panic!("impossible"),
                        };

                        println!("Mode: {mode} ({})", stat & 0x3);
                    }
                    _ => println!("Not a valid command"),
                }
            }
        }
        //println!("{:04X}", gameboy.cpu.regs.pc);
        if debugging {
            gameboy.run_instruction();
        }
        else {
            gameboy.run_frame();
        }

        if gameboy.cpu.regs.pc == 0xCBB0 && blaargs {
            return;
        }
    }
}