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
    current_volume: u8,
    volume_enevelope_timer: u8, 
    sweep_period: u8,
    sweep_is_downwards: bool,
    sweep_change: u8,
    sweep_enabled: bool,
    shadow_freq: u16,
    sweep_timer: u8,
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
                //println!("turn off channel");
                //self.length_timer = 64 - self.initial_length_timer;
            }
        }
    }

    pub fn tick_volume_envelope(&mut self) {
        if self.envelope_period == 0 {
            return;
        }

        if self.volume_enevelope_timer != 0 {
            self.volume_enevelope_timer -= 1;
            if self.volume_enevelope_timer == 0 {
                self.volume_enevelope_timer = self.envelope_period;

                if self.envelope_is_increase && self.current_volume < 0xF {
                    self.current_volume += 1;
                }

                if !self.envelope_is_increase && self.current_volume > 0x0 {
                    self.current_volume -= 1;
                }
            }
        }
    }

    pub fn tick_freq_sweep(&mut self) {
        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;

            if self.sweep_timer == 0 {
                self.sweep_timer = self.sweep_period;
                if self.sweep_timer == 0 {
                    self.sweep_timer = 8;
                }

                if self.sweep_enabled && self.sweep_period != 0 {
                    // caluclate new frequency
                    let mut new_freq = self.shadow_freq >> self.sweep_change;

                    if self.sweep_is_downwards {
                        new_freq = self.shadow_freq - new_freq;
                    }
                    else {
                        new_freq = self.shadow_freq + new_freq;
                    }

                    if new_freq < 2048 {
                        if self.sweep_change > 0 {
                            self.frequency = new_freq;
                            self.shadow_freq = new_freq;
                        }
                    }
                    else {
                        self.enable = false;
                    }
                }
            }
        }
    }

    fn trigger_event(&mut self) {
        self.enable = true;
        if self.length_timer == 0 {
            self.length_timer = 64;
        }
        self.current_volume = self.initial_volume;
        self.volume_enevelope_timer = self.envelope_period;

        self.shadow_freq = self.frequency;
        self.sweep_timer = self.sweep_period;
        if self.sweep_timer == 0 {
            self.sweep_timer = 8;
        }

        if self.sweep_period > 0 || self.sweep_change > 0 {
            self.sweep_enabled = true;
        }
        else {
            self.sweep_enabled = false;
        }

        if self.sweep_change > 0 {
            if self.frequency > 2048 {
                self.enable = false;
            }
        }

        // println!("Frequency: {}", self.frequency);
        // println!("Volume: {}", self.current_volume);
        // println!("Envelope period: {}", self.envelope_period);
        // println!("Duty cycle: {}\n", self.wave_duty);
    }

    pub fn read_io(&self, address: u16) -> u8 {
        // TODO: some of these are supposed to be write-only?
        match address & 0xF {   // mask to only get the last nibble to get register regardless of channel1 or channel2
            0 => {
                (self.sweep_period << 4)
                    | if self.sweep_is_downwards { 0x8 } else { 0 }
                    | self.sweep_change
            },
            1 | 6 => {
                (self.wave_duty << 6) | 0x3F
            },
            2 | 7 => {
                (self.initial_volume << 4)
                    | if self.envelope_is_increase { 0x8 } else { 0 }
                    | self.envelope_period
            }
            3 | 8 => {
                0xFF // write only
            }
            4 | 9 => {
                (if self.length_timer_enabled { 0x40 } else { 0 }) | 0xBF
            }
            _ => { warn!("{address} not valid APU io address"); 0xFF }
        }
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        match address & 0xF {   // mask to only get the last nibble to get register regardless of channel1 or channel2
            0 => {
                self.sweep_period = (value >> 4) & 0x7;
                self.sweep_is_downwards = value & 0x8 == 0x8;
                self.sweep_change = value & 0x7;
            }
            1 | 6 => {
                self.wave_duty = value >> 6;
                self.initial_length_timer = value & 0x3F;
                self.length_timer = 64 - self.initial_length_timer;
            },
            2 | 7 => {
                self.initial_volume = value >> 4;
                self.envelope_is_increase = value & 0x8 == 0x8;
                self.envelope_period = value & 0x7;
            },
            3 | 8 => {
                self.frequency = (self.frequency & 0x700) | value as u16;
            },
            4 | 9 => {
                self.length_timer_enabled = value & 0x40 == 0x40;
                self.frequency = ((value as u16 & 0x7) << 8) | (self.frequency & 0xFF);
                if value & 0x80 == 0x80 {
                    self.trigger_event();
                }
            },
            _ => warn!("{address} not valid APU io address")
        };
    }

    pub fn get_amplitude(&self) -> (u8, u8) {
        if !self.enable {
            return (0, 0);
        }
        
        let mut amplitude = (WAVE_PATTERNS[self.wave_duty as usize] >> self.wave_position) & 1;
        amplitude *= self.current_volume;

        let left = if self.left_pan { amplitude } else { 0 };
        let right = if self.right_pan { amplitude } else { 0 };

        (left, right)
    }
}