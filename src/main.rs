use std::collections::HashSet;
use std::{fs, env};
use std::io::stdin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use viennetta_gb::hardware::io::cart::Cartridge;
use viennetta_gb::hardware::GameBoy;
use viennetta_gb::hardware::io::joypad::Buttons;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = fs::read(&args[1]).expect(format!("{} is not a valid path\n", args[1]).as_str());
    let mut gameboy = GameBoy::new(Cartridge::new(&rom));

    let mut breakpoint: HashSet<u16> = HashSet::new();
    let stepping = Arc::new(AtomicBool::new(false));
    let mut debugging = false;
    let mut blaargs = false;
    let mut trace = false;

    gameboy.mmu.joypad.update_state(Buttons::from_bits(0).unwrap());

    if args.contains(&"--debugger".to_string()) {
        //stepping = true;
        debugging = true;
        breakpoint.insert(0x100);

        // enable ctrl+c to break into debugger
        let stepping = stepping.clone();
        ctrlc::set_handler(move || {
            stepping.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
    }

    if args.contains(&"--blaargs".to_string()) {
        blaargs = true;
    }

    if args.contains(&"--trace".to_string()) {
        trace = true;
        debugging = true;
        stepping.store(true, Ordering::SeqCst);
    }

    loop {
        if breakpoint.contains(&gameboy.cpu.regs.pc) || stepping.load(Ordering::SeqCst) {
            if trace {
                if gameboy.cpu.regs.pc >= 0x100 {
                    gameboy.cpu.trace_regs();
                }
            }
            else {
                stepping.store(false, Ordering::SeqCst);
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
                            stepping.store(true, Ordering::SeqCst);
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
                        "timer" => {
                            gameboy.mmu.timer.debug();
                        }
                        _ => println!("Not a valid command"),
                    }
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