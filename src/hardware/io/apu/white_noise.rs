use log::warn;

#[derive(Debug, Default)]
pub struct WhiteNoise {
    pub enable: bool,
    pub right_pan: bool,
    pub left_pan: bool,
    lfsr: u16,
    lfsr_7_bit: bool,
    frequency_timer: u16,
    freq_divisor: u8,
    freq_shift: u8,
    length_timer: u8,
    length_timer_enabled: bool,
    initial_length_timer: u8, 
    initial_volume: u8,
    envelope_is_increase: bool,
    envelope_period: u8,
    current_volume: u8,
    volume_enevelope_timer: u8,
}

impl WhiteNoise {
    fn trigger_event(&mut self) {
        self.enable = true;
        if self.length_timer == 0 {
            self.length_timer = 64;
        }
        self.current_volume = self.initial_volume;
        self.volume_enevelope_timer = self.envelope_period;
        self.lfsr = 0x7F;
        //println!("triggered");
    }

    fn reset_freq_timer(&mut self) {
        let divisor = [8, 16, 32, 48, 64, 80, 96, 112][self.freq_divisor as usize];
        self.frequency_timer = divisor << self.freq_shift;
    }

    pub fn run_cycle(&mut self) {
        self.frequency_timer -= 1;
        if self.frequency_timer == 0 {
            self.reset_freq_timer();

            let xor_result = (self.lfsr & 0x1) ^ ((self.lfsr >> 1) & 0x1);
            self.lfsr >>= 1;
            self.lfsr |= xor_result << 14;
            if self.lfsr_7_bit {
                self.lfsr &= !(1 << 6);
                self.lfsr |= xor_result >> 6;
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
        //println!("OUTPUTTING SAMPLE");
        let mut amplitude = (!self.lfsr & 1) as u8;
        amplitude *= self.current_volume;

        let left = if self.left_pan { amplitude } else { 0 };
        let right = if self.right_pan { amplitude } else { 0 };

        (left, right)
    }

    pub fn read_io(&self, address: u16) -> u8 {
        match address & 0xF {
            0 => 0xFF,
            1 => {
                (self.initial_volume << 4)
                    | if self.envelope_is_increase { 0x8 } else { 0 }
                    | self.envelope_period
            },
            2 => {
                (self.freq_shift << 4)
                    | if self.lfsr_7_bit { 0x8 } else { 0 }
                    | self.freq_divisor
            },
            3 => {
                if self.length_timer_enabled { 0x40 } else { 0 }
            },
            _ => { warn!("{address} not valid APU io address"); 0xFF }
        }
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        match address & 0xF {
            0 => {
                self.initial_length_timer = value & 0x3F;
                self.length_timer = 64 - self.initial_length_timer;
            },
            1 => {
                self.initial_volume = value >> 4;
                self.envelope_is_increase = value & 0x8 == 0x8;
                self.envelope_period = value & 0x7;
            },
            2 => {
                self.freq_divisor = value & 0x7;
                self.lfsr_7_bit = value & 0x8 == 1;
                self.freq_shift = value >> 4;
                self.reset_freq_timer();
            },
            3 => {
                if value & 0x80 == 0x80 {
                    self.trigger_event();
                }
                self.length_timer_enabled = value & 0x40 == 0x40;
            }
            _ => warn!("{address} not valid APU io address")
        };
    }
}