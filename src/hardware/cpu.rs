mod registers;

use crate::hardware::memory::Memory;
use registers::*;

macro_rules! unsupported_opcode {
    ( $( $opcode:expr )+, $( $pc:expr )+ ) => {
        panic!("{:02X} is not supported at {:04X}", $($opcode),+, $($pc),+)
    };
}

#[derive(Default, Debug)]
pub struct CPU {
    pub regs: Registers,
    interrupts_enabled: bool,
}

impl CPU {
    pub fn execute_opcode(&mut self, memory: &mut Memory) -> u8 {
        let opcode = memory[self.regs.pc];
        let block = opcode >> 6;
        
        self.regs.pc += 1;
        
        //self.dump_regs();
        let cycles = match block {
            0x0 => self.execute_block_0_opcode(opcode, memory),
            0x1 => self.execute_block_1_opcode(opcode, memory),
            0x2 => self.execute_block_2_opcode(opcode, memory),
            0x3 => self.execute_block_3_opcode(opcode, memory),

            _ => panic!("opcode block should only be 2 bits wide")
        };

        //dbg!(&self.regs);
        //dbg!(self.regs.a);
        cycles
    }

    pub fn dump_regs(&self) {
        let mut flags_str = String::new();
        for &flag in &[Flags::Carry, Flags::HalfCarry, Flags::Negative, Flags::Zero] {
            flags_str.push(match self.regs.flags.contains(flag) {
                true => flag.to_char(),
                false => '-',
            });
        }

        println!("AF: {:02x}{:02x} ({flags_str})", self.regs.a, self.regs.flags);
        println!("BC: {:02x}{:02x}", self.regs.b, self.regs.c);
        println!("DE: {:02x}{:02x}", self.regs.d, self.regs.e);
        println!("HL: {:02x}{:02x}", self.regs.h, self.regs.l);
        println!("SP: {:04x}", self.regs.sp);
        println!("PC: {:04x} ", self.regs.pc);
    }

