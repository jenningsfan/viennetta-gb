use log::warn;
use super::T_CYCLES_RATE;

mod square_wave;
use square_wave::SquareWave;

pub const SAMPLE_RATE: u16 = 48000;
const SAMPLING_TIMER_INTERVAL: u32 = T_CYCLES_RATE / SAMPLE_RATE as u32;

pub type SampleBuffer = Vec<i16>;

#[derive(Debug, Default)]
pub struct APU {
    vol_control: u8,
    enable: bool,
    channel1: SquareWave,
    channel2: SquareWave,
    channel3: CustomWave,
    channel4: WhiteNoise,
    pub sample_buf: SampleBuffer,
    sampling_timer: u8,
}

impl APU {
    pub fn run_cycles(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.run_cycle();
        }
    }
    
    pub fn run_cycle(&mut self) {
        self.channel1.run_cycle();
        self.channel1.enable = true;

        self.sampling_timer += 1;
        if self.sampling_timer as u32 == SAMPLING_TIMER_INTERVAL {
            self.sampling_timer = 0;
            
            let mut sample = self.channel1.get_amplitude() as i16;
            if sample == 1 {
                sample = i16::MAX;
            }

            self.sample_buf.push(sample);
            self.sample_buf.push(sample);

            //println!("pushed samples");
        }
    }

    pub fn read_io(&self, address: u16) -> u8 {
        match address {
            0xFF26 => self.calculate_control_reg(),
            0xFF25 => 0xFF, // TODO: panning
            0xFF24 => 0xFF, // TODO: volume
            _ => { warn!("{address} not valid APU io address"); 0xFF }
        }
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        
    }

    pub fn read_wave(&self, address: u16) -> u8 {
        0xFF
    }

    pub fn write_wave(&mut self, address: u16, value: u8) {
        
    }

    fn calculate_control_reg(&self) -> u8 {
        let mut enable_reg = 0;
        if self.enable {
            enable_reg |= 0x80;
        }

        if self.channel1.enable {
            enable_reg |= 0x01;
        }

        if self.channel2.enable {
            enable_reg |= 0x02;
        }

        if self.channel3.enable {
            enable_reg |= 0x04;
        }

        if self.channel4.enable {
            enable_reg |= 0x08;
        }

        enable_reg
    }
}

#[derive(Debug, Default)]
struct CustomWave {
    enable: bool,
}

#[derive(Debug, Default)]
struct WhiteNoise {
    enable: bool,
}