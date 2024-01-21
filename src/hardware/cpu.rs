use crate::hardware::ram::RAM;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct Flags: u8 {
        const Carry = 1 << 4;
        const Zero = 1 << 7;
    }
}

struct Registers {
    flags: Flags,

    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,  
    h: u8,
    l: u8,
    
    sp: u16,
    pc: u16,
}

impl Registers {
    fn default() -> Self {
        Self {
            flags: Flags::empty(),

            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,  
            h: 0,
            l: 0,
        
            sp: 0,
            pc: 0,
        }
    }
    
    fn get_r8(&self, opcode: u8, ram: &RAM) -> u8 {
        match opcode {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => ram[self.get_hl()],
            7 => self.a,
            _ => panic!("literaly impossible. it should only be 3 bits wide"),
        }
    }

    fn set_r8(&mut self, opcode: u8, value: u8, ram: &mut RAM) {
        match opcode {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => ram[self.get_hl()] = value,
            7 => self.a = value,

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    fn get_r16(&self, opcode: u8) -> u16 {
        match opcode {
            0 => (self.b as u16) << 8 | self.c as u16,
            1 => (self.d as u16) << 8 | self.e as u16,
            2 => self.get_hl(),
            3 => self.sp,

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    fn get_r16_stk(&self, opcode: u8) -> u16 {
        match opcode {
            0 => (self.b as u16) << 8 | self.c as u16,
            1 => (self.d as u16) << 8 | self.e as u16,
            2 => self.get_hl(),
            3 => (self.a as u16) << 8 | self.flags.bits() as u16,

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    fn get_r16_mem(&self, opcode: u8) -> u16 {
        let hl = self.get_hl();

        match opcode & 0x3 {
            0 => (self.b as u16) << 8 | self.c as u16,
            1 => (self.d as u16) << 8 | self.e as u16,
            2 => hl + 1,
            3 => hl - 1,

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    fn add_acc(&mut self, value: u8) {
        let result = self.a.overflowing_add(value);
        self.a = result.0;

        if result.1 {
            self.flags |= Flags::Carry;
        }
        else {
            self.flags -= Flags::Carry;
        }
    }

    fn sub_acc(&mut self, value: u8) {
        let result = self.a.overflowing_sub(value);
        self.a = result.0;

        if result.1 {
            self.flags |= Flags::Carry;
        }
        else {
            self.flags -= Flags::Carry;
        }
    }
}

pub struct CPU {
    regs: Registers,
}

impl CPU {
    fn execute_opcode(&mut self, ram: &mut RAM) {
        let opcode = ram[self.regs.pc];
        let block = opcode >> 6;

        match block {
            0x0 => self.execute_block_0_opcode(opcode, ram),
            0x1 => self.execute_block_1_opcode(opcode, ram),
            0x2 => self.execute_block_2_opcode(opcode, ram),
            0x3 => self.execute_block_3_opcode(opcode, ram),

            _ => panic!("opcode block should only be 2 bits wide")
        };
    }
    
    fn execute_block_0_opcode(&mut self, opcode: u8, ram: &mut RAM) {

    }

    fn execute_block_1_opcode(&mut self, opcode: u8, ram: &mut RAM) {
        
    }

    fn execute_block_2_opcode(&mut self, opcode: u8, ram: &mut RAM) {
        // All 8-bit arithmetic
        let operand = self.regs.get_r8(opcode & 0x3, ram);

        match opcode & 0xF8 {
            0x80 => self.regs.add_acc(operand),
            0x84 => {
                self.regs.add_acc(operand);
                self.regs.add_acc((self.regs.flags & Flags::Carry).bits());
            },
            0x88 => self.regs.sub_acc(operand),
            0x8c => {
                self.regs.sub_acc(operand);
                self.regs.sub_acc((self.regs.flags & Flags::Carry).bits());
            },
            0x90 => self.regs.a &= operand,
            0x94 => self.regs.a ^= operand,
            0x98 => self.regs.a |= operand,
            0x9c => {},

            _ => panic!("{opcode} is not supported")
        }
        
        if self.regs.a == 0 {
            self.regs.flags |= Flags::Zero;
        }
        else {
            self.regs.flags -= Flags::Zero;
        }
    }

    fn execute_block_3_opcode(&mut self, opcode: u8, ram: &mut RAM) {
        
    }

    fn execute_cb_opcode(&mut self, ram: &mut RAM) {

    }
}