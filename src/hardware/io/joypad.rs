use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Default)]
    pub struct Buttons: u8 {
        const A = 1 << 0;
        const B = 1 << 1;
        const Select = 1 << 2;
        const Start = 1 << 3;
        const Right = 1 << 4;
        const Left = 1 << 5;
        const Up = 1 << 6;
        const Down = 1 << 7;
    }
}

#[derive(Debug, Default)]
pub struct Joypad {
    type_select: u8,
    buttons_pressed: Buttons,
}

impl Joypad {
    pub fn update_state(&mut self, buttons: Buttons) {
        self.buttons_pressed = buttons;
    }

    pub fn read(&self) -> u8 {
        if self.type_select & 0x10 == 0x00 {
            self.type_select | (self.buttons_pressed.bits() & 0xF)
        }
        else if self.type_select & 0x20 == 0x00 {
            self.type_select | (self.buttons_pressed.bits() >> 4)
        }
        else {
            self.type_select
        }
    }

    pub fn write(&mut self, value: u8) {
        self.type_select = value & 0x30;
        //println!("{value:02X}");
    }
}