// TODO: this is a bare-bones implementaion for blaargs

#[derive(Default, Debug)]
pub struct Serial {
    data: u8,
    control: u8, // TODO: maybe turn into a bitflags
}

impl Serial {
    pub fn read_data(&self) -> u8 {
        self.data
    }

    pub fn write_data(&mut self, data: u8) {
        self.data = data;
    }

    pub fn read_control(&self) -> u8 {
        self.control
    }

    pub fn write_control(&mut self, control: u8) {
        if control & 0x80 == 0x80 {
            print!("{}", self.data as char);
        }

        self.control = control & 1; // only set clock select as transfer has already completed
    }
}