    fn execute_block_0_opcode(&mut self, opcode: u8, memory: &mut Memory) -> u8 {
        if opcode == 0x00 {
            return 1; // nop
        }

        {
            let r16 = (opcode >> 4) & 0x03;
            let imm16 = (memory[self.regs.pc + 1] as u16) << 8 | memory[self.regs.pc] as u16;

            match opcode & 0xF {
                0x1 => {
                    // ld r16, imm16
                    self.regs.set_r16(r16, imm16);
                    self.regs.pc += 2;
                    return 3;
                },
                0x2 => {
                    // ld [r16mem], a
                    let addr = self.regs.get_r16_mem(r16);
                    memory[addr] = self.regs.a;
                    return 2;
                },
                0xA => {
                    // ld a, [r16mem]
                    let addr = self.regs.get_r16_mem(r16);
                    self.regs.a = memory[addr];
                    return 2;
                },

                0x3 => {
                    // inc r16
                    self.regs.apply_r16(r16, |r| r + 1);
                    return 2;
                },
                0xB => {
                    // dec r16
                    self.regs.apply_r16(r16, |r| r - 1);
                    return 2;
                },
                0x9 => {
                    // add hl, r16
                    let hl = self.regs.get_hl();
                    let r16 = self.regs.get_r16(r16);

                    self.regs.flags = self.regs.flags & Flags::Zero;
                    if hl.overflowing_add(r16).1 {
                        self.regs.flags |= Flags::Carry;
                    }
                    if ((hl & 0x7FF) + (r16 & 0x7FF)) & 0x800 == 0x800 {
                        self.regs.flags |= Flags::HalfCarry;
                    }

                    self.regs.set_hl(hl.wrapping_add(r16));
                    return 2;
                },

                _ => {},
            };

            if opcode == 0x08 {
                // ld [imm16], sp
                memory[imm16] = (self.regs.sp & 0xFF) as u8;
                memory[imm16 + 1] = (self.regs.sp >> 8) as u8;
                self.regs.pc += 2;
                return 5;
            }
        }

        {
            let r8 = (opcode >> 3) & 0x07;
            const HL_POINT: u8 = 6;

            match opcode & 0x7 {
                0x4 => {
                    // inc r8
                    //dbg!(self.regs.get_r8(r8, memory));
                    self.regs.add_r8(r8, 1, memory, false);
                    //dbg!(self.regs.get_r8(r8, memory));
                    //dbg!(self.regs.flags);
                    if r8 == HL_POINT {
                        return 3;
                    }
                    else {
                        return 1;
                    }
                },
                0x5 => {
                    // dec r8
                    self.regs.sub_r8(r8, 1, memory, false);
                    if r8 == HL_POINT {
                        return 3;
                    }
                    else {
                        return 1;
                    }
                },
                0x6 => {
                    // ld r8, imm8
                    self.regs.set_r8(r8, memory[self.regs.pc], memory);
                    self.regs.pc += 1;
                    if r8 == HL_POINT {
                        return 3;
                    }
                    else {
                        return 2;
                    }
                },

                _ => {},
            };
        }
        
        match opcode {
            0x07 => {
                // rlca
                self.regs.flags = Flags::default();
                if self.regs.a & 0x80 == 0x80 {
                    self.regs.flags |= Flags::Carry;
                }

                self.regs.a = self.regs.a.rotate_left(1);

                return 1;
            },
            0x0F => {
                // rrca
                self.regs.flags = Flags::default();
                if self.regs.a & 0x01 == 1 {
                    self.regs.flags |= Flags::Carry;
                }
                
                self.regs.a = self.regs.a.rotate_right(1);

                return 1;
            },
            0x17 => {
                // rla
                let mut shifted = self.regs.a << 1;
                if self.regs.flags.contains(Flags::Carry) {
                    shifted |= 1;
                    self.regs.flags = Flags::default();
                }
                if self.regs.a & 0x80 == 0x80 {
                    self.regs.flags = Flags::Carry
                }
                self.regs.a = shifted;

                return 1;     
            },
            0x1F => {
                // rra
                let mut shifted = self.regs.a >> 1;
                if self.regs.flags.contains(Flags::Carry) {
                    shifted |= 0x80;
                    self.regs.flags = Flags::default();
                }
                if self.regs.a & 0x01 == 1 {
                    self.regs.flags = Flags::Carry
                }
                self.regs.a = shifted;

                return 1;
            },
            0x27 => {
                // daa
                let mut offset = 0;
                let mut should_carry = false;
                let negative = self.regs.flags.contains(Flags::Negative);
                if (!negative && self.regs.a & 0xF > 0x09) || self.regs.flags.contains(Flags::HalfCarry) {
                    offset |= 0x06;
                }
                if (!negative && self.regs.a > 0x99) || self.regs.flags.contains(Flags::Carry) {
                    offset |= 0x60;
                    should_carry = true;
                }

                if negative {
                    self.regs.a = self.regs.a.wrapping_sub(offset);
                }
                else {
                    self.regs.a = self.regs.a.wrapping_add(offset);
                }
                
                self.regs.flags = self.regs.flags & (Flags::Negative );//| Flags::Carry);
                if self.regs.a == 0 {
                    self.regs.flags |= Flags::Zero;
                }
                if !negative && should_carry {
                    self.regs.flags |= Flags::Carry;
                }

                return 1;
            },
            0x2F => {
                // cpl
                self.regs.a = !self.regs.a;
                self.regs.flags |= Flags::Negative;
                self.regs.flags |= Flags::HalfCarry;
                return 1;
            },
            0x37 => {
                // scf
                self.regs.flags |= Flags::Carry;
                self.regs.flags -= Flags::HalfCarry;
                self.regs.flags -= Flags::Negative;
                return 1;
            },
            0x3F => {
                // ccf
                self.regs.flags ^= Flags::Carry;
                self.regs.flags -= Flags::HalfCarry;
                self.regs.flags -= Flags::Negative;
                return 1;
            },

            _ => {},
        };

        if opcode == 0x18 {
            // jr imm8
            let offset = memory[self.regs.pc] as i8 as i16 + 1;
            self.regs.pc = (self.regs.pc as i16 + offset) as u16;

            return 3;
        }

        if opcode & 0x27 == 0x20 {
            // jr cond, imm8
            let condition = self.regs.condition((opcode & 0x18) >> 3);
            if condition {
                let offset = memory[self.regs.pc] as i8 as i16 + 1;
                self.regs.pc = (self.regs.pc as i16 + offset) as u16;

                return 3;
            }
            else {
                self.regs.pc += 1; // imm8
                return 2;
            }
        }

        if opcode == 0x10 {
            // stop
            todo!("Stop opcode");
            return 1;
        }

        unsupported_opcode!(opcode, self.regs.pc);
    }

