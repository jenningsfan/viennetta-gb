pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
pub type LcdPixels = [Colour; WIDTH * HEIGHT];

#[derive(Debug, Clone, Copy)]
pub enum Colour {
    White = 0,
    LightGrey = 1,
    DarkGrey = 2,
    Black = 3,
}

#[derive(Debug)]
pub struct PPU {
    lcd: LcdPixels,
}

impl Default for PPU {
    fn default() -> Self {
        Self {
            lcd: [Colour::DarkGrey; WIDTH * HEIGHT]
        }
    }
}

impl PPU {
    pub fn get_frame(&self) -> LcdPixels {
        self.lcd
    }

    pub fn run_cycle(&mut self) {
        
    }
}