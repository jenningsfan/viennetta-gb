use std::ffi::CString;

use rust_libretro::{
    contexts::*, core::Core, env_version, input_descriptors, proc::*, retro_core, sys::*, types::*,
}; // TODO: see which imports are necessary
use crate::hardware::io::{WIDTH, HEIGHT};

#[derive(CoreOptions)]
struct ViennettaCore {
    pixels: Vec<u8>,
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

    fn on_init(&mut self, ctx: &mut InitContext) {
        
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
        dbg!("loading");
        dbg!(game);
        Ok(())
    }

    #[inline]
    fn on_run(&mut self, ctx: &mut RunContext, _delta_us: Option<i64>) {
            let color_a = 0xFF;
            let color_b = !color_a;

            for (i, chunk) in self.pixels.chunks_exact_mut(4).enumerate() {
                let x = (i % WIDTH as usize) as f64 / WIDTH as f64;
                let y = (i / WIDTH as usize) as f64 / HEIGHT as f64;

                let total = (50.0f64 * x).floor() + (37.5f64 * y).floor();
                let even = total as usize % 2 == 0;

                let color = if even { color_a } else { color_b };

                chunk.fill(color);
            }

            ctx.draw_frame(self.pixels.as_ref(), WIDTH as u32, HEIGHT as u32, WIDTH as usize * 4);
        
    }
}

retro_core!(ViennettaCore {
     pixels: vec![0; WIDTH * HEIGHT * 4],
});