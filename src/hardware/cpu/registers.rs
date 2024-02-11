use bitflags::bitflags;
use crate::hardware::memory::Memory;

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u8 {
        const Zero = 1 << 7;
        const Negative = 1 << 6;
        const HalfCarry = 1 << 5;
        const Carry = 1 << 4;
    }
}

impl Flags {
    pub fn to_char(self) -> char {
        match self {
            Flags::Carry => 'C',
            Flags::HalfCarry => 'H',
            Flags::Negative => 'N',
            Flags::Zero => 'Z',
            _ => panic!("too big"),
        }
    }
}

#[derive(Debug)]
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

impl Default for Registers {
    fn default() -> Self {
        Self {
            flags: Flags::default(),
            
            a: 1,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            
            sp: 0xFFFE,
            pc: 0x100,
        }
    }
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

            _ => panic!("literaly impossible. it should only be 3 bits wide at {:04x}", self.pc),
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

            _ => panic!("opcode segment should only be 3 bits wide at {:04x}", self.pc),
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
        self.l = (value & 0xFF) as u8;
    }

    pub fn get_r16(&self, reg: u8) -> u16 {
        match reg {
            0 => (self.b as u16) << 8 | self.c as u16,
            1 => (self.d as u16) << 8 | self.e as u16,
            2 => self.get_hl(),
            3 => self.sp,

            _ => panic!("opcode segment should only be 3 bits wide at {:04x}", self.pc),
        }
    }

    pub fn set_r16(&mut self, reg: u8, value: u16) {
        match reg {
            0 => {
                self.b = (value >> 8) as u8;
                self.c = (value & 0xFF) as u8;
            },
            1 => {
                self.d = (value >> 8) as u8;
                self.e = (value & 0xFF) as u8;
            },
            2 => {
                self.h = (value >> 8) as u8;
                self.l = (value & 0xFF) as u8;
            },
            3 => self.sp = value,

            _ => panic!("opcode segment should only be 3 bits wide at {:04x}", self.pc),
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

            _ => panic!("opcode segment should only be 3 bits wide at {:04x}", self.pc),
        }
    }

    pub fn set_r16_stk(&mut self, reg: u8, value: u16) {
        match reg {
            0 => {
                self.b = (value >> 8) as u8;
                self.c = (value & 0xFF) as u8;
            },
            1 => {
                self.d = (value >> 8) as u8;
                self.e = (value & 0xFF) as u8;
            },
            2 => {
                self.h = (value >> 8) as u8;
                self.l = (value & 0xFF) as u8;
            },
            3 => {
                self.a = (value >> 8) as u8;
                self.flags = Flags::from_bits((value & 0xF0) as u8)
                    .ok_or_else(|| format!("Failed to convert {value:04X} to Flags {:04x}", self.pc)).unwrap();
            },

            _ => panic!("opcode segment should only be 3 bits wide at {:04x}", self.pc),
        }
    }

    pub fn get_r16_mem(&mut self, reg: u8) -> u16 {
        let hl = self.get_hl();

        match reg {
            0 => (self.b as u16) << 8 | self.c as u16,
            1 => (self.d as u16) << 8 | self.e as u16,
            2 => {self.set_hl(hl + 1); hl},
            3 => {self.set_hl(hl - 1); hl},

            _ => panic!("opcode segment should only be 3 bits wide at {:04x}", self.pc),
        }
    }

    pub fn add_acc(&mut self, value: u8) {
        let result = self.a.overflowing_add(value);
        
        self.flags = Flags::empty();

        if result.0 == 0 {
            self.flags |= Flags::Zero;
        }
        if result.1 {
            self.flags |= Flags::Carry;
        }
        if (((value & 0xF) + (self.a & 0xF)) & 0x10) == 0x10 {
            self.flags |= Flags::HalfCarry;
        }

        self.a = result.0;
    }

    pub fn add_r8(&mut self, reg: u8, value: u8, memory: &mut Memory, set_carry: bool) {
        let result = self.get_r8(reg, memory).overflowing_add(value);
        
        self.flags = Flags::empty();

        if result.0 == 0 {
            self.flags |= Flags::Zero;
        }
        if result.1 && set_carry {
            self.flags |= Flags::Carry;
        }
        if (((value & 0xF) + (self.a & 0xF)) & 0x10) == 0x10 {
            self.flags |= Flags::HalfCarry;
        }

        self.set_r8(reg, result.0, memory);
    }

    pub fn sub_r8(&mut self, reg: u8, value: u8, memory: &mut Memory, set_carry: bool)  {
        let result = self.get_r8(reg, memory).overflowing_sub(value);

        self.flags = Flags::Negative;

        if result.0 == 0 {
            self.flags |= Flags::Zero;
        }
        if result.1 {
            self.flags |= Flags::Carry;
        }
        if (value & 0xF) > (self.get_r8(reg, memory) & 0xF) {
            self.flags |= Flags::HalfCarry;
        }

        self.set_r8(reg, result.0, memory);
    }

    pub fn sub_acc(&mut self, value: u8) {
        let result = self.a.overflowing_sub(value);
        self.flags = Flags::Negative;

        if result.0 == 0 {
            self.flags |= Flags::Zero;
        }
        if result.1 {
            self.flags |= Flags::Carry;
        }
        if (value & 0xF) > (self.a & 0xF) {
            self.flags |= Flags::HalfCarry;
        }

        self.a = result.0;
    }

    pub fn condition(&self, condition: u8) -> bool {
        match condition {
            0x0 => !self.flags.contains(Flags::Zero),
            0x1 => self.flags.contains(Flags::Zero),
            0x2 => !self.flags.contains(Flags::Carry),
            0x3 => self.flags.contains(Flags::Carry),

            _ => panic!("should only be 2 bits wide at {:04x}", self.pc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_r8() {
        let mut memory = Memory::default();
        memory[0x0607 as u16] = 0xAB;

        let regs = Registers {
            flags: Flags::default(),
            a: 0x01,    // 7
            b: 0x02,    // 0
            c: 0x03,    // 1
            d: 0x04,    // 2
            e: 0x05,    // 3
            h: 0x06,    // 4
            l: 0x07,    // 5
            // hl is 0x0607 which is 6
            sp: 0x0000,
            pc: 0x0000,
        };

        assert_eq!(regs.get_r8(0, &memory), regs.b);
        assert_eq!(regs.get_r8(1, &memory), regs.c);
        assert_eq!(regs.get_r8(2, &memory), regs.d);
        assert_eq!(regs.get_r8(3, &memory), regs.e);
        assert_eq!(regs.get_r8(4, &memory), regs.h);
        assert_eq!(regs.get_r8(5, &memory), regs.l);
        assert_eq!(regs.get_r8(6, &memory), memory[0x0607 as u16]);
        assert_eq!(regs.get_r8(7, &memory), regs.a);
    }

    #[test]
    fn test_set_r8() {
        let mut regs = Registers::default();
        let mut memory = Memory::default();

        regs.set_r8(0, 0x02, &mut memory);
        regs.set_r8(1, 0x03, &mut memory);
        regs.set_r8(2, 0x04, &mut memory);
        regs.set_r8(3, 0x05, &mut memory);
        regs.set_r8(4, 0x06, &mut memory);
        regs.set_r8(5, 0x07, &mut memory);
        regs.set_r8(6, 0xAB, &mut memory);
        regs.set_r8(7, 0x01, &mut memory);

        assert_eq!(regs.b, 0x02);
        assert_eq!(regs.c, 0x03);
        assert_eq!(regs.d, 0x04);
        assert_eq!(regs.e, 0x05);
        assert_eq!(regs.h, 0x06);
        assert_eq!(regs.l, 0x07);
        assert_eq!(memory[0x0607 as u16], 0xAB);
        assert_eq!(regs.a, 0x01);
    }

    #[test]
    fn test_apply_r8() {
        let mut regs = Registers::default();
        let mut memory = Memory::default();

        regs.a = 0xAB;
        regs.apply_r8(7, &mut memory, |reg| reg + 7);
        assert_eq!(regs.a, 0xB2);
    }

    #[test]
    fn test_set_hl() {
        let mut regs = Registers::default();
        regs.set_hl(0xABCD);
        assert_eq!(regs.h, 0xAB);
        assert_eq!(regs.l, 0xCD);
    }

    #[test]
    fn test_get_hl() {
        let mut regs = Registers::default();
        regs.h = 0xAB;
        regs.l = 0xCD;
        assert_eq!(regs.get_hl(), 0xABCD)
    }

    #[test]
    fn test_get_r16() {
        let mut regs = Registers {
            a: 0xFF,
            flags: Flags::Zero | Flags::HalfCarry,
            b: 0x11,
            c: 0x22,
            d: 0x33,
            e: 0x44,
            h: 0x55,
            l: 0x66,
            sp: 0xAABB,
            pc: 0x1234,
        };

        assert_eq!(regs.get_r16(0), 0x1122);
        assert_eq!(regs.get_r16(1), 0x3344);
        assert_eq!(regs.get_r16(2), 0x5566);
        assert_eq!(regs.get_r16(3), 0xAABB);

        assert_eq!(regs.get_r16_stk(0), 0x1122);
        assert_eq!(regs.get_r16_stk(1), 0x3344);
        assert_eq!(regs.get_r16_stk(2), 0x5566);
        assert_eq!(regs.get_r16_stk(3), 0xFFA0);

        assert_eq!(regs.get_r16_mem(0), 0x1122);
        assert_eq!(regs.get_r16_mem(1), 0x3344);
        assert_eq!(regs.get_r16_mem(2), 0x5566);
        assert_eq!(regs.get_r16(2), 0x5567);
        assert_eq!(regs.get_r16_mem(3), 0x5567);
        assert_eq!(regs.get_r16(2), 0x5566);
    }

    #[test]
    fn test_set_r16() {
        let mut regs = Registers::default();

        regs.set_r16(0, 0x1122);
        regs.set_r16(1, 0x3344);
        regs.set_r16(2, 0x5566);
        regs.set_r16(3, 0x7788);

        assert_eq!(regs.b, 0x11);
        assert_eq!(regs.c, 0x22);
        assert_eq!(regs.d, 0x33);
        assert_eq!(regs.e, 0x44);
        assert_eq!(regs.h, 0x55);
        assert_eq!(regs.l, 0x66);
        assert_eq!(regs.sp, 0x7788);

        regs.set_r16_stk(0, 0xAABB);
        regs.set_r16_stk(1, 0xCCDD);
        regs.set_r16_stk(2, 0xEEFF);
        regs.set_r16_stk(3, 0x00C0);

        assert_eq!(regs.b, 0xAA);
        assert_eq!(regs.c, 0xBB);
        assert_eq!(regs.d, 0xCC);
        assert_eq!(regs.e, 0xDD);
        assert_eq!(regs.h, 0xEE);
        assert_eq!(regs.l, 0xFF);
        assert_eq!(regs.a, 0x00);
        assert_eq!(regs.flags, Flags::Zero | Flags::Negative);
    }
}