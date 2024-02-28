use std::ffi::{c_void, CString};

use rust_libretro::{
    contexts::*, core::Core, env_version, proc::*, retro_core, sys::*, types::*,
}; // TODO: see which imports are necessary

use crate::hardware::{io::{Cartridge, HEIGHT, WIDTH}, GameBoy};
use crate::hardware::io::joypad::Buttons;
use crate::ui::io::graphics::convert_gameboy_to_rgb565;

fn convert_data_to_vec(data: *const c_void, len: usize) -> Vec<u8> {
    // Safety: Ensure that the pointer is valid and doesn't cause UB
    let data_slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };

    // Convert the slice to a Vec<u8>
    data_slice.to_vec()
}

#[derive(CoreOptions)]
struct ViennettaCore {
    gameboy: GameBoy,
}

impl Core for ViennettaCore {
    fn get_info(&self) -> SystemInfo {
        SystemInfo {
            library_name: CString::new("Viennetta").unwrap(),
            library_version: CString::new(env_version!("CARGO_PKG_VERSION").to_string()).unwrap(),
            valid_extensions: CString::new("gb|gbc|txt").unwrap(),

            need_fullpath: false,
            block_extract: false,
        }
    }

    fn on_get_av_info(&mut self, _ctx: &mut GetAvInfoContext) -> retro_system_av_info {
        retro_system_av_info {
            geometry: retro_game_geometry {
                base_width: WIDTH as u32,
                base_height: HEIGHT as u32,
                max_width: WIDTH as u32,
                max_height: HEIGHT as u32,
                aspect_ratio: 0.0,
            },
            timing: retro_system_timing {
                fps: 60.0,
                sample_rate: 0.0, // TODO: CHANGE FOR AUDIO SUPPORT
            },
        }
    }

    fn on_set_environment(&mut self, initial: bool, ctx: &mut SetEnvironmentContext) {
        if !initial {
            return;
        }

        ctx.set_support_no_game(true);
    }

    fn on_load_game(
            &mut self,
            game: Option<retro_game_info>,
            ctx: &mut LoadGameContext,
        ) -> Result<(), Box<dyn std::error::Error>> {
        ctx.set_pixel_format(PixelFormat::RGB565);

        if let Some(game) = game {
            if game.data.is_null() {
                panic!("game.data is NULL");
            }

            let data = convert_data_to_vec(game.data, game.size);
            self.gameboy = GameBoy::new(Cartridge::new(&data));
        }
        Ok(())
    }

    #[inline]
    fn on_run(&mut self, ctx: &mut RunContext, _delta_us: Option<i64>) {
        self.update_gb_joypad(ctx);
        let pixels = convert_gameboy_to_rgb565(self.gameboy.run_frame());
        ctx.draw_frame(&pixels, WIDTH as u32, HEIGHT as u32, WIDTH as usize * 2);
    }
}

impl ViennettaCore {
    fn update_gb_joypad(&mut self, ctx: &mut RunContext) {
        let buttons = [
            JoypadState::A, JoypadState::B, JoypadState::SELECT, JoypadState::START,
            JoypadState::RIGHT, JoypadState::LEFT, JoypadState::UP, JoypadState::DOWN
        ];
        let joypad = ctx.get_joypad_state(0, 0);
        let mut gb_buttons = 0xFF;

        for (i, button) in buttons.iter().enumerate() {
            if joypad.contains(*button) {
                gb_buttons &= !(1 << i);
            }
        }

        self.gameboy.mmu.joypad.update_state(Buttons::from_bits(gb_buttons).unwrap());
    }
}

retro_core!(ViennettaCore {
    gameboy: GameBoy::new(Cartridge::new(&[0; 0x8000])),
});