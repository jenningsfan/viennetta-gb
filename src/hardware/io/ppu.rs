mod fifo;

use fifo::*;
use super::Interrupts;
use bitflags::bitflags;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
pub type LcdPixels = [u16; WIDTH * HEIGHT];

const OAM_START: u16 = 0;
const DRAW_START: u16 = 80;
const LINE_LEN: u16 = 456;
const VBLANK_START: u8 = 144;
const VBLANK_LEN: u8 = 10;
const FRAME_SCANLINES: u8 = VBLANK_START + VBLANK_LEN;

const COLOURS: [u16; 4] = [0xFFFF, 0xB573, 0x6B4B, 0x0000];

bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct LCDC: u8 {
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

#[derive(Default, Debug, Clone, Copy)]
struct Palettes {
    bg_palette: u8,
    obj1_palette: u8,
    obj2_palette: u8,
}

#[derive(Default, Debug, Clone, Copy)]
struct Object {
    x: u8,
    y: u8,
    tile: u8,
    priority: bool,
    x_flip: bool,
    y_flip: bool,
    palette: bool,
}

impl Object {
    fn from_bytes(bytes: u32) -> Self {
        let flags = bytes & 0xFF;
        Self {
            x: ((bytes >> 16) & 0xFF) as u8,
            y: (bytes >> 24) as u8,
            tile: ((bytes >> 8) & 0xFF) as u8,
            priority: (flags >> 7) == 1,
            x_flip: (flags >> 6) == 1,
            y_flip: (flags >> 5) == 1,
            palette: (flags >> 4) == 1,
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
    lcdc: LCDC,
    line_compare: u8,
    status: u8,
    stat_flag: bool,
    scroll_x: u8,
    scroll_y: u8,
    win_x: u8,
    win_y: u8,
    palettes: Palettes,
    objects_array: Vec<Object>,
    fifo: FIFO,
    pub debug: bool,
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
            objects_array: vec![],
            fifo: FIFO::default(),
            debug: false,
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
            0xFF48 => self.palettes.obj1_palette,
            0xFF49 => self.palettes.obj2_palette,
            0xFF4A => self.win_y,
            0xFF4B => self.win_x,
            _ => 0,
        }
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => self.lcdc = LCDC::from_bits(value).unwrap(),
            0xFF41 => self.status = value & 0x7C,
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF45 => self.line_compare = value,
            0xFF47 => self.palettes.bg_palette = value,
            0xFF48 => self.palettes.obj1_palette = value,
            0xFF49 => self.palettes.obj2_palette = value,
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
        if self.mode == Mode::Drawing {
            self.update_lcd();
        }
        if self.update_stat() {
            interrupts |= Interrupts::LcdStat;
        }
        interrupts
    }
                                                                                       
    fn update_lcd(&mut self) {
        let colour = self.fifo.run_cycle(self.scroll_x, self.scroll_y, self.line_y, self.vram, self.lcdc, self.palettes);
        if let Some(colour) = colour {
            self.lcd[self.line_x as usize + self.line_y as usize * WIDTH] = COLOURS[colour as usize];
            self.line_x += 1;
        }

        if self.line_x == 160 {
            self.mode = Mode::HBlank;
            self.status &= 0x7C | Mode::HBlank as u8;
            self.fifo.x_pos = 0;
            self.line_x = 0;
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

        objects
    }

    fn update_mode(&mut self) -> Interrupts {
        let mut interrupts = Interrupts::empty();
        
        self.cycles_line += 1;
        // if self.cycles_line == LINE_LEN {
        //     self.cycles_line = 0;
        //     println!("New line new line new linety line")
        // }
        if self.cycles_line == DRAW_START && self.mode != Mode::VBlank {
            self.fifo.x_pos = 0;
            self.line_x = 0;
            self.mode = Mode::Drawing;
            self.status &= 0x7C | Mode::Drawing as u8;
            //println!("Change to drawing");
            //dbg!(self.line_y);
        }
        else if self.cycles_line == LINE_LEN {
            self.line_y += 1;
            self.cycles_line = 0;
            self.line_x = 0;
            
            if self.line_y == FRAME_SCANLINES {
                //println!("reset");
                self.line_y = 0;
                self.mode = Mode::OAMScan;
            }
            else if self.line_y == VBLANK_START {
                //println!("VBLANK");
                self.mode = Mode::VBlank;
                self.status &= 0x7C | Mode::VBlank as u8;
                interrupts |= Interrupts::VBlank;
            }
            
            if self.mode != Mode::VBlank {
                self.enter_oam_scan();
            }
        }
        //dbg!(self.mode);
        interrupts
    }

    fn enter_oam_scan(&mut self) {
        //println!("Change to OAM scan");
        self.mode = Mode::OAMScan;
        self.objects_array = self.oam_search();
        self.status &= 0x7C | Mode::OAMScan as u8;
    }

    fn update_stat(&mut self) -> bool {
        let old_stat_flag = self.stat_flag;
        if self.line_y == self.line_compare {
            self.status |= StatReg::LycLy as u8;

            if self.status | StatReg::LycInt as u8 == StatReg::LycInt as u8 {
                self.stat_flag = true;
            }
        }

        if self.status | StatReg::HBlankInt as u8 == StatReg::HBlankInt as u8 && self.mode == Mode::HBlank {
            self.stat_flag = true;
        }

        if self.status | StatReg::VBlankInt as u8 == StatReg::VBlankInt as u8 && self.mode == Mode::VBlank {
            self.stat_flag = true;
        }

        if self.status | StatReg::OamInt as u8 == StatReg::OamInt as u8 && self.mode == Mode::OAMScan {
            self.stat_flag = true;
        }

        !old_stat_flag && self.stat_flag
    }
}