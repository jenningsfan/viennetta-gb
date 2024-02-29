use super::{Palettes, LCDC, PPU};

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
}

impl FetchState {
    pub fn next(&self) -> Self {
        match *self {
            Self::FetchTile => Self::FetchDataLow,
            Self::FetchDataLow => Self::FetchDataHigh,
            Self::FetchDataHigh => Self::Push,
            Self::Push => Self::Push, // this is not a bug. it will have to manunally change it to fetchtile
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct FIFO {
    scroll_x_left: u8,
    pub x_pos: u8,
    bg_fifo: Vec<FifoPixel>,
    sprite_fifo: Vec<FifoPixel>,
    fetch_state: FetchState,
    state_cycles: u8,
    last_tile: usize,
    last_high: u8,
    last_low: u8,
}

impl FIFO {
    pub fn get_tile_addr(&mut self, fetcher_x: usize, fetcher_y: usize, vram: [u8; 0x2000], lcdc: LCDC) -> usize {
        let tilemap = if lcdc.contains(LCDC::BgTileMap) {
            0x1C00
        }
        else {
            0x1800
        };

        vram[tilemap + (fetcher_y / 8) * 32 + fetcher_x] as usize * 16
    }

    pub fn get_tile_data_low(&mut self, tile: usize, vram: [u8; 0x2000], lcdc: LCDC) -> u8 {
        if lcdc.contains(LCDC::BgTileData) {
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
        }
    }

    pub fn get_tile_data_high(&mut self, tile: usize, vram: [u8; 0x2000], lcdc: LCDC) -> u8 {
        if lcdc.contains(LCDC::BgTileData) {
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
        }
    }

    pub fn run_cycle(&mut self, scroll_x: u8, scroll_y: u8, line_y: u8, vram: [u8; 0x2000], lcdc: LCDC, palettes: Palettes) -> Option<u8> {
        let fetcher_x = (((scroll_x / 8) + self.x_pos) & 0x1F) as usize;
        let fetcher_y = line_y.wrapping_add(scroll_y) as usize;

        match self.fetch_state {
            FetchState::FetchTile => self.last_tile = self.get_tile_addr(fetcher_x, fetcher_y, vram, lcdc),
            FetchState::FetchDataLow => self.last_low = self.get_tile_data_low(self.last_tile + (fetcher_y % 8) * 2, vram, lcdc),
            FetchState::FetchDataHigh => self.last_low = self.get_tile_data_high(self.last_tile + (fetcher_y % 8) * 2, vram, lcdc),
            FetchState::Push => {
                if self.bg_fifo.len() == 0 || self.bg_fifo.len() == 8 {
                    //println!("pushed pixel to fifo");
                    for i in 0..8 {
                        //let i = 7 - i;
                        let colour = ((self.last_low >> i) & 1) << 1 | ((self.last_high >> i) & 1);
                        let palette = Palette::Background;
                        let pixel = FifoPixel {
                            colour,
                            palette,
                            bg_priority: None,
                        };
                        self.bg_fifo.push(pixel);
                    }

                    self.fetch_state = FetchState::FetchTile;
                    self.x_pos += 1;
                }
            }
        }

        self.state_cycles += 1;

        if self.state_cycles == 2 {
            self.state_cycles = 0;
            self.fetch_state = self.fetch_state.next();
        }
        
        if self.bg_fifo.len() > 8 {
            //println!("pushed pixel to display");
            Some((palettes.bg_palette >> (2 * self.bg_fifo.pop().unwrap().colour)) & 0x3)
        }
        else {
            None
        }
    }
}