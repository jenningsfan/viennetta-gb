#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use log::error;
use std::collections::HashSet;
use std::{env, fs, path::Path, fs::File};
use std::io::Write;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use viennetta_gb::hardware::{io::{cart::Cartridge, HEIGHT, WIDTH, LcdPixels, joypad::Buttons}, GameBoy};
use viennetta_gb::disasm::disasm;

const PIXEL_SIZE: usize = 4;

enum Mode {
    Normal,
    TileDump,
}

/// Representation of the application state. In this example, a box will bounce around the screen.
struct State {
    gameboy: GameBoy,
    mode: Mode,
    stepping: bool,
    breakpoints: HashSet<u16>,
    prev: u16,
}

impl State {
    fn new(rom: &[u8]) -> Self {
        let mut breakpoints = HashSet::new();
        breakpoints.insert(0x150);

        Self {
            gameboy: GameBoy::new(Cartridge::new(rom)),
            mode: Mode::Normal,
            stepping: false,
            breakpoints,
            prev: 0,
        }
    }

    fn update_keys(&mut self, input: &WinitInputHelper) {
        if input.key_pressed(VirtualKeyCode::F1) {
            let path = Path::new("vram.bin");
            let mut file = File::create(path).unwrap();
            file.write_all(&self.gameboy.mmu.ppu.vram).unwrap();
        }
        else if input.key_pressed(VirtualKeyCode::F2) {
            self.mode = match self.mode {
                Mode::Normal => Mode::TileDump,
                Mode::TileDump => Mode::Normal,
            };
        }
        else if input.key_pressed(VirtualKeyCode::F4) {
            self.stepping = true;
        }
    
        let buttons = [
            VirtualKeyCode::Right, VirtualKeyCode::Left, VirtualKeyCode::Up, VirtualKeyCode::Down,
            VirtualKeyCode::X, VirtualKeyCode::Z, VirtualKeyCode::RShift, VirtualKeyCode::Return,
        ];
        let mut gb_buttons = 0xFF;
    
        for (i, button) in buttons.iter().enumerate() {
            if input.key_held(*button) {
                gb_buttons &= !(1 << i);
            }
        }

        self.gameboy.mmu.joypad.update_state(Buttons::from_bits(gb_buttons).unwrap());
    }
    
