const WAVE_PATTERNS: [u8; 4] = [0b00000001, 0b00000011, 0b00001111, 0b11111100];

#[derive(Debug, Default)]
pub struct SquareWave {
    pub enable: bool,
    pattern_number: u8,
    wave_position: u8,
    frequency_timer: u16,
    frequency: u16,
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

    pub fn get_amplitude(&self) -> u8 {
        if !self.enable {
            return 0;
        }
        
        (WAVE_PATTERNS[self.pattern_number as usize] >> self.wave_position) & 1
    }
}