    fn execute_block_1_opcode(&mut self, opcode: u8, memory: &mut Memory) -> u8 {
        if opcode == 0x76 {
            // TODO: halt opcode
            todo!("Halt opcode");
            return 1;
        }

        let source_reg = opcode & 0x07;
        let dest_reg = (opcode >> 3) & 0x07;

        self.regs.set_r8(dest_reg, self.regs.get_r8(source_reg, memory), memory);

        // check if [hl] is being used
        if source_reg == 6 || dest_reg == 6 {
            return 2;
        }
        else {
            return 1;
        }
    }

    fn execute_block_2_opcode(&mut self, opcode: u8, memory: &mut Memory) -> u8 {
        // All 8-bit arithmetic
        let operand = self.regs.get_r8(opcode & 0x7, memory);

        // println!("operand: {operand:02X}");
        // println!("a: {:02X}", self.regs.a);

        match opcode & 0xF8 {
            0x80 => self.regs.add_acc(operand), // add a, r8
            0x88 => {
                // adc a, r8
                let carry = if self.regs.flags.contains(Flags::Carry) { 1 } else { 0 };
                self.regs.add_acc(operand + carry);
                
                if carry == 1 && operand == 0xFF {
                    self.regs.flags |= Flags::Carry;
                }
                if carry == 1 && operand & 0xF == 0xF {
                    self.regs.flags |= Flags::HalfCarry;
                }
            },
            0x90 => self.regs.sub_acc(operand), // sub a, r8
            0x98 => {
                // subc a, r8
                let carry = if self.regs.flags.contains(Flags::Carry) { 1 } else { 0 };
                self.regs.sub_acc(operand + carry);

                if carry == 1 && operand == 0xFF {
                    self.regs.flags |= Flags::Carry;
                }
                if carry == 1 && operand & 0xF == 0xF {
                    self.regs.flags |= Flags::HalfCarry;
                }
            },
            0xA0 => {
                // and a, r8
                self.regs.a &= operand;

                self.regs.flags -= Flags::Negative;
                self.regs.flags |= Flags::HalfCarry;
                self.regs.flags -= Flags::Carry;
                if self.regs.a == 0 && opcode != 0xB8 {
                    self.regs.flags |= Flags::Zero;
                }
                else {
                    self.regs.flags -= Flags::Zero;
                }
            },
            0xA8 => {
                // xor a, r8
                self.regs.a ^= operand;

                self.regs.flags -= Flags::Negative;
                self.regs.flags -= Flags::HalfCarry;
                self.regs.flags -= Flags::Carry;
                if self.regs.a == 0 && opcode != 0xB8 {
                    self.regs.flags |= Flags::Zero;
                }
                else {
                    self.regs.flags -= Flags::Zero;
                }
            },
            0xB0 => {
                // or a, r8
                self.regs.a |= operand;

                self.regs.flags -= Flags::Negative;
                self.regs.flags -= Flags::HalfCarry;
                self.regs.flags -= Flags::Carry;
                if self.regs.a == 0 && opcode != 0xB8 {
                    self.regs.flags |= Flags::Zero;
                }
                else {
                    self.regs.flags -= Flags::Zero;
                }
            }
            0xB8 => {
                // cp a, r8
                let old_a = self.regs.a;
                self.regs.sub_acc(operand);
                self.regs.a = old_a;
            },

            _ => unsupported_opcode!(opcode, self.regs.pc),
        }

        // check for [hl]
        if operand == 6 {
            return 2;
        }
        else {
            return 1;
        }
    }

