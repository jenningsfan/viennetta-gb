#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use log::error;
use pixels::wgpu::ImageCopyBuffer;
use std::{env, fs, path::Path, fs::File};
use std::io::Write;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use viennetta_gb::hardware::{io::{cart::Cartridge, HEIGHT, WIDTH, LcdPixels, joypad::Buttons}, GameBoy};

const PIXEL_SIZE: usize = 4;

enum Mode {
    Normal,
    TileDump,
}

/// Representation of the application state. In this example, a box will bounce around the screen.
struct State {
    pub gameboy: GameBoy,
    pub mode: Mode
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

            if input.key_pressed(VirtualKeyCode::F1) {
                let path = Path::new("vram.bin");
                let mut file = File::create(path).unwrap();
                file.write_all(&world.gameboy.mmu.ppu.vram).unwrap();
            }

            if input.key_pressed(VirtualKeyCode::M) {
                world.mode = match world.mode {
                    Mode::Normal => Mode::TileDump,
                    Mode::TileDump => Mode::Normal,
                };
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

impl State {
    /// Create a new `World` instance that can draw a moving box.
    fn new(rom: &[u8]) -> Self {
        Self {
            gameboy: GameBoy::new(Cartridge::new(rom)),
            mode: Mode::Normal,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self, input: &WinitInputHelper) {
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

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&mut self, frame: &mut [u8]) {
        match self.mode {
            Mode::Normal => {
                frame.copy_from_slice(&convert_gameboy_to_rgb565(self.gameboy.run_frame()))
            },
            Mode::TileDump => {
                self.gameboy.run_frame();
                frame.copy_from_slice(&convert_gameboy_to_rgb565(self.gameboy.mmu.ppu.dump_tiles()));
            }
        };
    }
}