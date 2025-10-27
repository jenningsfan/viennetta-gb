use std::time;

use log::warn;
use dbg_hex::dbg_hex;
use chrono::{Duration, TimeZone, Timelike, Utc};

const EIGHT_KILOBYTES: usize = 8 * 1024;
const SIXTEEN_KILOBYTES: usize = 16 * 1024;
const THIRTY_TWO_KILOBYTES: usize = 32 * 1024;

const MBC_TYPE_ADDR: usize = 0x147;
const ROM_SIZE_ADDR: usize = 0x148;
const RAM_SIZE_ADDR: usize = 0x149;

trait MBC: std::fmt::Debug {
    fn from_cart_header(mbc_type: u8, rom_banks: usize, ram_banks: usize, rom: Vec<u8>) -> Self where Self: Sized;
    fn read_rom(&self, address: u16) -> u8;
    fn write_rom(&mut self, address: u16, value: u8);
    fn read_ram(&self, address: u16) -> u8;
    fn write_ram(&mut self, address: u16, value: u8);
    fn get_save_data(&self) -> Option<&Vec<u8>>;
    fn get_extra_data(&self) -> Option<Vec<&u8>> {
        None
    }
}

#[derive(Debug)]
struct MBC1 {
    ram_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,
    total_rom_banks: u8,
    total_ram_banks: u8,
    advanced_bank_mode: bool,
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl MBC for MBC1 {
    fn from_cart_header(mbc_type: u8, rom_banks: usize, ram_banks: usize, rom: Vec<u8>) -> Self {
        let ram_bytes = EIGHT_KILOBYTES * ram_banks;
        let ram = vec![0; ram_bytes];

        Self {
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            total_rom_banks: rom_banks as u8,
            total_ram_banks: ram_banks as u8,
            advanced_bank_mode: false,
            rom,
            ram,
        }
    }

    fn read_rom(&self, address: u16) -> u8 {
        let bank = match address {
            0x0000..=0x3FFF => {
                if self.advanced_bank_mode {
                    self.rom_bank & 0x60
                }
                else {
                    0
                }
            },
            0x4000..=0x7FFF => {
                self.rom_bank
            }
            _ => panic!("{address} not a valid ROM address")
        };
        let address = bank as usize * SIXTEEN_KILOBYTES + (address as usize & 0x3FFF);

        self.rom[address]
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // RAM enable
                self.ram_enabled = value & 0xA == 0xA;
            }
            0x2000..=0x3FFF => {
                let mut bank = value & 0x1F;
                if bank == 0 {
                    bank = 1;
                }
                let mask = (self.total_rom_banks >> 1) | ((self.total_rom_banks >> 1) - 1);
                bank &= mask;

                self.rom_bank = (self.rom_bank & 0x60) | bank;
            }
            0x4000..=0x5FFF => {
                if self.total_ram_banks == 4 {
                    self.ram_bank = value & 0x3;
                }
                if self.total_rom_banks >= 64 {
                    self.ram_bank = (self.ram_bank & 0x1F) | ((value & 0x3) << 5);
                }
                //dbg!(self.ram_bank);
            }
            0x6000..=0x7FFF => {
                self.advanced_bank_mode = value & 0x1 == 1;
            }
            _ => panic!("{address} not a valid ROM address")
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if self.ram_enabled {
            let address = ((self.ram_bank as u16) << 12) | address;
            //dbg!(address);
            self.ram[address as usize]
        }
        else {
            0xFF
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if self.ram_enabled {
            let address = ((self.ram_bank as u16) << 12) | address;
            //dbg!(address);
            self.ram[address as usize] = value;
        }
    }

    fn get_save_data(&self) -> Option<&Vec<u8>> {
        if self.total_ram_banks > 0 {
            Some(&self.ram)
        }
        else {
            None
        }
    }
}

// uses a horrific hack to get rtc and libretro playing together as friends
// just save rtc data into the ram
// yeah that is stupid but whatever
// there is almost certainly a better way to do this
#[derive(Debug)]
struct MBC3 {
    rom_bank: u8,
    ram_bank: u8,
    total_rom_banks: u8,
    total_ram_banks: u8,
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    rtc_latch: [u8; 5],
    last_latch: i8,
}

impl MBC3 {
    fn read_rtc_regs(&self) -> [u8; 5] {
        let ram_bytes = EIGHT_KILOBYTES * self.total_ram_banks as usize;
        let time_delta = u32::from_le_bytes(self.ram[ram_bytes..ram_bytes + 4].try_into().unwrap());
        let time = Utc::now() + Duration::seconds(time_delta as i64);

        [time.second() as u8, time.minute() as u8, time.hour() as u8, self.ram[ram_bytes + 4], self.ram[ram_bytes + 5]]
    }
}

impl MBC for MBC3 {
    fn from_cart_header(mbc_type: u8, rom_banks: usize, ram_banks: usize, rom: Vec<u8>) -> Self {
        let ram_bytes = EIGHT_KILOBYTES * ram_banks + 11;
        let ram = vec![0; ram_bytes];

        Self {
            ram_enabled: false,
            rom_bank: 0,
            ram_bank: 0,
            total_rom_banks: rom_banks as u8,
            total_ram_banks: ram_banks as u8,
            rom,
            ram,
            rtc_latch: [0; 5],
            last_latch: -1,
        }
    }