    fn execute_block_3_opcode(&mut self, opcode: u8, memory: &mut Memory) -> u8 {
        let imm8 = memory[self.regs.pc];
        let imm16 = (memory[self.regs.pc + 1] as u16) << 8 | (memory[self.regs.pc] as u16);
        let condition = self.regs.condition((opcode & 0x18) >> 3);

        self.regs.pc += 1;
        match opcode {
            0xC6 => {
                // add a, imm8
                self.regs.add_acc(imm8);
                return 2;
            },
            0xCE => {
                // adc a, imm8
                let carry = if self.regs.flags.contains(Flags::Carry) { 1 } else { 0 };
                self.regs.add_acc(imm8 + carry);
                if carry == 1 && imm8 == 0xFF {
                    self.regs.flags |= Flags::Carry;
                }
                if carry == 1 && imm8 & 0xF == 0xF {
                    self.regs.flags |= Flags::HalfCarry;
                }

                return 2;
            },
            0xD6 => {
                // sub a, imm8
                self.regs.sub_acc(imm8);
                return 2;
            },
            0xDE => {
                // subc a, imm8
                let carry = if self.regs.flags.contains(Flags::Carry) { 1 } else { 0 };
                self.regs.sub_acc(imm8 + carry);
                if carry == 1 && imm8 == 0xFF {
                    self.regs.flags |= Flags::Carry;
                }
                if carry == 1 && imm8 & 0xF == 0xF {
                    self.regs.flags |= Flags::HalfCarry;
                }
                return 2;
            },
            0xE6 => {
                // and a, imm8
                self.regs.a &= imm8;
                self.regs.flags -= Flags::Negative;
                self.regs.flags |= Flags::HalfCarry;
                self.regs.flags -= Flags::Carry;
                if self.regs.a == 0 {
                    self.regs.flags |= Flags::Zero;
                }
                else {
                    self.regs.flags -= Flags::Zero;
                }
                return 2;
            }
            0xEE => {
                // xor a, imm8
                self.regs.a ^= imm8;
                self.regs.flags -= Flags::Negative;
                self.regs.flags -= Flags::HalfCarry;
                self.regs.flags -= Flags::Carry;
                if self.regs.a == 0 {
                    self.regs.flags |= Flags::Zero;
                }
                else {
                    self.regs.flags -= Flags::Zero;
                }
                return 2;
            } 
            0xF6 => {
                // or a, imm8
                self.regs.a |= imm8;
                self.regs.flags -= Flags::Negative;
                self.regs.flags -= Flags::HalfCarry;
                self.regs.flags -= Flags::Carry;
                if self.regs.a == 0 {
                    self.regs.flags |= Flags::Zero;
                }
                else {
                    self.regs.flags -= Flags::Zero;
                }
                return 2;
            }
            0xFE => {
                // cp a, imm8
                let old_a = self.regs.a;
                self.regs.sub_acc(imm8);
                self.regs.a = old_a;
                return 2;
            },

            _ => {},
        }
        self.regs.pc -= 1; // this is to counteract the increase because it wasn't needed

        if opcode == 0xC9 {
            // ret
            self.regs.pc = self.pop_from_stack(memory);
            return 4;
        }

        if opcode == 0xD9 {
            // reti
            self.regs.pc = self.pop_from_stack(memory);
            self.interrupts_enabled = true;
            return 4;
        }

        if opcode & 0x27 == 0x00 {
            // ret cond
            if condition {
                self.regs.pc = self.pop_from_stack(memory);
                return 5;
            }
            else {
                return 2;
            }
        }

        if opcode & 0x27 == 0x02 {
            // jp cond, imm16
            if condition {
                self.regs.pc = imm16;
                return 4;
            }
            else {
                self.regs.pc += 2;
                return 3;
            }
        }
        
        if opcode == 0xC3 {
            // jp imm16
            self.regs.pc = imm16;
            return 4;
        }

        if opcode == 0xE9 {
            // jp hl
            self.regs.pc = self.regs.get_hl();
            return 1;
        }

        if opcode == 0xCD {
            // call imm16
            self.push_to_stack(self.regs.pc + 2, memory);
            self.regs.pc = imm16;
            return 6;
        }

        if opcode & 0x07 == 0x04 {
            if condition {
                self.push_to_stack(self.regs.pc + 2, memory);
                self.regs.pc = imm16;
                return 24;
            }
            else {
                self.regs.pc += 2;
                return 12;
            }
        }

        if opcode & 0x07 == 0x07 {
            // rst tgst3
            let target = (opcode & 0x38) as u16;
            self.push_to_stack(self.regs.pc, memory);
            self.regs.pc = target;
            return 4;
        }

        let r16 = (opcode & 0x30) >> 4;
        match opcode & 0x0F {
            0x01 => {
                // pop r16stk
                //println!("{:02X} at {:04X}", opcode, self.regs.pc);
                let value = self.pop_from_stack(memory);
                self.regs.set_r16_stk(r16, value);
                if opcode == 0xF1 {
                    //println!("pop af( {:02X}) at {:04X}", value, self.regs.pc);
                }
                return 3;
            },
            0x05 => {
                // push r16stk
                if opcode == 0xF5 {
                    //println!("push af( {:02X}) at {:04X}", self.regs.get_r16_stk(r16), self.regs.pc);
                }
                
                self.push_to_stack(self.regs.get_r16_stk(r16), memory);
                return 4;
            },
            _ => {},
        }

        match opcode {
            0xE2 => {
                // ldh [c], a
                memory[0xFF00 + self.regs.c as u16] = self.regs.a;
                return 2;
            },
            0xE0 => {
                // ldh [imm8], a
                memory[0xFF00 + imm8 as u16] = self.regs.a;
                self.regs.pc += 1;
                return 3;
            },
            0xEA => {
                // ld [imm16], a
                memory[imm16] = self.regs.a;
                self.regs.pc += 2;
                return 4;
            },
            0xF2 => {
                // ldh a, [c]
                self.regs.a = memory[0xFF00 + self.regs.c as u16];
                return 2;
            },
            0xF0 => {
                // ldh a, [imm8]
                self.regs.a = memory[0xFF00 + imm8 as u16];
                self.regs.pc += 1;
                return 3;
            },
            0xFA => {
                // ld a, [imm16]
                self.regs.a = memory[imm16];
                self.regs.pc += 2;
                return 4;
            },
            0xE8 => {
                // add sp, imm8
                let offset = (imm8 as i8) as i16;
                let result = (self.regs.sp as i16).overflowing_add(offset);
                
                if result.1 {
                    self.regs.flags |= Flags::Carry;
                }
                if (((self.regs.sp as i16 & 0xF) + (offset & 0xF)) as u16 & 0x10) == 0x10 {
                    self.regs.flags |= Flags::HalfCarry;
                }

                self.regs.sp = result.0 as u16;
                self.regs.pc += 1;
                return 4;
            },
            0xF8 => {
                // ld hl, sp + imm8
                let offset = (imm8 as i8) as i16;
                let result = (self.regs.sp as i16).overflowing_add(offset);
                
                if result.1 {
                    self.regs.flags |= Flags::Carry;
                }
                if (((self.regs.sp as i16 & 0xF) + (offset & 0xF)) as u16 & 0x10) == 0x10 {
                    self.regs.flags |= Flags::HalfCarry;
                }

                self.regs.set_hl(result.0 as u16);
                self.regs.pc += 1;
                return 3;
            },
            0xF9 => {
                // ld sp, hl
                self.regs.sp = self.regs.get_hl();
                return 2;
            },
            0xF3 => {
                // di
                self.interrupts_enabled = false;
                return 1;
            },
            0xFB => {
                // ei
                self.interrupts_enabled = true;
                return 1;
            },
            0xCB => {
                let cycles = self.execute_cb_opcode(memory[self.regs.pc], memory);
                self.regs.pc += 1;
                return cycles;
            },
            _ => {},
        }

        unsupported_opcode!(opcode, self.regs.pc);
    }

