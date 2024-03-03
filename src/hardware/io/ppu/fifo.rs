use super::{Palettes, LCDC, Object, Palette};
use derivative::Derivative;

#[derive(Debug, Clone)]
pub struct FifoPixel {
    colour: u8,
    palette: Palette,
    bg_priority: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum FetchState {
    #[default] FetchTile,
    FetchDataLow,
    FetchDataHigh,
    Push,
    Paused,
}

impl FetchState {
    pub fn next(&self) -> Self {
        match *self {
            Self::FetchTile => Self::FetchDataLow,
            Self::FetchDataLow => Self::FetchDataHigh,
            Self::FetchDataHigh => Self::Push,
            Self::Push => Self::Push, // this is not a bug. it will have to manunally change it to fetchtile
            Self::Paused => Self::Paused,
        }
    }
}

#[derive(Default, Debug, Clone)]
struct SpriteFetcher {
    sprite: Object,
    last_tile: usize,
    last_high: u8,
    last_low: u8,
}

impl SpriteFetcher {
    fn fetch_tile(&mut self, sprite: Object) {
        self.last_tile = sprite.tile as usize;
        self.sprite = sprite;
    }

    fn fetch_data_low(&mut self, mut tile_offset: usize, vram: &[u8; 0x2000]) {
        if self.sprite.y_flip {
            tile_offset = 15 - tile_offset;
        }

        let tile = self.last_tile * 16 + tile_offset;
        self.last_low = vram[tile];
    }

    fn fetch_data_high(&mut self, mut tile_offset: usize, vram: &[u8; 0x2000]) {
        if self.sprite.y_flip {
            tile_offset = 15 - tile_offset;
        }

        let tile = self.last_tile * 16 + tile_offset;
        self.last_high = vram[tile + 1];
    }

    fn push_to_fifo(&mut self, fifo: &mut Vec<FifoPixel>) {
        for mut i in 0..8 {
            if self.sprite.x_flip {
                i = 7 - i;
            }
            let colour = ((self.last_low >> i) & 1) << 1 | ((self.last_high >> i) & 1);
            let palette = self.sprite.palette;
            let pixel = FifoPixel {
                colour,
                palette,
                bg_priority: Some(self.sprite.priority),
            };
            fifo.push(pixel);
        }
    }
}

#[derive(Default, Debug, Clone)]
struct BgFetcher {
    last_tile: usize,
    last_high: u8,
    last_low: u8,
}

impl BgFetcher {
    fn fetch_tile(&mut self, fetcher_x: usize, fetcher_y: usize, vram: &[u8; 0x2000], lcdc: LCDC) {
        let tilemap = if lcdc.contains(LCDC::BgTileMap) {
            0x1C00
        }
        else {
            0x1800
        };

        self.last_tile = vram[tilemap + (fetcher_y / 8) * 32 + fetcher_x - 1] as usize;
    }

    fn fetch_data_low(&mut self, tile_offset: usize, vram: &[u8; 0x2000], lcdc: LCDC) {
        let tile = self.last_tile * 16 + tile_offset;
        self.last_low = if lcdc.contains(LCDC::BgTileData) {
            vram[tile]
        }
        else {
            if self.last_tile > 127 {
                vram[tile]
            }
            else {
                vram[0x1000 + tile]
            }
        };
    }

    fn fetch_data_high(&mut self, tile_offset: usize, vram: &[u8; 0x2000], lcdc: LCDC) {
        let tile = self.last_tile * 16 + tile_offset;
        self.last_high = if lcdc.contains(LCDC::BgTileData) {
            vram[tile + 1]
        }
        else {
            if self.last_tile > 127 {
                vram[tile + 1]
            }
            else {
                vram[0x1000 + tile + 1]
            }
        };
    }

