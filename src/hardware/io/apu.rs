use log::warn;
use super::T_CYCLES_RATE;

mod square_wave;
mod custom_wave;
use square_wave::SquareWave;
use custom_wave::CustomWave;

pub const SAMPLE_RATE: u16 = 48000;
const SAMPLING_TIMER_INTERVAL: u32 = T_CYCLES_RATE / SAMPLE_RATE as u32;

pub type SampleBuffer = Vec<i16>;

// TODO: vin - external audio from cart. not sure if any games actually did this

#[derive(Debug, Default)]
pub struct APU {
    left_vol: u8,
    right_vol: u8,
    enable: bool,
    channel1: SquareWave,
    channel2: SquareWave,
    channel3: CustomWave,
    channel4: WhiteNoise,
    pub sample_buf: SampleBuffer,
    sampling_timer: u8,
    frame_sequencer_cycle: u16,
    frame_sequencer_step: u8,
}

impl APU {
    pub fn run_cycles(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.run_cycle();
        }
    }
    
    pub fn run_cycle(&mut self) {
        self.frame_sequencer_cycle += 1;
        if self.frame_sequencer_cycle == 8192 { // TODO: falling edge bit 5 of div. shouldn't make a difference though
            self.frame_sequencer_cycle = 0;
            self.frame_sequencer_step += 1;
            if self.frame_sequencer_step == 8 {
                self.frame_sequencer_step = 0;
            }

            if self.frame_sequencer_step % 2 == 0 {
                self.channel1.tick_length_timer();
                self.channel2.tick_length_timer();
                self.channel3.tick_length_timer();
            }

            if self.frame_sequencer_step == 7 {
                self.channel1.tick_volume_envelope();
                self.channel2.tick_volume_envelope();
            }

            if self.frame_sequencer_step == 2 || self.frame_sequencer_step == 6 {
                self.channel1.tick_freq_sweep();
                self.channel2.tick_freq_sweep();
            }
        }

        self.channel1.run_cycle();
        self.channel2.run_cycle();
        self.channel3.run_cycle();

        self.sampling_timer += 1;
        if self.sampling_timer as u32 == SAMPLING_TIMER_INTERVAL {
            self.sampling_timer = 0;
            let mut sample: f32 = 0.0;

            sample += self.channel1.get_amplitude();
            sample += self.channel2.get_amplitude();
            sample += self.channel3.get_amplitude();

            sample /= 3.0;

            let sample = (sample * (i16::MAX as f32)) as i16;
            self.sample_buf.push(sample);
            self.sample_buf.push(sample);

            //println!("pushed samples");
        }
    }

    pub fn read_io(&self, address: u16) -> u8 {
        match address {
            0xFF26 => self.read_control_reg(),                 // NR52 - master control
            0xFF25 => self.read_pan_reg(),                     // NR51 - panning
            0xFF24 => self.right_vol | (self.left_vol << 4),        // NR50 - volume
            0xFF10..=0xFF14 => self.channel1.read_io(address),
            0xFF21..=0xFF24 => self.channel2.read_io(address),
            0xFF1A..=0xFF1E => self.channel3.read_io(address),
            0xFF20..=0xFF23 => self.channel2.read_io(address),
            _ => { warn!("{address} not valid APU io address"); 0xFF }
        }
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        match address {
            0xFF26 => self.write_control_reg(value),                 // NR52 - master control
            0xFF25 => self.write_pan_reg(value),                     // NR51 - panning
            0xFF24 => self.write_vol_reg(value),         // NR50 - volume
            0xFF10..=0xFF14 => self.channel1.write_io(address, value),
            0xFF21..=0xFF24 => self.channel2.write_io(address, value),
            0xFF1A..=0xFF1E => self.channel3.write_io(address, value),
            0xFF20..=0xFF23 => self.channel2.write_io(address, value),
            _ => warn!("{address} not valid APU io address")
        };
    }

    pub fn read_wave(&self, address: u16) -> u8 {
        self.channel3.read_wave(address)
    }

    pub fn write_wave(&mut self, address: u16, value: u8) {
        self.channel3.write_wave(address, value);
    }

    fn read_pan_reg(&self) -> u8 {
        (if self.channel1.right_pan { 0x01 } else { 0 })
            | (if self.channel2.right_pan { 0x02 } else { 0 })
            | (if self.channel3.right_pan { 0x04 } else { 0 })
            | (if self.channel4.right_pan { 0x08 } else { 0 })
            | (if self.channel1.left_pan { 0x10 } else { 0 })
            | (if self.channel2.left_pan { 0x20 } else { 0 })
            | (if self.channel3.left_pan { 0x40 } else { 0 })
            | (if self.channel4.left_pan { 0x80 } else { 0 })
    }

    fn write_pan_reg(&mut self, value: u8)  {
        self.channel1.right_pan = value & 0x01 == 0x01;
        self.channel2.right_pan = value & 0x02 == 0x02;
        self.channel3.right_pan = value & 0x04 == 0x04;
        self.channel4.right_pan = value & 0x08 == 0x08;
        self.channel1.left_pan = value & 0x10 == 0x10; 
        self.channel2.left_pan = value & 0x20 == 0x20; 
        self.channel3.left_pan = value & 0x40 == 0x40; 
        self.channel4.left_pan = value & 0x80 == 0x80;
    }

    fn read_control_reg(&self) -> u8 {
        (if self.enable { 0x80 } else { 0 })
            | (if self.channel1.enable { 0x01 } else { 0 })
            | (if self.channel2.enable { 0x02 } else { 0 })
            | (if self.channel3.enable { 0x04 } else { 0 })
            | (if self.channel4.enable { 0x08 } else { 0 })
    }

    fn write_control_reg(&mut self, value: u8)  {
        self.enable = value & 0x80 == 0x80;
    }

    fn write_vol_reg(&mut self, value: u8) {
        self.right_vol = value & 0x7;
        self.left_vol = (value >> 4) & 0x7;
    }
}

#[derive(Debug, Default)]
struct WhiteNoise {
    enable: bool,
    right_pan: bool,
    left_pan: bool,
}

impl WhiteNoise {
    pub fn read_io(&self, address: u16) -> u8 {
        match address {
            _ => { warn!("{address} not valid APU io address"); 0xFF }
        }
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        match address {
            _ => warn!("{address} not valid APU io address")
        };
    }
}