    fn read_rom(&self, address: u16) -> u8 {
        let bank = match address {
            0x0000..=0x3FFF => 0,
            0x4000..=0x7FFF => self.rom_bank,
            _ => panic!("{address} not a valid ROM address")
        };
        let address = bank as usize * SIXTEEN_KILOBYTES + (address as usize & 0x3FFF);

        self.rom[address]
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // RAM enable
                self.ram_enabled = value & 0xA == 0xA;
            }
            0x2000..=0x3FFF => {
                let mask = (self.total_rom_banks >> 1) | ((self.total_rom_banks >> 1) - 1);
                let mut bank = value & mask;
                if bank == 0 {
                    bank = 1;
                }
                
                self.rom_bank = bank;
            }
            0x4000..=0x5FFF => {
                let mask = if self.total_ram_banks > 1 {
                    (self.total_ram_banks >> 1) | ((self.total_ram_banks >> 1) - 1)
                }
                else {
                    self.total_ram_banks
                };
                self.ram_bank = value & mask;
            }
            0x6000..=0x7FFF => {                
                if self.last_latch == 0 && value == 1 {
                    self.rtc_latch = self.read_rtc_regs();
                }
                self.last_latch = value as i8;
            }
            _ => panic!("{address} not a valid ROM address")
        }
    }



    fn read_ram(&self, address: u16) -> u8 {
        // TODO: RTC
        if self.ram_enabled {
            match self.ram_bank {
                0..=3 => {
                    let address = ((self.ram_bank as u16) << 12) | address;
                    self.ram[address as usize]
                },
                 
                0x8..=0xC => {
                    let regs = if self.last_latch == -1 { self.read_rtc_regs() } else { self.rtc_latch };
                    regs[self.ram_bank as usize - 8]
                }
                _ => panic!("{} not valid ram bank", self.ram_bank),
            }
        }
        else {
            0xFF
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        // TODO: RTC
        if self.ram_enabled {
            let ram_bytes = EIGHT_KILOBYTES * self.total_ram_banks as usize;
            let mut time_delta = u32::from_le_bytes(self.ram[ram_bytes..ram_bytes + 4].try_into().unwrap());
            let mut time = Utc::now() + Duration::seconds(time_delta as i64);

            match self.ram_bank {
                0..=3 => {
                    let address = ((self.ram_bank as u16) << 12) | address;
                    self.ram[address as usize] = value;
                },
                0x8 => time = time.with_second(value.into()).unwrap_or(time),
                0x9 => time = time.with_minute(value.into()).unwrap_or(time),
                0xA => time = time.with_hour(value.into()).unwrap_or(time),
                0xB => self.ram[ram_bytes + 4] = value,
                0xC => self.ram[ram_bytes + 5] = value,
                _ => panic!("{} not valid ram bank", self.ram_bank),
            }

            time_delta = (time - Utc::now()).num_seconds() as u32;
            self.ram[ram_bytes..ram_bytes + 4].copy_from_slice(&time_delta.to_le_bytes());
        }
    }

    fn get_save_data(&self) -> Option<&Vec<u8>> {
        if self.total_ram_banks > 0 {
            Some(&self.ram)
        }
        else {
            None
        }
    }
}