    fn execute_cb_opcode(&mut self, opcode: u8, memory: &mut Memory) -> u8 {
        let reg = opcode & 0x07;
        let bit = 1 << ((opcode & 0x38) >> 3);
        
        match opcode & 0xC0 {
            0x00 => {
                match opcode & 0xF8 {
                    0x00 => {
                        // rlc r8
                        let mut value = self.regs.get_r8(reg, memory);
                        self.regs.flags = Flags::default();

                        if value & 0x80 == 0x80 {
                            self.regs.flags |= Flags::Carry;
                        }
                        value = value.rotate_left(1);
                        self.regs.set_r8(reg, value, memory);
                        if value == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                        return 2;
                    },
                    0x08 => {
                        // rrc r8
                        let mut value = self.regs.get_r8(reg, memory);
                        self.regs.flags = Flags::default();
                        if value & 0x01 == 1 {
                            self.regs.flags |= Flags::Carry;
                        }
                        
                        value = value.rotate_right(1);
                        self.regs.set_r8(reg, value, memory);
                        if value == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                        return 2;
                    },
                    0x10 => {
                        // rl r8
                        let value = self.regs.get_r8(reg, memory);
                        
                        let mut shifted = value << 1;
                        
                        if self.regs.flags.contains(Flags::Carry) {
                            shifted |= 1;
                        }
                        self.regs.flags = Flags::default();
                        if value & 0x80 == 0x80 {
                            self.regs.flags |= Flags::Carry
                        }

                        self.regs.set_r8(reg, shifted, memory);
                        if shifted == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                        return 2;
                    },
                    0x18 => {
                        // rr r8
                        let value = self.regs.get_r8(reg, memory);
                        
                        let mut shifted = value >> 1;
                        
                        if self.regs.flags.contains(Flags::Carry) {
                            shifted |= 0x80;
                        }
                        self.regs.flags = Flags::default();
                        if value & 0x01 == 1 {
                            self.regs.flags |= Flags::Carry;
                        }
                        
                        self.regs.set_r8(reg, shifted, memory);
                        if shifted == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                        return 2;
                    },
                    0x20 => {
                        // sla r8
                        self.regs.flags = Flags::default();
                        if self.regs.get_r8(reg, memory) & 0x80 == 0x80 {
                            self.regs.flags |= Flags::Carry;
                        }
                        self.regs.apply_r8(reg, memory, |r| r << 1);
                        if self.regs.get_r8(reg, memory) == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                        return 2;
                    },
                    0x28 => {
                        // sra r8
                        self.regs.flags = Flags::default();
                        if self.regs.get_r8(reg, memory) & 0x01 == 0x01 {
                            self.regs.flags |= Flags::Carry;
                        }
                        self.regs.apply_r8(reg, memory, |r| ((r as i8) >> 1) as u8);
                        if self.regs.get_r8(reg, memory) == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                        return 2;
                    },
                    0x30 => {
                        // swap r8
                        self.regs.apply_r8(reg, memory, |r| ((r & 0x0F) << 4) | (r >> 4));
                        if self.regs.get_r8(reg, memory) == 0 {
                            self.regs.flags = Flags::Zero;
                        }
                        else {
                            self.regs.flags = Flags::default();
                        }
                        return 13;
                    }
                    0x38 => {
                        // srl r8
                        self.regs.flags = Flags::default();
                        if self.regs.get_r8(reg, memory) & 0x01 == 0x01 {
                            self.regs.flags |= Flags::Carry;
                        }
                        self.regs.apply_r8(reg, memory, |r| r >> 1);
                        if self.regs.get_r8(reg, memory) == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                        return 13;
                    },

                    _ => {},
                }
            },
            0x40 => {
                // bit b3, r8
                self.regs.flags &= Flags::Carry;
                self.regs.flags |= Flags::HalfCarry;

                if self.regs.get_r8(reg, memory) & bit == 0 {
                    self.regs.flags |= Flags::Zero;
                }
                return 13;
            },
            0x80 => {
                // res b3, r8
                self.regs.apply_r8(reg, memory, |reg| reg & !bit);
                return 13;
            },
            0xC0 => {
                // set b3, r8
                self.regs.apply_r8(reg, memory, |reg| reg | bit);
                return 13;
            },
            _ => {},
        }

        unsupported_opcode!(0xCB00 | opcode as u16, self.regs.pc);
    }

    fn push_to_stack(&mut self, value: u16, memory: &mut Memory) {
        memory[self.regs.sp - 1] = (value >> 8) as u8;
        memory[self.regs.sp - 2] = (value & 0xFF) as u8;
        self.regs.sp -= 2;
    }

    fn pop_from_stack(&mut self, memory: &mut Memory) -> u16 {
        let value = (memory[self.regs.sp + 1] as u16) << 8 | memory[self.regs.sp] as u16;
        self.regs.sp += 2;
        value
    }
}