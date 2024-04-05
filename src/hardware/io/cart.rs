#[derive(Debug)]
pub struct Cartridge {
    rom: [u8; 0x8000],
}

impl Cartridge {
    pub fn new(game_rom: &[u8]) -> Self {
        let mut rom = [0; 0x8000];
        rom.copy_from_slice(game_rom);

        Self {
            rom,
        }
    }

    pub fn read_rom(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    pub fn write_rom(&mut self, _address: u16, _value: u8) {
        // Add mapper support here later
        // It is left empty on purpose
        // TODO: MBC
    }

    pub fn read_ram(&self, _address: u16) -> u8 {
        // This would be external ram
        // Left empty on purpose
        // TODO: MBC
        0xFF
    }

    pub fn write_ram(&mut self, _address: u16, _value: u8) {
        // Add mapper support here later
        // It is left empty on purpose
        // TODO: MBC
    }
}