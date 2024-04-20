use log::warn;

#[derive(Debug, Default)]
pub struct WhiteNoise {
    pub enable: bool,
    pub right_pan: bool,
    pub left_pan: bool,
    lfsr: u16,
    lfsr_15_bit: bool,
    frequency_timer: u16,
    freq_divisor: u8,
    freq_shift: u8,
    length_timer: u8,
    length_timer_enabled: bool,
    initial_length_timer: u8, 
    initial_volume: u8,
    envelope_is_increase: bool,
    envelope_period: u8,
}

impl WhiteNoise {
    pub fn read_io(&self, address: u16) -> u8 {
        match address & 0xF {
            _ => { warn!("{address} not valid APU io address"); 0xFF }
        }
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        match address {
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
                self.lfsr_15_bit = value & 0x8 == 0;
                self.freq_shift = value >> 4;
            },
            3 => {
                
            }
            _ => warn!("{address} not valid APU io address")
        };
    }
}