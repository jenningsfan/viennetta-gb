use bitflags::bitflags;
use crate::hardware::memory::Memory;

bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Flags: u8 {
        const Zero = 1 << 7;
        const Sub = 1 << 6;
        const HalfCarry = 1 << 5;
        const Carry = 1 << 4;
    }
}

#[derive(Default, Debug)]
pub struct Registers {
    pub flags: Flags,

    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn get_r8(&self, reg: u8, memory: &Memory) -> u8 {
        match reg {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => memory[self.get_hl()],
            7 => self.a,

            _ => panic!("literaly impossible. it should only be 3 bits wide"),
        }
    }

    pub fn set_r8(&mut self, reg: u8, value: u8, memory: &mut Memory) {
        match reg {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => memory[self.get_hl()] = value,
            7 => self.a = value,

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    pub fn apply_r8<F: Fn(u8) -> u8>(&mut self, reg: u8, memory: &mut Memory, func: F) {
        self.set_r8(reg, func(self.get_r8(reg, memory)), memory);
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0x0F) as u8;
    }

    pub fn get_r16(&self, reg: u8) -> u16 {
        match reg {
            0 => (self.b as u16) << 8 | self.c as u16,
            1 => (self.d as u16) << 8 | self.e as u16,
            2 => self.get_hl(),
            3 => self.sp,

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    pub fn set_r16(&mut self, reg: u8, value: u16) {
        match reg {
            0 => {
                self.b = (value >> 8) as u8;
                self.c = (value & 0x0F) as u8;
            },
            1 => {
                self.d = (value >> 8) as u8;
                self.e = (value & 0x0F) as u8;
            },
            2 => {
                self.h = (value >> 8) as u8;
                self.l = (value & 0x0F) as u8;
            },
            3 => self.sp = value,

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    pub fn apply_r16(&mut self, reg: u8, func: fn(u16) -> u16) {
        self.set_r16(reg, func(self.get_r16(reg)));
    }

    pub fn get_r16_stk(&self, reg: u8) -> u16 {
        match reg {
            0 => (self.b as u16) << 8 | self.c as u16,
            1 => (self.d as u16) << 8 | self.e as u16,
            2 => self.get_hl(),
            3 => (self.a as u16) << 8 | self.flags.bits() as u16,

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    pub fn set_r16_stk(&mut self, reg: u8, value: u16) {
        match reg {
            0 => {
                self.b = (value >> 8) as u8;
                self.c = (value & 0x0F) as u8;
            },
            1 => {
                self.d = (value >> 8) as u8;
                self.e = (value & 0x0F) as u8;
            },
            2 => {
                self.h = (value >> 8) as u8;
                self.l = (value & 0x0F) as u8;
            },
            3 => {
                self.a = (value >> 8) as u8;
                self.flags = Flags::from_bits((value & 0x0F) as u8).unwrap();
            },

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    pub fn get_r16_mem(&mut self, reg: u8) -> u16 {
        let hl = self.get_hl();

        match reg {
            0 => (self.b as u16) << 8 | self.c as u16,
            1 => (self.d as u16) << 8 | self.e as u16,
            2 => {self.set_hl(hl + 1); hl},
            3 => {self.set_hl(hl - 1); hl},

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    pub fn add_acc(&mut self, value: u8) {
        let result = self.a.overflowing_add(value);
        self.a = result.0;

        if result.1 {
            self.flags |= Flags::Carry;
        }
        else {
            self.flags -= Flags::Carry;
        }
    }

    pub fn sub_acc(&mut self, value: u8) {
        let result = self.a.overflowing_sub(value);
        self.a = result.0;

        if result.1 {
            self.flags |= Flags::Carry;
        }
        else {
            self.flags -= Flags::Carry;
        }
    }

    pub fn condition(&self, condition: u8) -> bool {
        match condition {
            0x0 => !self.flags.contains(Flags::Zero),
            0x1 => self.flags.contains(Flags::Zero),
            0x2 => !self.flags.contains(Flags::Carry),
            0x3 => self.flags.contains(Flags::Carry),

            _ => panic!("should only be 2 bits wide"),
        }
    }
}