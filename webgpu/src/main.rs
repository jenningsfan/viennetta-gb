#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use log::error;
use std::{env, fs};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use viennetta_gb::hardware::{io::{cart::Cartridge, HEIGHT, WIDTH, LcdPixels, joypad::Buttons}, GameBoy};

const PIXEL_SIZE: usize = 4;
const COLOURS: [u32; 4] = [0xFFFFFFFF, 0xFFC0C0C0, 0xFF808080, 0xFF000000];

/// Representation of the application state. In this example, a box will bounce around the screen.
struct State {
    gameboy: GameBoy,
}

pub fn convert_gameboy_to_rgb565(gameboy: LcdPixels) -> [u8; WIDTH * HEIGHT * PIXEL_SIZE] {
    let mut result = [0; WIDTH * HEIGHT * PIXEL_SIZE];

    for (i, pixel) in gameboy.iter().enumerate() {
        let colour = COLOURS[*pixel as usize];
        result[i * PIXEL_SIZE] = (colour & 0xFF) as u8; // truncates
        result[i * PIXEL_SIZE + 1] = ((colour >> 8) & 0xFF) as u8;
        result[i * PIXEL_SIZE + 2] = ((colour >> 16) & 0xFF) as u8;
        result[i * PIXEL_SIZE + 3] = ((colour >> 24) & 0xFF) as u8;
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
            .with_title("Hello Pixels")
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
            world.update();
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
            gameboy: GameBoy::new(Cartridge::new(rom))
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        self.gameboy.mmu.joypad.update_state(Buttons::from_bits(0xFF).unwrap());
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&mut self, frame: &mut [u8]) {
        frame.copy_from_slice(&convert_gameboy_to_rgb565(self.gameboy.run_frame()));
    }
}