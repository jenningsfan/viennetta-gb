use std::ffi::CString;

use rust_libretro::{
    contexts::*, core::Core, env_version, proc::*, retro_core, sys::*, types::*,
}; // TODO: see which imports are necessary

use crate::hardware::{GameBoy, io::{WIDTH, HEIGHT}};
use crate::ui::libretro::libretro_utils::convert_data_to_vec;

#[derive(CoreOptions)]
struct ViennettaCore {
    gameboy: GameBoy,
}

impl Core for ViennettaCore {
    fn get_info(&self) -> SystemInfo {
        SystemInfo {
            library_name: CString::new("Viennetta B").unwrap(),
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
            _ctx: &mut LoadGameContext,
        ) -> Result<(), Box<dyn std::error::Error>> {
        
        if let Some(game) = game {
            if game.data.is_null() {
                panic!("game.data is NULL");
            }

            let data = convert_data_to_vec(game.data, game.size);
            self.gameboy = GameBoy::default();
            self.gameboy.load_rom(&data);
        }
        Ok(())
    }

    #[inline]
    fn on_run(&mut self, ctx: &mut RunContext, _delta_us: Option<i64>) {
        self.gameboy.run_frame();
        
        let pixels = [0xFF; WIDTH * HEIGHT * 4];
        ctx.draw_frame(&pixels, WIDTH as u32, HEIGHT as u32, WIDTH as usize * 4);
    
    }
}

retro_core!(ViennettaCore {
    gameboy: GameBoy::default(),
});