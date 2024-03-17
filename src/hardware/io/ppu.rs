use super::Interrupts;
use bitflags::bitflags;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
pub type LcdPixels = [u16; WIDTH * HEIGHT];

const OAM_START: u16 = 0;
const DRAW_START: u16 = 80;
const HBLANK_START: u16 = DRAW_START + WIDTH as u16 * 4;
const LINE_LEN: u16 = 456;
const VBLANK_START: u8 = 144;
const VBLANK_LEN: u8 = 10;
const FRAME_SCANLINES: u8 = VBLANK_START + VBLANK_LEN;

const COLOURS: [u16; 4] = [0xFFFF, 0xB573, 0x6B4B, 0x0000];

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct LCDC: u8 {
        const PpuEnable = 1 << 7;
        const WinTileMap = 1 << 6;
        const WinEnable = 1 << 5;
        const BgTileData = 1 << 4;
        const BgTileMap = 1 << 3;
        const ObjSize = 1 << 2;
        const ObjEnable = 1 << 1;
        const BgWinEnable = 1 << 0;
    }
}

#[derive(Debug)]
enum StatReg {
    LycLy = 1 << 2,
    HBlankInt = 1 << 3,
    VBlankInt = 1 << 4,
    OamInt = 1 << 5,
    LycInt = 1 << 6,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OAMScan = 2,
    Drawing = 3,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Colour {
    White = 0,
    LightGrey = 1,
    DarkGrey = 2,
    Black = 3,
}

impl From<u8> for Colour {
    fn from(value: u8) -> Self {
        match value {
            0 => Colour::White,
            1 => Colour::LightGrey,
            2 => Colour::DarkGrey,
            3 => Colour::Black,
            _ => panic!("Invalid u8 value for Colour: {value}"),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum Palette {
    #[default] Background,
    Sprite0,
    Sprite1,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Palettes {
    pub bg_palette: u8,
    pub obj0_palette: u8,
    pub obj1_palette: u8,
}

#[derive(Default, Debug, Clone, Copy)]
struct Object {
    x: u8,
    y: u8,
    tile: u8,
    priority: bool,
    x_flip: bool,
    y_flip: bool,
    palette: Palette,
}

impl Object {
    fn from_bytes(bytes: u32) -> Self {
        let flags = bytes & 0xFF;
        Self {
            x: ((bytes >> 16) & 0xFF) as u8,
            y: (bytes >> 24) as u8,
            tile: ((bytes >> 8) & 0xFF) as u8,
            priority: (flags >> 7) & 1 == 1,
            y_flip: (flags >> 6) & 1 == 1,
            x_flip: (flags >> 5) & 1 == 1,
            palette: match (flags >> 4) & 1 {
                0 => Palette::Sprite0,
                1 => Palette::Sprite1,
                _ => panic!("not possible"),
            },
        }
    }
}

#[derive(Debug)]
pub struct PPU {
    lcd: LcdPixels,
    mode: Mode,
    pub line_y: u8,
    pub line_x: u8,
    pub cycles_line: u16,
    vram: [u8; 0x2000],
    oam: [u8; 0x100],
    pub lcdc: LCDC,
    pub line_compare: u8,
    pub status: u8,
    stat_flag: bool,
    scroll_x: u8,
    scroll_y: u8,
    win_x: u8,
    win_y: u8,
    pub palettes: Palettes,
    sprite_buffer: Vec<Object>,
    pub debug: bool,
    scheduled_stat_update: bool,
    window_triggered: bool,
    win_line_counter: u8,
}

impl Default for PPU {
    fn default() -> Self {
        Self {
            mode: Mode::OAMScan,
            line_y: FRAME_SCANLINES - 1,
            line_x: 0,
            cycles_line: 455,               // So that first line has OAM scan
            lcd: [0xFFFF; WIDTH * HEIGHT],
            vram: [0; 0x2000],
            oam: [0; 0x100],
            lcdc: LCDC::empty(),
            line_compare: 0,
            status: 0,
            stat_flag: false,
            scroll_x: 0,
            scroll_y: 0,
            win_x: 0,
            win_y: 0,
            palettes: Palettes::default(),
            sprite_buffer: vec![],
            debug: false,
            scheduled_stat_update: false,
            window_triggered: false,
            win_line_counter: 0,
        }
    }
}

impl PPU {
    pub fn get_frame(&self) -> LcdPixels {
        self.lcd
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        if self.mode != Mode::Drawing {
            self.vram[address as usize]
        }
        else {
            0xFF
        }
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        // INVESTIGATE
        // COMMENTED OUT FOR DR. MARIO TO WORK
        if self.mode != Mode::Drawing {
            self.vram[address as usize] = value;
        }
    }

    pub fn read_oam(&self, address: u16) -> u8 {
        if self.mode == Mode::HBlank || self.mode == Mode::VBlank {
            self.oam[address as usize]
        }
        else {
            0xFF
        }
    }

    pub fn write_oam(&mut self, address: u16, value: u8) {
        // INVESTIGATE
        // COMMENTED OUT FOR TETRIS TO WORK
        if self.mode == Mode::HBlank || self.mode == Mode::VBlank {
            self.oam[address as usize] = value;
        }
    }

    pub fn read_io(&self, address: u16) -> u8 {
        match address {
            0xFF40 => self.lcdc.bits(),
            0xFF41 => self.status,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.line_y,
            0xFF45 => self.line_compare,
            0xFF47 => self.palettes.bg_palette,
            0xFF48 => self.palettes.obj0_palette,
            0xFF49 => self.palettes.obj1_palette,
            0xFF4A => self.win_y,
            0xFF4B => self.win_x,
            _ => 0,
        }
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => self.lcdc = LCDC::from_bits(value).unwrap(),
            0xFF41 => { self.scheduled_stat_update = true; self.status = (value & 0xFC) | 0x80 },
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF45 => self.line_compare = value,
            0xFF47 => self.palettes.bg_palette = value,
            0xFF48 => self.palettes.obj0_palette = value,
            0xFF49 => self.palettes.obj1_palette = value,
            0xFF4A => self.win_y = value,
            0xFF4B => self.win_x = value,
            _ => {},
        }
    }

    pub fn run_cycles(&mut self, cycles: u8) -> Interrupts {
        let mut interrupts = Interrupts::empty();

        if self.lcdc.contains(LCDC::PpuEnable) {
            for _ in 0..cycles {
                interrupts |= self.run_cycle();
            }
        }
        else {
            self.lcd = [0xFFFF; WIDTH * HEIGHT];
        }

        interrupts
    }

    fn run_cycle(&mut self) -> Interrupts {
        let mut interrupts = Interrupts::empty();
        interrupts |= self.update_mode();
        interrupts |= self.update_stat();
        
        interrupts
    }
                                                                                       
    fn update_lcd(&mut self) {
        let mut pixels = [(0, Palette::Background); WIDTH];
        let mut window_occured = false;

        for _ in 0..WIDTH / 8 {
            let window = self.line_x + 7 >= self.win_x && self.line_y >= self.win_y && self.lcdc.contains(LCDC::WinEnable);

            let fetcher_x;
            let fetcher_y;
            let tilemap;
            
            if window {
                window_occured = true;
                fetcher_x = (((self.line_x - self.win_x + 7) / 8) & 0x1F) as usize;
                fetcher_y = (self.win_line_counter) as usize;
                tilemap = self.lcdc.contains(LCDC::WinTileMap);
            }
            else {
                fetcher_x = (((self.scroll_x / 8) + (self.line_x / 8)) & 0x1F) as usize;
                fetcher_y = self.line_y.wrapping_add(self.scroll_y) as usize;
                tilemap = self.lcdc.contains(LCDC::BgTileMap);
            }
            let fetcher_offset = (fetcher_y % 8) * 2;
            let tile_data_area = self.lcdc.contains(LCDC::BgTileData);

            let tile = self.fetch_tile(fetcher_x, fetcher_y, tilemap);
            let tile = self.fetch_tile_data(tile, fetcher_offset, tile_data_area);

            for i in 0..8 {
                let i = 7 - i;
                let mut pixel = ((tile.1 >> i) & 1) << 1 | ((tile.0 >> i) & 1);
                if !self.lcdc.contains(LCDC::BgWinEnable) {
                    pixel = 0;
                }

                pixels[self.line_x as usize] = (pixel, Palette::Background);
                self.line_x += 1;
            }
        }

        if self.lcdc.contains(LCDC::ObjEnable) {
            for obj in &self.sprite_buffer {
                let mut obj_y = self.line_y - obj.y;
                if obj.y_flip {
                    obj_y = 7 - obj_y;
                }
                let fetcher_offset = (obj_y % 8) * 2;
                let tile = self.fetch_tile_data(obj.tile as usize, fetcher_offset as usize, true);
                
                for offset in 0..8 { 
                    let i = if obj.x_flip { offset } else { 7 - offset };
                    let pixel = ((tile.1 >> i) & 1) << 1 | ((tile.0 >> i) & 1);
                    let offset = obj.x as usize + offset as usize - 8;

                    if pixel != 0 && (!obj.priority || pixels[offset].0 == 0) {
                        pixels[offset] = (pixel, obj.palette);
                    }
                }
            }
        }

        if window_occured {
            self.win_line_counter += 1;
        }

        for (i, pixel) in pixels.iter().enumerate() {
            // Highlighting code
            if (pixel.0 & 0xF0) == 0xF0 {
                let colour = 3 + pixel.0 & 0xF;
                self.lcd[i + self.line_y as usize * WIDTH] = ((colour as u16) << 12) | 0x800;
                continue;
            }

            let palette = match pixel.1 {
                Palette::Background => self.palettes.bg_palette,
                Palette::Sprite0 => self.palettes.obj0_palette,
                Palette::Sprite1 => self.palettes.obj1_palette,
            };
            let colour = (palette >> (2 * pixel.0)) & 0x3;
            self.lcd[i + self.line_y as usize * WIDTH] = COLOURS[colour as usize];
        }
    }

    fn oam_search(&self) -> Vec<Object> {
        let mut objects = vec![];

        for object in self.oam.chunks_exact(4) {
            let object = (u32::from(object[0]) << 24)
            | (u32::from(object[1]) << 16)
            | (u32::from(object[2]) << 8)
            | u32::from(object[3]);
            let object = Object::from_bytes(object);
            let obj_height = if self.lcdc.contains(LCDC::ObjSize) { 16 } else { 8 };
            
            if self.line_y + 16 >= object.y && self.line_y + 16 < object.y + obj_height {
                objects.push(object);
            }
            if objects.len() == 10 {
                break;
            }
        } 
        objects.sort_by_key(|obj| obj.x);
        objects.reverse();
        objects
    }

    fn update_mode(&mut self) -> Interrupts {
        let mut interrupts = Interrupts::empty();
        
        self.cycles_line += 1;
        if self.cycles_line == DRAW_START && self.mode != Mode::VBlank {
            self.line_x = 0;
            self.mode = Mode::Drawing;
            self.status &= 0xFC;
            self.status |= Mode::Drawing as u8;
            self.update_lcd();
        }
        else if self.cycles_line == HBLANK_START && self.mode != Mode::VBlank {
            self.mode = Mode::HBlank;
            self.status &= 0xFC;
            self.status |= Mode::HBlank as u8;
            self.line_x = 0;
        }
        else if self.cycles_line == LINE_LEN {
            self.line_y += 1;
            self.cycles_line = 0;
            self.line_x = 0;
            
            if self.line_y == FRAME_SCANLINES {
                self.line_y = 0;
                self.mode = Mode::OAMScan;
                self.status &= 0xFC;
                self.status |= Mode::OAMScan as u8;
            }
            else if self.line_y == VBLANK_START {
                self.mode = Mode::VBlank;
                self.status &= 0xFC;
                self.status |= Mode::VBlank as u8;
                interrupts |= Interrupts::VBlank;
                self.win_line_counter = 0;
            }

            if self.mode != Mode::VBlank {
                self.enter_oam_scan();
            }
        }
        interrupts
    }

    fn enter_oam_scan(&mut self) {
        self.mode = Mode::OAMScan;
        self.sprite_buffer = self.oam_search();
        self.status &= 0xFC;
        self.status |= Mode::OAMScan as u8;

        if self.win_y == self.line_y {
            self.window_triggered = true;
        }
    }

    fn update_stat(&mut self) -> Interrupts {
        let old_stat_flag = self.stat_flag;

        let lyc = if self.line_y == self.line_compare {
            self.status |= StatReg::LycLy as u8;
            if self.status & StatReg::LycInt as u8 == StatReg::LycInt as u8 {
                true
            }
            else {
                false
            }
        }
        else {
            self.status &= !(StatReg::LycLy as u8);
            false
        };
        let hblank = self.status & StatReg::HBlankInt as u8 == StatReg::HBlankInt as u8 && self.mode == Mode::HBlank;
        let vblank = self.status & StatReg::VBlankInt as u8 == StatReg::VBlankInt as u8 && self.mode == Mode::VBlank;
        let oam = self.status & StatReg::OamInt as u8 == StatReg::OamInt as u8 && self.mode == Mode::OAMScan;

        self.stat_flag = lyc || hblank || vblank || oam;

        if !old_stat_flag && self.stat_flag {
            Interrupts::LcdStat
        }
        else {
            Interrupts::empty()
        }
    }


    fn fetch_tile(&self, fetcher_x: usize, fetcher_y: usize, tilemap: bool) -> usize {
        let tilemap = if tilemap { 0x1C00 } else { 0x1800 };
        self.vram[tilemap + (fetcher_y / 8) * 32 + fetcher_x] as usize
    }
    
    fn get_tile_fetch_index(&self, tile_index: usize, tile_offset: usize, tile_data_area: bool) -> usize {
        let tile = tile_index * 16 + tile_offset;
        if tile_data_area {
            tile
        }
        else {
            if tile_index > 127 {
                tile
            }
            else {
                0x1000 + tile
            }
        }
    }

    fn fetch_tile_data(&self, tile_index: usize, tile_offset: usize, tile_data_area: bool) -> (u8, u8) {
        let index = self.get_tile_fetch_index(tile_index, tile_offset, tile_data_area);
        (self.vram[index], self.vram[index + 1])
    }
}