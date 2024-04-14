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
            self.wave_position += 1;
            if self.wave_position > 32 {
                self.wave_position = 0;
            }

            self.frequency_timer = (2048 - self.frequency) * 4;
        }
    }

    pub fn trigger_event(&mut self) {
        self.enable = true;
        if self.length_timer == 0 {
            self.length_timer = 256;
        }
    }

    pub fn tick_length_timer(&mut self) {
        if self.length_timer_enabled {
            self.length_timer -= 1;
            if self.length_timer == 0 {
                self.enable = false;
                //println!("turn off channel");
                //self.length_timer = 64 - self.initial_length_timer;
            }
        }
    }

    pub fn get_amplitude(&self) -> f32 {
        let mut sample = self.wave[self.wave_position as usize];
        if self.wave_position % 2 == 0 {
            sample >>= 4;
        }
        else {
            sample &= 0xF;
        }

        sample >>= self.volume;

        let scaled = (sample as f32 / 7.5) - 1.0;

        scaled
    }

    pub fn read_io(&self, address: u16) -> u8 {
        match address {
            _ => { warn!("{address} not valid APU io address"); 0xFF }
        }
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        match address & 0xF {
            0 => self.enable = value & 0x80 == 0x80,
            1 => {
                self.initial_length_timer = value as u16 & 0x3F;
                self.length_timer = 256 - self.initial_length_timer;
            },
            2 => {
                self.volume = value >> 4;
                if self.volume == 0 {
                    self.volume = 4;
                }
                else {
                    self.volume -= 1;
                }
            },
            3 => {
                self.frequency = (self.frequency & 0x700) | value as u16;
            },
            4 => {
                if value & 0x80 == 0x80 {
                    self.trigger_event();
                }
                self.length_timer_enabled = value & 0x40 == 0x40;
                self.frequency = ((value as u16 & 0x3) << 8) | (self.frequency & 0xFF);
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