use log::warn;

#[derive(Debug, Default)]
pub struct WhiteNoise {
    pub enable: bool,
    pub right_pan: bool,
    pub left_pan: bool,
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