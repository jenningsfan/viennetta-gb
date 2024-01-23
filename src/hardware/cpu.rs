use bitflags::bitflags;
use crate::hardware::memory::Memory;

macro_rules! unsupported_opcode {
    ( $( $opcode:expr )+ ) => {
        panic!("Unsupported opcode: {:X}", $($opcode),+)
    };
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    struct Flags: u8 {
        const Carry = 1 << 4;
        const Zero = 1 << 7;
    }
}

#[derive(Default, Debug)]
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
    
    fn get_r8(&self, reg: u8, memory: &Memory) -> u8 {
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

    fn set_r8(&mut self, reg: u8, value: u8, memory: &mut Memory) {
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

    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0x0F) as u8;
    }

    fn get_r16(&self, reg: u8) -> u16 {
        match reg {
            0 => (self.b as u16) << 8 | self.c as u16,
            1 => (self.d as u16) << 8 | self.e as u16,
            2 => self.get_hl(),
            3 => self.sp,

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    fn set_r16(&mut self, reg: u8, value: u16) {
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

    fn get_r16_stk(&self, reg: u8) -> u16 {
        match reg {
            0 => (self.b as u16) << 8 | self.c as u16,
            1 => (self.d as u16) << 8 | self.e as u16,
            2 => self.get_hl(),
            3 => (self.a as u16) << 8 | self.flags.bits() as u16,

            _ => panic!("opcode segment should only be 3 bits wide"),
        }
    }

    fn get_r16_mem(&self, reg: u8) -> u16 {
        let hl = self.get_hl();

        match reg {
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

#[derive(Default, Debug)]
pub struct CPU {
    regs: Registers,
}

impl CPU {
    pub fn execute_opcode(&mut self, memory: &mut Memory) {
        let opcode = memory[self.regs.pc];
        let block = opcode >> 6;

        self.regs.pc += 1;
        
        match block {
            0x0 => self.execute_block_0_opcode(opcode, memory),
            0x1 => self.execute_block_1_opcode(opcode, memory),
            0x2 => self.execute_block_2_opcode(opcode, memory),
            0x3 => self.execute_block_3_opcode(opcode, memory),

            _ => panic!("opcode block should only be 2 bits wide")
        };
    }
    
    fn execute_block_0_opcode(&mut self, opcode: u8, memory: &mut Memory) {
        if opcode == 0x00 {
            return; // nop
        }

        {
            let r16 = (opcode & 0x30) >> 4;
            let imm16 = (memory[self.regs.pc + 1] as u16) << 8 | memory[self.regs.pc] as u16;

            match opcode & 0xF {
                0x1 => {
                    // ld r16, imm16
                    self.regs.set_r16(r16, imm16);
                    self.regs.pc += 2;
                    return;
                },
                0x2 => {
                    // ld [r16mem], a
                    let addr = self.regs.get_r16_mem(r16);
                    memory[addr] = self.regs.a;
                    return;
                },
                0xA => {
                    // ld a, [r16mem]
                    let addr = self.regs.get_r16_mem(r16);
                    self.regs.a = memory[addr];
                    return;
                },

                0x3 => {
                    // inc r16
                    self.regs.set_r16(r16, self.regs.get_r16(r16) + 1);
                    return;
                },
                0xB => {
                    // dec r16
                    self.regs.set_r16(r16, self.regs.get_r16(r16) - 1);
                    return;
                },
                0x9 => {
                    // add hl, r16
                    self.regs.set_hl(self.regs.get_hl() + self.regs.get_r16(r16));
                    return;
                },
                _ => {},
            };

            if opcode == 0x08 {
                // ld [imm16], sp
                memory[imm16] = (self.regs.sp & 0xFF) as u8;
                memory[imm16 + 1] = (self.regs.sp >> 8) as u8;
                self.regs.pc += 2;
                return;
            }
        }

        {
            let r8 = (opcode & 0x30) >> 4;
            match opcode & 0x3 {
                0x4 => {
                    // inc r8
                    self.regs.set_r8(r8, self.regs.get_r8(r8, memory) + 1, memory);
                    return;
                },
                0x5 => {
                    // dec r8
                    self.regs.set_r8(r8, self.regs.get_r8(r8, memory) - 1, memory);
                    return;
                },
                0x6 => {
                    // ld r8, imm8
                    self.regs.set_r8(r8, memory[self.regs.pc], memory);
                    self.regs.pc += 1;
                    return;
                },
                _ => {},
            };
        }
        
        match opcode {
            0x07 => {
                return;
            },
            0x0F => {
                return;
            },
            0x17 => {
                return;
            },
            0x1F => {
                return;
            },
            0x27 => {
                return;
            },
            0x2F => {
                return;
            },
            0x37 => {
                return;
            },
            0x3F => {
                return;
            },
            _ => {},
        };

        if opcode == 0x18 {
            return;
        }

        if opcode & 0x27 == 0x20 {
            return;
        }

        if opcode == 0x10 {
            return;
        }

        panic!("Invalid opcode: {opcode}");
    }

    fn execute_block_1_opcode(&mut self, opcode: u8, memory: &mut Memory) {
        if opcode == 0x76 {
            // TODO: halt opcode
            return;
        }

        let source_reg = opcode & 0x07;
        let dest_reg = (opcode >> 3) & 0x07;

        self.regs.set_r8(dest_reg, self.regs.get_r8(source_reg, memory), memory);
    }

    fn execute_block_2_opcode(&mut self, opcode: u8, memory: &mut Memory) {
        // All 8-bit arithmetic
        let operand = self.regs.get_r8(opcode & 0x3, memory);

        match opcode & 0xF8 {
            0x80 => self.regs.add_acc(operand),
            0x88 => {
                self.regs.add_acc(operand);
                self.regs.add_acc((self.regs.flags & Flags::Carry).bits());
            },
            0x90 => self.regs.sub_acc(operand),
            0x98 => {
                self.regs.sub_acc(operand);
                self.regs.sub_acc((self.regs.flags & Flags::Carry).bits());
            },
            0xA0 => self.regs.a &= operand,
            0xA8 => self.regs.a ^= operand,
            0xB0 => self.regs.a |= operand,
            0xB8 => {},

            _ => unsupported_opcode!(opcode),
        }
        
        if self.regs.a == 0 {
            self.regs.flags |= Flags::Zero;
        }
        else {
            self.regs.flags -= Flags::Zero;
        }
    }

    fn execute_block_3_opcode(&mut self, opcode: u8, memory: &mut Memory) {
        
    }

    fn execute_cb_opcode(&mut self, memory: &mut Memory) {

    }
}