#[derive(Debug)]
struct MBC5 {
    ram_enabled: bool,
    rom_bank: u16,
    ram_bank: u8,
    total_rom_banks: u16,
    total_ram_banks: u8,
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl MBC for MBC5 {
    fn from_cart_header(mbc_type: u8, rom_banks: usize, ram_banks: usize, rom: Vec<u8>) -> Self {
        let ram_bytes = EIGHT_KILOBYTES * ram_banks;
        let ram = vec![0; ram_bytes];

        dbg!(rom_banks);
        dbg!(ram_banks);

        Self {
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            total_rom_banks: rom_banks as u16,
            total_ram_banks: ram_banks as u8,
            rom,
            ram,
        }
    }

    fn read_rom(&self, address: u16) -> u8 {
        let bank = match address {
            0x0000..=0x3FFF => {
                0
            },
            0x4000..=0x7FFF => {
                self.rom_bank
            }
            _ => panic!("{address} not a valid ROM address")
        };
        let address = bank as usize * SIXTEEN_KILOBYTES + (address as usize & 0x3FFF);
        self.rom[address]
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // RAM enable
                self.ram_enabled = value & 0xA == 0xA;
            }
            0x2000..=0x2FFF => {
                let mask = (self.total_rom_banks >> 1) | ((self.total_rom_banks >> 1) - 1);
                self.rom_bank = (self.rom_bank & 0x100) | value as u16;
                self.rom_bank &= mask;
            }
            0x3000..=0x3FFF => {
                if self.total_rom_banks > 255 {
                    self.rom_bank = (self.rom_bank & 0xFF) | ((value as u16 & 1) << 8);
                }
            }
            0x4000..=0x5FFF => {
                let mask = if self.total_ram_banks > 1 {
                    (self.total_ram_banks >> 1) | ((self.total_ram_banks >> 1) - 1)
                }
                else {
                    self.total_ram_banks
                };
                self.ram_bank = value & mask;
            }
            _ => warn!("{address} not a valid ROM address")
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if self.ram_enabled {
            let address = ((self.ram_bank as u16) << 12) | address;
            self.ram[address as usize]
        }
        else {
            0xFF
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if self.ram_enabled {
            let address = ((self.ram_bank as u16) << 12) | address;
            self.ram[address as usize] = value;
        }
    }

    fn get_save_data(&self) -> Option<&Vec<u8>> {
        if self.total_ram_banks > 0 {
            Some(&self.ram)
        }
        else {
            None
        }
    }
}

#[derive(Debug)]
struct NoMBC {
    rom: Vec<u8>,
    ram: Vec<u8>,
    has_ram: bool,
}

impl MBC for NoMBC {
    fn from_cart_header(mbc_type: u8, rom_banks: usize, ram_banks: usize, rom: Vec<u8>) -> Self {
        if rom_banks != 2 || ram_banks > 1 {
            panic!("Rom banks must be 2 but is: {rom_banks}. Ram banks should be 0 or 1 but is: {ram_banks}");
        }

        let ram_bytes = EIGHT_KILOBYTES * ram_banks;
        let ram = vec![0; ram_bytes];

        Self {
            rom,
            ram,
            has_ram: ram_banks > 0,
        }
    }

    fn read_rom(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        // this is a no-op without an mbc
    }

    fn read_ram(&self, address: u16) -> u8 {
        if self.has_ram {
            self.ram[address as usize]
        }
        else {
            0xFF
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if self.has_ram {
            self.ram[address as usize] = value;
        }
    }

    fn get_save_data(&self) -> Option<&Vec<u8>> {
        if self.has_ram {
            Some(&self.ram)
        }
        else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Cartridge {
    mbc: Box<dyn MBC>,
}

impl Cartridge {
    pub fn new(game_rom: &[u8]) -> Self {
        let rom = game_rom.to_vec();

        let predicted_rom_size = THIRTY_TWO_KILOBYTES * (1 << rom[ROM_SIZE_ADDR]);
        if rom.len() != predicted_rom_size {
            panic!("Rom loaded is {} bytes but it should be {predicted_rom_size}", rom.len());
        }

        let rom_banks = 1 << (rom[ROM_SIZE_ADDR] + 1) as usize;
        let ram_banks = [0, 0, 1, 4, 16, 8][rom[RAM_SIZE_ADDR] as usize];

        dbg!(rom[RAM_SIZE_ADDR]);

        let mbc = Self::mbc_from_cart_header(rom[MBC_TYPE_ADDR],
            rom_banks, ram_banks, rom);

        Self {
            mbc,
        }
    }

    fn mbc_from_cart_header(mbc_type: u8, rom_banks: usize, ram_banks: usize, rom: Vec<u8>) -> Box<dyn MBC> {
        if mbc_type == 0x00 {
            Box::new(NoMBC::from_cart_header(mbc_type, rom_banks, ram_banks, rom))
        } 
        else if mbc_type >= 0x01 && mbc_type <= 0x03 {
            Box::new(MBC1::from_cart_header(mbc_type, rom_banks, ram_banks, rom))
        }
        else if mbc_type >= 0x0F && mbc_type <= 0x13 {
            Box::new(MBC3::from_cart_header(mbc_type, rom_banks, ram_banks, rom))
        }
        else if mbc_type >= 0x19 && mbc_type <= 0x1E {
            Box::new(MBC5::from_cart_header(mbc_type, rom_banks, ram_banks, rom))
        }
        else {
            warn!("Unsopported MBC {mbc_type:02X}. Defaulting to no mbc");
            Box::new(NoMBC::from_cart_header(mbc_type, rom_banks, ram_banks, rom)) // Default case
        }
    }

    pub fn read_rom(&self, address: u16) -> u8 {
        self.mbc.read_rom(address)
    }

    pub fn write_rom(&mut self, address: u16, value: u8) {
        self.mbc.write_rom(address, value);
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        self.mbc.read_ram(address)
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        self.mbc.write_ram(address, value);
    }

    pub fn get_save_data(&self) -> Option<&Vec<u8>> {
        self.mbc.get_save_data()
    }
}