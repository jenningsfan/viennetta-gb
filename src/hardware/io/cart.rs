const SIXTEEN_KILOBYTES: usize = 16 * 1024;

const CART_TYPE_ADDR: usize = 0x147;
const ROM_SIZE_ADDR: usize = 0x148;
const RAM_SIZE_ADDR: usize = 0x149;

#[derive(Debug, PartialEq, Eq)]
enum MapperType {
    None,
    MBC1,
}

impl MapperType {
    fn from_cart_header(byte: u8) -> MapperType {
        match byte {
            0x00 => MapperType::None,
            0x01..=0x03 => MapperType::MBC1,
            0x04..=0xFF => MapperType::None, // todo: fill in
        }
    }
}

#[derive(Debug)]
pub struct Cartridge {
    rom: Vec<u8>,
    mapper: MapperType,
    bank_reg: u8,
    total_banks: u16,
}

impl Cartridge {
    pub fn new(game_rom: &[u8]) -> Self {
        let rom = game_rom.to_vec();

        let predicted_rom_size = SIXTEEN_KILOBYTES * 2 * (1 << rom[0x148]);
        let total_banks = 1 << rom[0x148] + 1;
        if rom.len() != predicted_rom_size {
            panic!("Rom loaded is {} bytes but it should be {predicted_rom_size}", rom.len());
        }

        let mapper = MapperType::from_cart_header(rom[CART_TYPE_ADDR]);

        // todo: mappable ram

        Self {
            rom,
            mapper,
            bank_reg: 0,
            total_banks,
        }
    }

    pub fn read_rom(&self, address: u16) -> u8 {
        let address = if address < 0x4000 {
            address as usize
        } else {
            address as usize + self.bank_reg as usize * SIXTEEN_KILOBYTES - 0x4000
        };
        self.rom[address]
    }

    pub fn write_rom(&mut self, address: u16, value: u8) {
        // Add mapper support here later
        // It is left empty on purpose
        // TODO: MBC

        if self.mapper == MapperType::MBC1 {
            if address >= 0x2000 && address < 0x4000 {
                let mask = (!self.total_banks) as u8; // flip bits. e.g. 4 needs 2 bits so it goes to 0b11;
                self.bank_reg = value & mask;

                if self.bank_reg == 0 {
                    self.bank_reg = 1;
                }

                //println!("Bank reg become {}", self.bank_reg);
            }
        }
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