    fn push_to_fifo(&mut self, fifo: &mut Vec<FifoPixel>) {
        for i in 0..8 {
            let colour = ((self.last_low >> i) & 1) << 1 | ((self.last_high >> i) & 1);
            let palette = Palette::Background;
            let pixel = FifoPixel {
                colour,
                palette,
                bg_priority: None,
            };
            fifo.push(pixel);
        }
    }
}

#[derive(Debug, Derivative, Clone)]
#[derivative(Default)]
pub struct FIFO {
    scroll_x_left: u8,
    pub x_pos: u8,
    bg_fifo: Vec<FifoPixel>,
    sprite_fifo: Vec<FifoPixel>,
    bg_fetch_state: FetchState,
    #[derivative(Default(value="FetchState::Paused"))]
    sprite_fetch_state: FetchState,
    state_cycles: u8,
    bg_fetcher: BgFetcher,
    sprite_fetcher: SpriteFetcher,
    current_sprite: Option<Object>,
    #[derivative(Default(value="true"))]
    pixel_shifter_enabled: bool,
}

impl FIFO {
    pub fn sprite_fetch(&mut self, sprite: Object) {
        self.bg_fetch_state = FetchState::Paused;
        self.sprite_fetch_state = FetchState::FetchTile;
        self.current_sprite = Some(sprite);
        self.pixel_shifter_enabled = false;
        self.state_cycles = 0;
        self.sprite_fifo = vec![];
    }

    pub fn run_cycle(&mut self, scroll_x: u8, scroll_y: u8, line_y: u8, vram: &[u8; 0x2000], lcdc: LCDC, palettes: Palettes) -> Option<u8> {
        let fetcher_x = (((scroll_x / 8) + self.x_pos) & 0x1F) as usize;
        let fetcher_y = line_y.wrapping_add(scroll_y) as usize;

        match self.bg_fetch_state {
            FetchState::FetchTile => self.bg_fetcher.fetch_tile(fetcher_x, fetcher_y, vram, lcdc),
            FetchState::FetchDataLow => self.bg_fetcher.fetch_data_low((fetcher_y % 8) * 2, vram, lcdc),
            FetchState::FetchDataHigh => self.bg_fetcher.fetch_data_high((fetcher_y % 8) * 2, vram, lcdc),
            FetchState::Push => {
                if self.bg_fifo.len() == 0 || self.bg_fifo.len() == 8 {
                    self.bg_fetcher.push_to_fifo(&mut self.bg_fifo);
                    self.bg_fetch_state = FetchState::FetchTile;
                    self.x_pos += 1;
                }
            },
            FetchState::Paused => {},
        };

        match self.sprite_fetch_state {
            FetchState::FetchTile => self.sprite_fetcher.fetch_tile(self.current_sprite.unwrap()),
            FetchState::FetchDataLow => self.sprite_fetcher.fetch_data_low((fetcher_y % 8) * 2, vram),
            FetchState::FetchDataHigh => self.sprite_fetcher.fetch_data_high((fetcher_y % 8) * 2, vram),
            FetchState::Push => {
                self.sprite_fetcher.push_to_fifo(&mut self.sprite_fifo);
                self.bg_fetch_state = FetchState::FetchTile;
                self.sprite_fetch_state = FetchState::Paused;
                self.current_sprite = None;
                self.pixel_shifter_enabled = true;
                self.state_cycles = 0;
            },
            FetchState::Paused => {},
        };

        self.state_cycles += 1;

        if self.state_cycles == 2 {
            self.state_cycles = 0;
            self.bg_fetch_state = self.bg_fetch_state.next();
            self.sprite_fetch_state = self.sprite_fetch_state.next();
        }
        
        if self.bg_fifo.len() > 8 && self.pixel_shifter_enabled {
            let mut colour = (palettes.bg_palette >> (2 * self.bg_fifo.pop().unwrap().colour)) & 0x3;
            
            if !lcdc.contains(LCDC::BgWinEnable) {
                //println!("bg reset");
                colour = 0;
            }
            
            if self.sprite_fifo.len() > 0 && lcdc.contains(LCDC::ObjEnable) {
                let sprite = self.sprite_fifo.pop().unwrap();
                let sprite_palette = match sprite.palette {
                    Palette::Sprite0 => palettes.obj0_palette,
                    Palette::Sprite1 => palettes.obj1_palette,
                    Palette::Background => panic!("Sprites can't have bg palette"),
                };

                let priority = sprite.bg_priority.unwrap();
                let sprite_colour = (sprite_palette >> (2 * sprite.colour)) & 0x3;

                // if sprite colour == 0 then push bg pixel
                // if bg-obj priority is 1 and bg is not 0 then push bg
                // else push sprite

                if sprite.colour != 0 && !(priority && colour != 0) {
                    colour = sprite_colour;
                }
            }
            Some(colour)
        }
        else {
            None
        }
    }
}