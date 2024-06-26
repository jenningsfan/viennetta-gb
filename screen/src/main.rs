use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        use std::collections::HashSet;
        use std::{env, fs, thread};
        use std::io::stdin;
        use std::time::{Instant, Duration};
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use device_query::{DeviceState, Keycode};
        use viennetta_gb::hardware::io::joypad::Buttons;

        use viennetta_gb::hardware::{io::{cart::Cartridge, HEIGHT, WIDTH, LcdPixels}, GameBoy};
        use viennetta_gb::hardware::cpu::CPU;

        use std::io::{self, Write};

        use crossterm::{
            ExecutableCommand, execute,
            cursor::{Hide}
        };
    }
}

const PIXEL_SIZE: usize = 2;
const COLOURS: [u32; 4] = [0xFFFFFFFF, 0x706869, 0xaba5a8, 0x0000];

#[cfg(target_os = "linux")]
fn convert_gameboy_to_fb(gameboy: LcdPixels, width: usize, height: usize) -> Vec<u32>
{
    let col_repeat = width / WIDTH;
    let row_repeat = height / HEIGHT;
    let mut pixels = vec![];
    for row in 0..HEIGHT {
        for _ in 0..row_repeat {
            for col in 0..WIDTH {
                for _ in 0..col_repeat {
                    let pixel = gameboy[row * WIDTH + col];
                    pixels.push(COLOURS[pixel as usize]);
                }
            }
        }
    }

    return pixels;
}

#[cfg(target_os = "linux")]
fn update_gb_joypad(gameboy: GameBoy, device_state: DeviceState) {
    let buttons = [
        Keycode::RIGHT, Keycode::LEFT, Keycode::UP, Keycode::DOWN,
        Keycode::A, Keycode::B, Keycode::SELECT, Keycode::START,
    ];
    let keys: Vec<Keycode> = device_state.get_keys();
    let mut gb_buttons = 0xFF;

    for (i, button) in buttons.iter().enumerate() {
        if keys.contains(button) {
            gb_buttons &= !(1 << i);
        }
    }

    gameboy.mmu.joypad.update_state(Buttons::from_bits(gb_buttons).unwrap());
}

#[cfg(target_os = "linux")]
fn main() {
        let args: Vec<String> = env::args().collect();
        let rom = fs::read(&args[1]).expect(format!("{} is not a valid path\n", args[1]).as_str());
        let mut gameboy = GameBoy::new(Cartridge::new(&rom));

    execute!(
        io::stdout(),
        Hide
    );
    let device_state = DeviceState::new();

    let mut fb: linuxfb::Framebuffer = linuxfb::Framebuffer::new("/dev/fb0").unwrap();
    let mut buffer = linuxfb::double::Buffer::new(fb).unwrap();
    println!("Width: {}\nHeight: {}", buffer.width, buffer.height);

    let width = buffer.width as usize;
    let height = buffer.height as usize;

    let frame: &mut[u8] = buffer.as_mut_slice();

    for i in 0..frame.len() {
        frame[i] = 0xFF;
    }
    buffer.flip().unwrap();
    //panic!();
    gameboy.mmu.joypad.update_state(Buttons::from_bits(0xFF).unwrap());

    let mut accumulator = Duration::new(0, 0);
    let target_frame_time = Duration::from_secs_f64(1.0 / 60.0);
    let mut old_time = Instant::now();

    loop {
        let current_time = Instant::now();
        let delta_time = current_time.duration_since(old_time);
        old_time = current_time;
        
        accumulator += delta_time;

        update_gb_joypad(gameboy, device_state);

        let mut gameboy_pixels: LcdPixels = [0; WIDTH * HEIGHT];
        while accumulator >= target_frame_time {
            gameboy_pixels = gameboy.run_frame();
            accumulator -= target_frame_time;
        }

        let frame: &mut[u8] = buffer.as_mut_slice();
        let (prefix, screen_pixels, suffix) = unsafe { frame.align_to_mut::<u32>() };
        assert_eq!(prefix.len(), 0);
        assert_eq!(suffix.len(), 0);
        let converted: &mut[u32] = &mut convert_gameboy_to_fb(gameboy_pixels, width, height);
        //pixels.copy_from_slice(converted.as_mut());

        for i in 0..converted.len()
        {
            screen_pixels[i] = converted[i];
        }
        buffer.flip().unwrap();

        gameboy.mmu.apu.sample_buf = vec![];

        thread::sleep_ms(1000 / 60);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    panic!("only for linux");
}