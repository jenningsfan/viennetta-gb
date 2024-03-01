use super::{Palettes, LCDC, Object};
use derivative::Derivative;

#[derive(Debug, Clone)]
enum Palette {
    Background,
    Sprite1,
    Sprite2,
}

#[derive(Debug, Clone)]
pub struct FifoPixel {
    colour: u8,
    palette: Palette,
    bg_priority: Option<u8>,
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
    last_tile: usize,
    last_high: u8,
    last_low: u8,
}

impl SpriteFetcher {
    fn fetch_tile(&mut self, sprite: Object) {
        self.last_tile = sprite.tile as usize;
    }

    fn fetch_data_low(&mut self, tile_offset: usize, vram: &[u8; 0x2000]) {
        let tile = self.last_tile * 16 + tile_offset;
        self.last_low = vram[tile];
    }

    fn fetch_data_high(&mut self, tile_offset: usize, vram: &[u8; 0x2000]) {
        let tile = self.last_tile * 16 + tile_offset;
        self.last_low = vram[tile + 1];
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
            if tile / 16 > 127 {
                let tile = (!tile as u8 + 1) as usize;
                vram[0x800 + tile]
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
            if tile / 16 > 127 {
                let tile = (!tile as u8 + 1) as usize;
                vram[0x800 + tile + 1]
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
        self.current_sprite = Some(sprite);
        self.pixel_shifter_enabled = false;
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
            },
            FetchState::Paused => {},
        };

        self.state_cycles += 1;

        if self.state_cycles == 2 {
            self.state_cycles = 0;
            self.bg_fetch_state = self.bg_fetch_state.next();
        }
        
        if self.bg_fifo.len() > 8 && self.pixel_shifter_enabled {
            Some((palettes.bg_palette >> (2 * self.bg_fifo.pop().unwrap().colour)) & 0x3)
        }
        else {
            None
        }
    }
}