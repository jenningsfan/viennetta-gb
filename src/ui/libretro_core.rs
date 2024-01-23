use std::ffi::CString;

use rust_libretro::{
    contexts::*, core::Core, env_version, input_descriptors, proc::*, retro_core, sys::*, types::*,
}; // TODO: see which imports are necessary

#[derive(CoreOptions)]
struct ViennettaCore {
    
}

impl Core for ViennettaCore {
    fn get_info(&self) -> SystemInfo {
        SystemInfo {
            library_name: CString::new("Viennetta").unwrap(),
            library_version: CString::new(env_version!("CARGO_PKG_VERSION").to_string()).unwrap(),
            valid_extensions: CString::new("gb").unwrap(),

            need_fullpath: false,
            block_extract: false,
        }
    }

    fn on_get_av_info(&mut self, _ctx: &mut GetAvInfoContext) -> retro_system_av_info {
        retro_system_av_info {
            geometry: retro_game_geometry {
                base_width: crate::ui::io::graphics::PIXEL_SIZE * crate::hardware::io::WIDTH as u32,
                base_height: crate::ui::io::graphics::PIXEL_SIZE * crate::hardware::io::HEIGHT as u32,
                max_width: crate::ui::io::graphics::PIXEL_SIZE * crate::hardware::io::WIDTH as u32,
                max_height: crate::ui::io::graphics::PIXEL_SIZE * crate::hardware::io::HEIGHT as u32,
                aspect_ratio: 0.0,
            },
            timing: retro_system_timing {
                fps: 60.0,
                sample_rate: 0.0, // TODO: CHANGE FOR AUDIO SUPPORT
            },
        }
    }

    fn on_init(&mut self, _ctx: &mut InitContext) {
        
    }

    fn on_set_environment(&mut self, _initial: bool, _ctx: &mut SetEnvironmentContext) {
        
    }

    fn on_load_game(
            &mut self,
            _game: Option<retro_game_info>,
            _ctx: &mut LoadGameContext,
        ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    #[inline]
    fn on_run(&mut self, _ctx: &mut RunContext, _delta_us: Option<i64>) {
        
    }
}

retro_core!(ViennettaCore {

});