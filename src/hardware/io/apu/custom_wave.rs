use log::warn;

#[derive(Debug, Default)]
pub struct CustomWave {
    pub enable: bool,
    pub right_pan: bool,
    pub left_pan: bool,
    wave: [u8; 16],
    length_timer: u16,
    initial_length_timer: u16,
    volume: u8,
    frequency: u16,
    length_timer_enabled: bool,
    frequency_timer: u16,
    wave_position: u8
}

impl CustomWave {
    pub fn run_cycle(&mut self) {
        if !self.enable {
            return;
        }

        self.frequency_timer -= 1;
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency) * 2;
            self.wave_position += 1;
            if self.wave_position > 31 {
                self.wave_position = 0;
            }

        }
    }

    pub fn trigger_event(&mut self) {
        self.enable = true;
        self.wave_position = 0;
        if self.length_timer == 0 {
            self.length_timer = 256;
        }
        self.frequency_timer = (2048 - self.frequency) * 2;
    }

    pub fn tick_length_timer(&mut self) {
        if self.length_timer_enabled {
            self.length_timer -= 1;
            if self.length_timer == 0 {
                self.enable = false;
            }
        }
    }

    pub fn get_amplitude(&self) -> (u8, u8) {
        if !self.enable {
            return (0, 0);
        }

        let mut sample = self.wave[self.wave_position as usize / 2];
        if self.wave_position % 2 == 0 {
            sample >>= 4;
        }
        else {
            sample &= 0xF;
        }
        // if self.enable && sample != 1 {
        //     println!("{:X}", sample);
        // }
        let vol_shift = if self.volume == 0 {
            4
        }
        else {
            self.volume - 1
        };
        sample >>= vol_shift;

        let left = if self.left_pan { sample } else { 0 };
        let right = if self.right_pan { sample } else { 0 };

        (left, right)
    }

    pub fn read_io(&self, address: u16) -> u8 {
        match address & 0xF {
            0xA => if self.enable { 0xFF } else { 0x7F },
            0xB => 0xFF,
            0xC => (self.volume << 5) | 0x9F,
            0xD => 0xFF,
            0xE => if self.length_timer_enabled { 0xFF } else { 0xBF },
            _ => { warn!("{address} not valid APU io address"); 0xFF }
        }
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        match address & 0xF {
            0xA => self.enable = value & 0x80 == 0x80,
            0xB => {
                self.initial_length_timer = value as u16;
                self.length_timer = 256 - self.initial_length_timer;
            },
            0xC => {
                self.volume = value >> 5;
            },
            0xD => {
                self.frequency = (self.frequency & 0x700) | value as u16;
            },
            0xE => {
                self.length_timer_enabled = value & 0x40 == 0x40;
                self.frequency = ((value as u16 & 0x7) << 8) | (self.frequency & 0xFF);
                if value & 0x80 == 0x80 {
                    self.trigger_event();
                }
            },
            _ => warn!("{address} not valid APU io address")
        };
    }

    pub fn read_wave(&self, address: u16) -> u8 {
        self.wave[address as usize]
    }

    pub fn write_wave(&mut self, address: u16, value: u8) {
        self.wave[address as usize] = value;
    }
}