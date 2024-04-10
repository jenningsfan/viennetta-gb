#[derive(Debug, Default)]
pub struct APU {
    
}

impl APU {
    pub fn read_io(&self, address: u16) -> u8 {
        0xFF
    }

    pub fn write_io(&mut self, address: u16, value: u8) {
        
    }

    pub fn read_wave(&self, address: u16) -> u8 {
        0xFF
    }

    pub fn write_wave(&mut self, address: u16, value: u8) {
        
    }
}

struct SquareWave {

}