use super::Interrupts;
use bitflags::bitflags;
use dbg_hex::dbg_hex;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
pub type LcdPixels = [u16; WIDTH * HEIGHT];

const DRAW_START: u16 = 80;
const HBLANK_START: u16 = 252;
const LINE_LEN: u16 = 456;
const VBLANK_START: u8 = 144;
const VBLANK_LEN: u8 = 10;
const FRAME_SCANLINES: u8 = VBLANK_START + VBLANK_LEN;
const DMG_COLOURS: [u16; 4] = [0x7FFF, 0x5AB9, 0x35A5, 0x0000];

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
enum DMGPalette {
    #[default] Background,
    Sprite0,
    Sprite1,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct DMGPalettes {
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
    dmg_palette: DMGPalette,
    bank: bool,
    cgb_pal: u8,
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
            dmg_palette: match (flags >> 4) & 1 {
                0 => DMGPalette::Sprite0,
                1 => DMGPalette::Sprite1,
                _ => panic!("not possible"),
            },
            bank: bytes & 0x08 == 0x08,
            cgb_pal: bytes as u8 & 0x7,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct TileAttrib {
    priority: bool,
    x_flip: bool,
    y_flip: bool,
    bank: bool,
    cgb_palette: u8,
}

impl TileAttrib {
    fn from(byte: u8) -> Self {
        Self {
            priority: byte & 0x80 == 0x80,
            y_flip: byte & 0x40 == 0x40,
            x_flip: byte & 0x20 == 0x20,
            bank: byte & 0x08 == 0x08,
            //bank: false,
            cgb_palette: byte & 0x07, 
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
    vram: [u8; 0x4000],
    vram_bank: u8,
    oam: [u8; 0x100],
    pub lcdc: LCDC,
    pub line_compare: u8,
    pub status: u8,
    stat_flag: bool,
    scroll_x: u8,
    scroll_y: u8,
    win_x: u8,
    win_y: u8,
    pub dmg_palettes: DMGPalettes,
    cgb_bg_pals: [u16; 32],
    cgb_obj_pals: [u16; 32],
    sprite_buffer: Vec<Object>,
    pub debug: bool,
    scheduled_stat_update: bool,
    window_triggered: bool,
    win_line_counter: u8,
    is_cgb: bool,
    bgpi: u8,
    obpi: u8,
}

impl Default for PPU {
    fn default() -> Self {
        Self {
            mode: Mode::OAMScan,
            line_y: 0,
            line_x: 0,
            cycles_line: 0,               // So that first line has OAM scan
            lcd: [0x0; WIDTH * HEIGHT],
            vram: [0; 0x4000],
            vram_bank: 0,
            oam: [0; 0x100],
            lcdc: LCDC::empty(),
            line_compare: 0,
            status: 0,
            stat_flag: false,
            scroll_x: 0,
            scroll_y: 0,
            win_x: 0,
            win_y: 0,
            dmg_palettes: DMGPalettes::default(),
            sprite_buffer: vec![],
            debug: false,
            scheduled_stat_update: false,
            window_triggered: false,
            win_line_counter: 0,
            is_cgb: true,
            cgb_bg_pals: [0; 32],
            cgb_obj_pals: [0; 32],
            bgpi: 0,
            obpi: 0,
        }
    }
}

impl PPU {
    pub fn get_frame(&self) -> LcdPixels {
        self.lcd
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        if self.mode != Mode::Drawing {
            self.vram[(address + 0x2000 * self.vram_bank as u16) as usize]
        }
        else {
            0xFF
        }
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        if self.mode != Mode::Drawing {
            self.vram[(address + 0x2000 * self.vram_bank as u16) as usize] = value;
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
        if self.mode == Mode::HBlank || self.mode == Mode::VBlank {
            self.oam[address as usize] = value;
        }
    }
    
    pub fn dump_regs(&self) {
        println!("BGPI: {:02X}", self.bgpi);
        println!("OBPI: {:02X}", self.obpi);

        for i in 0..8 {
            println!("BG{}: {:04X} {:04X} {:04X} {:04X}", i, self.cgb_bg_pals[i * 4], self.cgb_bg_pals[i * 4 + 1], self.cgb_bg_pals[i * 4 + 2], self.cgb_bg_pals[i * 4 + 3]);
        }
        
        for i in 0..8 {
            println!("OBJ{}: {:04X} {:04X} {:04X} {:04X}", i, self.cgb_obj_pals[i * 4], self.cgb_obj_pals[i * 4 + 1], self.cgb_obj_pals[i * 4 + 2], self.cgb_obj_pals[i * 4 + 3]);
        }
    }

    pub fn read_io(&self, address: u16) -> u8 {
        match address {
            0xFF40 => self.lcdc.bits(),
            0xFF41 => self.status | 0x80,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => if self.line_y == 153 { 0 } else { self.line_y },
            0xFF45 => self.line_compare,
            0xFF47 => self.dmg_palettes.bg_palette,
            0xFF48 => self.dmg_palettes.obj0_palette,
            0xFF49 => self.dmg_palettes.obj1_palette,
            0xFF4A => self.win_y,
            0xFF4B => self.win_x,
            0xFF4F => self.vram_bank | 0xFE,
            0xFF68 => self.bgpi,
            0xFF69 => Self::read_io_pal(&self.cgb_bg_pals, self.bgpi as usize),
            0xFF6A => self.obpi,
            0xFF6B => Self::read_io_pal(&self.cgb_obj_pals, self.obpi as usize),
            0xFF6C => if self.is_cgb { 0 } else { 1 },
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
            0xFF47 => self.dmg_palettes.bg_palette = value,
            0xFF48 => self.dmg_palettes.obj0_palette = value,
            0xFF49 => self.dmg_palettes.obj1_palette = value,
            0xFF4A => self.win_y = value,
            0xFF4B => self.win_x = value,
            0xFF4F => self.vram_bank = value & 1,
            0xFF68 => self.bgpi = value,
            0xFF69 => Self::write_io_pal(&mut self.cgb_bg_pals, &mut self.bgpi, value, true),
            0xFF6A => self.obpi = value,
            0xFF6B => Self::write_io_pal(&mut self.cgb_obj_pals, &mut self.obpi, value, false),
            0xFF6C => self.is_cgb = (value & 1) == 0,
            _ => {},
        }
    }

    fn read_io_pal(pals: &[u16], index: usize) -> u8 {
        let shift = if index % 2 == 0 { 0 } else { 8 };
        ((pals[(index >> 1) & 0x1F] >> shift) & 0xFF) as u8
    }


    // obj pals 6 and 7 not written to???
    fn write_io_pal(pals: &mut [u16], index: &mut u8, new_val: u8, bg: bool) { 
        let old_val = &mut pals[((*index >> 1) & 0x1F) as usize];

        if *index % 2 == 0 {
            *old_val &= 0xFF00;
            *old_val |= new_val as u16;
        }
        else {
            *old_val &= 0x00FF;
            *old_val |= (new_val as u16) << 8;
        }

        if *index & 0x80 == 0x80 { // autio increment
            *index += 1;
            if (*index & 0x3F) == 64 {
                *index = 0;
            }
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
            self.lcd = [0x0; WIDTH * HEIGHT];
            self.line_y = 0;
            self.line_x = 0;
            self.status &= 0xFC;
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
        let mut pixels = [(0, DMGPalette::Background); WIDTH];
        let mut window_occured = false;
        let mut priority = [true; WIDTH];

        for tile_num in 0..(WIDTH / 8) + 1 {
            let window = self.line_x + 7 >= self.win_x && self.line_y >= self.win_y && self.lcdc.contains(LCDC::WinEnable);

            let fetcher_x;
            let mut fetcher_y;
            let tilemap;
            
            if window {
                window_occured = true;
                fetcher_x = (((self.line_x - self.win_x + 7) / 8) & 0x1F) as usize;
                fetcher_y = (self.win_line_counter) as usize;
                tilemap = self.lcdc.contains(LCDC::WinTileMap);
            }
            else {
                fetcher_x = (((self.scroll_x / 8) + (tile_num as u8)) & 0x1F) as usize;
                fetcher_y = self.line_y.wrapping_add(self.scroll_y) as usize;
                tilemap = self.lcdc.contains(LCDC::BgTileMap);
            }
            let tile_data_area = self.lcdc.contains(LCDC::BgTileData);
            
            let tile = self.fetch_tile(fetcher_x, fetcher_y, tilemap, !window);
            let attrib = self.fetch_tile_attrib(fetcher_x, fetcher_y, tilemap);

            
            if attrib.y_flip {
                fetcher_y = 7 - fetcher_y;
            }
            
            let fetcher_offset = (fetcher_y % 8) * 2;
            let tile = self.fetch_tile_data(tile, fetcher_offset, tile_data_area, attrib.bank);

            let scroll_discard = self.scroll_x & 0x7;

            for i in 0..8 {
                if tile_num == 0 && i < scroll_discard && !window_occured {
                    continue;
                }

                let i = if attrib.x_flip { i } else { 7 - i };
                let mut pixel = ((tile.1 >> i) & 1) << 1 | ((tile.0 >> i) & 1);
                if !self.lcdc.contains(LCDC::BgWinEnable) && !self.is_cgb {
                    pixel = 0;
                }
                
                if self.line_x >= WIDTH as u8 {
                    break;
                }
                
                priority[self.line_x as usize] = attrib.priority;
                pixels[self.line_x as usize] = (pixel, DMGPalette::Background);


                if self.is_cgb {
                    let colour = self.cgb_bg_pals[(attrib.cgb_palette * 4 + pixel) as usize];
                    //let colour = self.cgb_bg_pals[(pixel) as usize];
                    //if tile_index == 11 {
                    // if attrib.unwrap().priority {
                    //   self.lcd[self.line_x as usize + self.line_y as usize * WIDTH] = 0x7888;
                    // }
                    // else {

                        self.lcd[self.line_x as usize + self.line_y as usize * WIDTH] = colour;
                    //}
                    //}
                    //self.lcd[self.line_x as usize + self.line_y as usize * WIDTH] = colour;
                    //self.lcd[self.line_x as usize + self.line_y as usize * WIDTH] = tile_index as u16 * 0x111;
                    //self.lcd[self.line_x as usize + self.line_y as usize * WIDTH] = attrib.unwrap().cgb_palette as u16 * 0x111;
                }

                self.line_x += 1;

                if self.line_x >= WIDTH as u8 {
                    break;
                }
            }
        }

        if self.lcdc.contains(LCDC::ObjEnable) {
            for obj in &self.sprite_buffer {
                let mut obj_y = (self.line_y + 16) - obj.y;
                if obj.y_flip {
                    if self.lcdc.contains(LCDC::ObjSize) {
                        obj_y = 15 - obj_y;
                    }
                    else {
                        obj_y = 7 - obj_y;
                    }
                }
                let fetcher_offset = (obj_y % 8) * 2;

                let mut tile = obj.tile as usize;
                if self.lcdc.contains(LCDC::ObjSize) {
                    if obj_y > 7 {
                        tile |= 0x01;
                    }
                    else {
                        tile &= 0xFE;
                    }
                }
                let tile_index = tile;
                let tile = self.fetch_tile_data(tile, fetcher_offset as usize, true, obj.bank);
                
                for offset in 0..8 { 
                    let i = if obj.x_flip { offset } else { 7 - offset };
                    let pixel = ((tile.1 >> i) & 1) << 1 | ((tile.0 >> i) & 1);
                    let offset = obj.x as usize + offset as usize - 8;
                    
                    if offset < WIDTH && pixel != 0 {
                        if self.is_cgb {
                            if pixels[offset].0 == 0 || !self.lcdc.contains(LCDC::BgWinEnable) || (!obj.priority && !priority[offset]) {
                            //if attrib.unwrap().priority {
                                let colour = self.cgb_obj_pals[(obj.cgb_pal * 4 + pixel) as usize];
                                //let colour = 0x7EEE;
                                self.lcd[offset + self.line_y as usize * WIDTH] = colour;
                            }
                        }
                        else {
                            if !obj.priority || pixels[offset].0 == 0 {
                                pixels[offset] = (pixel, obj.dmg_palette);
                            }
                        }
                    }
                }
            }
        }

        if window_occured {
            self.win_line_counter += 1;
        }

        for (i, pixel) in pixels.iter().enumerate() {
            // Highlighting code
            // if (pixel.0 & 0xF0) == 0xF0 {
            //     let colour = 3 + pixel.0 & 0xF;
            //     self.lcd[i + self.line_y as usize * WIDTH] = ((colour as u16) << 12) | 0x800;
            //     continue;
            // }


            if self.is_cgb {

            }
            else {
                let palette = match pixel.1 {
                    DMGPalette::Background => self.dmg_palettes.bg_palette,
                    DMGPalette::Sprite0 => self.dmg_palettes.obj0_palette,
                    DMGPalette::Sprite1 => self.dmg_palettes.obj1_palette,
                };
                let colour = (palette >> (2 * pixel.0)) & 0x3;
                self.lcd[i + self.line_y as usize * WIDTH] = DMG_COLOURS[colour as usize];
            }
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
        if !self.is_cgb {
            objects.sort_by_key(|obj| obj.x);
        }
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


    fn fetch_tile(&self, fetcher_x: usize, fetcher_y: usize, tilemap: bool, _is_bg: bool) -> usize {
        let tilemap_index = if tilemap { 0x1C00 } else { 0x1800 };

        // if self.is_cgb && self.fetch_tile_attrib(fetcher_x, fetcher_y, tilemap).bank {
        //     tilemap_index += 0x2000;
        // }

        self.vram[tilemap_index + (fetcher_y / 8) * 32 + fetcher_x] as usize
    }

    fn get_tile_fetch_index(&self, tile_index: usize, tile_offset: usize, tile_data_area: bool, bank: bool) -> usize {
        let mut tile = tile_index * 16 + tile_offset;
        if bank {
            tile += 0x2000;
        }
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

    fn fetch_tile_data(&self, tile_index: usize, tile_offset: usize, tile_data_area: bool, bank: bool) -> (u8, u8) {
        let index = self.get_tile_fetch_index(tile_index, tile_offset, tile_data_area, bank);
        (self.vram[index], self.vram[index + 1])
    }

    fn fetch_tile_attrib(&self, fetcher_x: usize, fetcher_y: usize, tilemap: bool) -> TileAttrib {
        let tilemap = if tilemap { 0x1C00 } else { 0x1800 };
        TileAttrib::from(self.vram[0x2000 + tilemap + (fetcher_y / 8) * 32 + fetcher_x])
    }
}