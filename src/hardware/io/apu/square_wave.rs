use crate::hardware::io::apu::warn;
const WAVE_PATTERNS: [u8; 4] = [0b00000001, 0b00000011, 0b00001111, 0b11111100];

#[derive(Debug, Default)]
pub struct SquareWave {
    pub enable: bool,
    wave_duty: u8,
    wave_position: u8,
    frequency_timer: u16,
    length_timer: u8,
    initial_length_timer: u8,
    frequency: u16,
    pub left_pan: bool,
    pub right_pan: bool,
    initial_volume: u8,
    envelope_is_increase: bool,
    envelope_period: u8,
    length_timer_enabled: bool,
}

impl SquareWave {
    pub fn run_cycle(&mut self) {
        if !self.enable {
            return;
        }

        self.frequency_timer -= 1;
        if self.frequency_timer == 0 {
            self.wave_position += 1;
            if self.wave_position > 7 {
                self.wave_position = 0;
            }

            self.frequency_timer = (2048 - self.frequency) * 4;
        }
    }

    pub fn tick_length_timer(&mut self) {
        if self.length_timer_enabled {
            self.length_timer -= 1;
            if self.length_timer == 0 {
                self.enable = false;
                println!("turn off channel");
                //self.length_timer = 64 - self.initial_length_timer;
            }
        }
    }

    fn trigger_event(&mut self) {
        self.enable = true;
        if self.length_timer == 0 {
            self.length_timer = 64;
        }
    }

    pub fn read_io(&self, address: u16) -> u8 {
        // TODO: some of these are supposed to be write-only?
        match address & 0xF {   // mask to only get the last nibble to get register regardless of channel1 or channel2
            0 => { warn!("TODO: sweep"); 0xFF }, // TODO: sweep
            1 => {
                (self.wave_duty << 6) | 0x3F
            },
            2 => {
                (self.initial_volume << 6)
                    | if self.envelope_is_increase { 0x8 } else { 0 }
                    | self.envelope_period
            }
            3 => {
                0xFF // write only
            }
            4 => {
                (if self.length_timer_enabled { 0x40 } else { 0 }) | 0xBF
            }
            _ => { warn!("{address} not valid APU io address"); 0xFF }
        }
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        match address & 0xF {   // mask to only get the last nibble to get register regardless of channel1 or channel2
            0 => warn!("TODO: sweep"), // TODO: sweep
            1 => {
                self.wave_duty = value >> 6;
                self.initial_length_timer = value & 0x3F;
                self.length_timer = 64 - self.initial_length_timer;
            },
            2 => {
                self.initial_volume = value >> 6;
                self.envelope_is_increase = value & 0x8 == 0x8;
                self.envelope_period = value & 0x7;
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

    pub fn get_amplitude(&self) -> f32 {
        if !self.enable {
            return 0.0;
        }
        
        let mut amplitude = (WAVE_PATTERNS[self.wave_duty as usize] >> self.wave_position) & 1;
        amplitude *= self.initial_volume;
        let scaled = (amplitude as f32 / 7.5) - 1.0;

        scaled
    }
}