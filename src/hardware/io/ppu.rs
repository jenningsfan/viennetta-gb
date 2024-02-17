pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
pub type LcdPixels = [Colour; WIDTH * HEIGHT];

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

#[derive(Debug)]
pub struct PPU {
    lcd: LcdPixels,
    vram: [u8; 0x2000],
    oam: [u8; 0x100],
}

impl Default for PPU {
    fn default() -> Self {
        Self {
            lcd: [Colour::Black; WIDTH * HEIGHT],
            vram: [0; 0x2000],
            oam: [0; 0x100],
        }
    }
}

impl PPU {
    pub fn get_frame(&self) -> LcdPixels {
        self.lcd
    }

    pub fn read_vram(&mut self, address: u16) -> u8 {
        self.vram[address as usize]
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        self.vram[address as usize] = value;
    }

    pub fn read_oam(&mut self, address: u16) -> u8 {
        self.oam[address as usize]
    }

    pub fn write_oam(&mut self, address: u16, value: u8) {
        self.oam[address as usize] = value;
    }


    pub fn run_cycle(&mut self) {

    }
}