    fn update_debug(&mut self) -> u8 {
        // are we stepping or at a breakpoint?
        // if self.gameboy.cpu.regs.pc == 0x2941 {
        //     let de = (self.gameboy.cpu.regs.d as u16) << 8 | self.gameboy.cpu.regs.e as u16;
        //     let copy_len = self.gameboy.mmu.read_memory(de - 1);
        //     let copy_dest = (self.gameboy.mmu.read_memory(de - 3) as u16) << 8 | self.gameboy.mmu.read_memory(de - 2) as u16;
            
        //     if copy_dest < 0x9C00 {
        //         println!("len of copy: {:02X}", copy_len);
        //         println!("copy dest: {:04X}", copy_dest);
        //         let mut copied = String::new();
        //         for i in 0..copy_len {
        //             copied += format!("{:02X} ", self.gameboy.mmu.read_memory(de + i as u16)).as_str();
        //         }
        //         println!("copied data: {copied}")
        //     }
        // }

        // if self.gameboy.cpu.regs.pc == 0x2444 {
        //     let bc = (self.gameboy.cpu.regs.b as u16) << 8 | self.gameboy.cpu.regs.c as u16;
        //     let hl = (self.gameboy.cpu.regs.h as u16) << 8 | self.gameboy.cpu.regs.l as u16;
        //     let tile = self.gameboy.mmu.read_memory(hl);
        //     println!("Copied {:02X} from {:04X} to {:04X}, LY: {}", tile, hl, bc, self.gameboy.mmu.ppu.line_y);
            
        //     if self.gameboy.mmu.ppu.status & 0x3 == 3 {
        //         println!("mode 3 op, hl = {hl:04X}");
        //         self.stepping = true;
        //     }
        // }

        if !(self.stepping || self.breakpoints.contains(&self.gameboy.cpu.regs.pc)) {
            self.prev = self.gameboy.cpu.regs.pc;
            return self.gameboy.run_instruction();
        }

        println!("{:04X}: {}", self.gameboy.cpu.regs.pc, disasm(self.gameboy.cpu.regs.pc, &self.gameboy.mmu));
        loop {
            let mut command = String::new();
            std::io::stdin().read_line(&mut command).unwrap();
            let command: Vec<_> = command.trim().split(" ").collect();

            match command[0] {
                "c" => {
                    self.stepping = false;
                    break;
                },
                "s" => {
                    self.stepping = true;
                    break;
                }
                "b" => {
                    self.breakpoints.insert(u16::from_str_radix(command[1], 16).unwrap());
                }
                "rb" => {
                    let offset = u16::from_str_radix(command[1], 16).unwrap();
                    self.breakpoints.retain(|x| *x != offset);
                }
                "r" => {
                    self.gameboy.cpu.dump_regs();
                }
                "q" => {
                    std::process::exit(0);
                }
                "ch" => {
                    for i in 0..4 {
                        print!("{:02X}", self.gameboy.mmu.read_memory(0xFF80 + i));
                    }
                    println!();
                }
                "prev" => {
                    println!("{:04X}: {}", self.prev, disasm(self.prev, &self.gameboy.mmu));
                }
                "ppu" => {
                    let stat = self.gameboy.mmu.ppu.status;
                    println!("LCDC: {:02X}", self.gameboy.mmu.ppu.lcdc);
                    println!("STAT: {:02X}", stat);
                    println!("LY: {}", self.gameboy.mmu.ppu.line_y);
                    println!("LYC: {}", self.gameboy.mmu.ppu.line_compare);
                    println!("line cycles: {}", self.gameboy.mmu.ppu.cycles_line);
                    println!("BG pal: {:02X}", self.gameboy.mmu.ppu.dmg_palettes.bg_palette);
                    println!("OBJ0 pal: {:02X}", self.gameboy.mmu.ppu.dmg_palettes.obj0_palette);
                    println!("OBJ1 pal: {:02X}", self.gameboy.mmu.ppu.dmg_palettes.obj1_palette);

                    let mode = match stat & 0x3 {
                        0 => "H-Blank",
                        1 => "V-Blank",
                        2 => "OAM Scan",
                        3 => "Drawing",
                        _ => panic!("impossible"),
                    };

                    println!("Mode: {mode} ({})", stat & 0x3);
                    self.gameboy.mmu.ppu.dump_regs();
                }
                "timer" => {
                    self.gameboy.mmu.timer.debug();
                }
                "vram" => {
                    let path = Path::new("vram.bin");
                    let mut file = File::create(path).unwrap();
                    file.write_all(&self.gameboy.mmu.ppu.vram[..6144]).unwrap();
                }
                _ => println!("Not a valid command"),
            }
        }
        
        self.prev = self.gameboy.cpu.regs.pc;
        self.gameboy.run_instruction()
    }

    fn update(&mut self, input: &WinitInputHelper) {
        self.update_keys(input);

        let mut total_cycles = 0;

        while total_cycles < viennetta_gb::hardware::CYCLES_PER_FRAME && !self.stepping {
            let cycles = self.update_debug() as u32;
            if self.gameboy.cpu.double_speed {
                total_cycles += cycles * 2;
            }
            else {
                total_cycles += cycles * 4;           
            }
        }

        if self.stepping {
            self.update_debug();
        }
    }

    fn draw(&mut self, frame: &mut [u8]) {
        let screen = match self.mode {
            Mode::Normal => self.gameboy.mmu.get_frame(),
            Mode::TileDump => self.gameboy.mmu.ppu.dump_tiles(),
        };

        frame.copy_from_slice(&convert_gameboy_to_rgb565(screen));
    }
}

fn col_conv(c: u16) -> u8 {
    ((c << 3) | (c >> 2)) as u8
}

pub fn convert_gameboy_to_rgb565(gameboy: LcdPixels) -> [u8; WIDTH * HEIGHT * PIXEL_SIZE] {
    let mut result = [0; WIDTH * HEIGHT * PIXEL_SIZE];

    for (i, pixel) in gameboy.iter().enumerate() {
        let r = (pixel >> 10) & 0x1F;
        let g = (pixel >> 5) & 0x1F;
        let b = pixel & 0x1F;
        
        result[i * PIXEL_SIZE] = col_conv(b); // truncates
        result[i * PIXEL_SIZE + 1] = col_conv(g);
        result[i * PIXEL_SIZE + 2] = col_conv(r);
        result[i * PIXEL_SIZE + 3] = 0xFF; // alpha channel
    }

    result
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Vienetta")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    dbg!(pixels.surface_texture_format());
    let args: Vec<String> = env::args().collect();
    let rom = fs::read(&args[1]).expect(format!("{} is not a valid path\n", args[1]).as_str());
    let mut world = State::new(&rom);

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            world.update(&input);
            window.request_redraw();
        }
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
