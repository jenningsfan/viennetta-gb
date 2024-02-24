use super::Interrupts;
use bitflags::bitflags;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
pub type LcdPixels = [u16; WIDTH * HEIGHT];

const OAM_START: u8 = 0;
const DRAW_START: u8 = 20;
const HBLANK_START: u8 = 63;
const LINE_LEN: u8 = 114;
const VBLANK_START: u8 = 144;
const VBLANK_LEN: u8 = 10;
const FRAME_SCANLINES: u8 = VBLANK_START + VBLANK_LEN;

bitflags! {
    #[derive(Debug)]
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Default, Debug)]
struct Palettes {
    bg_palette: u8,
    obj1_palette: u8,
    obj2_palette: u8,
}

#[derive(Debug)]
pub struct PPU {
    lcd: LcdPixels,
    mode: Mode,
    scanline: u8,
    cycles_line: u8,
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
}

impl Default for PPU {
    fn default() -> Self {
        Self {
            mode: Mode::OAMScan,
            scanline: 0,
            cycles_line: 0,
            lcd: [0; WIDTH * HEIGHT],
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
        }
    }
}

impl PPU {
    pub fn get_frame(&self) -> LcdPixels {
        self.lcd

        // to be used later
        // result[i] = match pixel {
        //     Colour::White => 0xFF,
        //     Colour::LightGrey => 0x73,
        //     Colour::DarkGrey => 0x4B,
        //     Colour::Black => 0x00,
        // };

        // result[i + 1] = match pixel {
        //     Colour::White => 0xFF,
        //     Colour::LightGrey => 0xB5,
        //     Colour::DarkGrey => 0x6B,
        //     Colour::Black => 0x00,
        // };
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
            0xFF44 => self.scanline,
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

        for _ in 0..cycles {
            interrupts |= self.run_cycle();
        }

        interrupts
    }

    fn run_cycle(&mut self) -> Interrupts {
        let mut interrupts = Interrupts::empty();
        interrupts |= self.update_mode();
        if self.update_stat() {
            interrupts |= Interrupts::LcdStat;
        }        

        interrupts
    }

    fn update_mode(&mut self) -> Interrupts {
        let mut interrupts = Interrupts::empty();

        if !self.lcdc.contains(LCDC::PpuEnable) {
            self.lcd = [0; WIDTH * HEIGHT];
            return interrupts
        }

        self.cycles_line += 1;
        if self.cycles_line == DRAW_START {
            self.mode = Mode::Drawing;
            self.status &= 0x7C | Mode::Drawing as u8;
        }
        else if self.cycles_line == HBLANK_START {
            self.mode = Mode::HBlank;
            self.status &= 0x7C | Mode::HBlank as u8;
        }
        else if self.cycles_line == LINE_LEN {
            self.mode = Mode::OAMScan;
            self.status &= 0x7C | Mode::OAMScan as u8;
            self.scanline += 1;

            if self.scanline == FRAME_SCANLINES {
                self.scanline = 0;
            }
            else if self.scanline == VBLANK_START {
                self.mode = Mode::VBlank;
                self.status &= 0x7C | Mode::VBlank as u8;
                interrupts |= Interrupts::VBlank;
            }
        }

        interrupts
    }

    fn update_stat(&mut self) -> bool {
        let old_stat_flag = self.stat_flag;
        if self.scanline == self.line_compare {
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