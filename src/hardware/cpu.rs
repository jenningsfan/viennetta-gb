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
    regs: Registers,
    interrupts_enabled: bool,
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
            let r16 = (opcode >> 4) & 0x03;
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
                    self.regs.apply_r16(r16, |r| r + 1);
                    return;
                },
                0xB => {
                    // dec r16
                    self.regs.apply_r16(r16, |r| r - 1);
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
            let r8 = (opcode >> 4) & 0x03;
            
            match opcode & 0x7 {
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
                // rlca
                self.regs.flags = Flags::default();
                if self.regs.a & 0x80 == 1 {
                    self.regs.flags |= Flags::Carry;
                }

                self.regs.a = self.regs.a.rotate_left(1);

                return;
            },
            0x0F => {
                // rrca
                self.regs.flags = Flags::default();
                if self.regs.a & 0x01 == 1 {
                    self.regs.flags |= Flags::Carry;
                }
                
                self.regs.a = self.regs.a.rotate_right(1);

                return;
            },
            0x17 => {
                // rla
                let mut shifted = self.regs.a << 1;
                if self.regs.flags.contains(Flags::Carry) {
                    shifted |= 1;
                }
                if self.regs.a & 0x80 == 1 {
                    self.regs.flags |= Flags::Carry
                }
                self.regs.a = shifted;

                return;     
            },
            0x1F => {
                // rra
                let mut shifted = self.regs.a >> 1;
                if self.regs.flags.contains(Flags::Carry) {
                    shifted |= 0x80;
                }
                if self.regs.a & 0x01 == 1 {
                    self.regs.flags |= Flags::Carry
                }
                self.regs.a = shifted;

                return;
            },
            0x27 => {
                // daa
                todo!("DAA instruction");
                return;
            },
            0x2F => {
                // cpl
                self.regs.a = !self.regs.a;
                return;
            },
            0x37 => {
                // scf
                self.regs.flags |= Flags::Carry;
                return;
            },
            0x3F => {
                // ccf
                self.regs.flags ^= Flags::Carry;
                return;
            },

            _ => {},
        };

        if opcode == 0x18 {
            // jr imm8
            let offset = memory[self.regs.pc] as i16;
            self.regs.pc = (self.regs.pc as i16 + offset) as u16;

            return;
        }

        if opcode & 0x27 == 0x20 {
            // jr cond, imm8
            let condition = self.regs.condition((opcode & 0x18) >> 3);
            if condition {
                let offset = memory[self.regs.pc] as i16;
                self.regs.pc = (self.regs.pc as i16 + offset) as u16;
            }
            else {
                self.regs.pc += 1; // imm8
            }

            return;
        }

        if opcode == 0x10 {
            // stop
            todo!("Stop opcode");
            return;
        }

        unsupported_opcode!(opcode, self.regs.pc);
    }

    fn execute_block_1_opcode(&mut self, opcode: u8, memory: &mut Memory) {
        if opcode == 0x76 {
            // TODO: halt opcode
            todo!("Halt opcode");
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
            0x80 => self.regs.add_acc(operand), // add a, r8
            0x88 => {
                // adc a, r8
                self.regs.add_acc(operand);
                self.regs.add_acc((self.regs.flags & Flags::Carry).bits());
            },
            0x90 => self.regs.sub_acc(operand), // sub a, r8
            0x98 => {
                // subc a, r8
                self.regs.sub_acc(operand);
                self.regs.sub_acc((self.regs.flags & Flags::Carry).bits());
            },
            0xA0 => self.regs.a &= operand, // and a, r8
            0xA8 => self.regs.a ^= operand, // xor a, r8
            0xB0 => self.regs.a |= operand, // or a, r8
            0xB8 => {
                // cp a, r8
                let old_a = self.regs.a;
                self.regs.sub_acc(operand);
                self.regs.a = old_a;
            },

            _ => unsupported_opcode!(opcode, self.regs.pc),
        }
        
        if self.regs.a == 0 {
            self.regs.flags |= Flags::Zero;
        }
        else {
            self.regs.flags -= Flags::Zero;
        }
    }

    fn execute_block_3_opcode(&mut self, opcode: u8, memory: &mut Memory) {
        let imm8 = memory[self.regs.pc];
        let imm16 = (memory[self.regs.pc + 1] as u16) >> 8 | (memory[self.regs.pc] as u16);
        let condition = self.regs.condition((opcode & 0x18) >> 3);

        self.regs.pc += 1;
        match opcode {
            0xC6 => {
                // add a, r8
                self.regs.add_acc(imm8);
                return;
            },
            0xCE => {
                // adc a, r8
                self.regs.add_acc(imm8);
                self.regs.add_acc((self.regs.flags & Flags::Carry).bits());
                return;
            },
            0xD6 => {
                // sub a, r8
                self.regs.sub_acc(imm8);
                return;
            },
            0xDE => {
                // subc a, r8
                self.regs.sub_acc(imm8);
                self.regs.sub_acc((self.regs.flags & Flags::Carry).bits());
                return;
            },
            0xE6 => {
                // and a, r8
                self.regs.a &= imm8;
                return;
            }
            0xEE => {
                // xor a, r8
                self.regs.a ^= imm8;
                return;
            } 
            0xF6 => {
                // or a, r8
                self.regs.a |= imm8;
                return;
            }
            0xFE => {
                // cp a, r8
                let old_a = self.regs.a;
                self.regs.sub_acc(imm8);
                self.regs.a = old_a;
                return;
            },

            _ => {},
        }
        self.regs.pc -= 1; // this is to counteract the increase because it wasn't needed

        if (opcode & 0x27 == 0x00 && condition) || opcode == 0xC9 || opcode == 0xD9 {
            // ret cond, ret, reti
            self.regs.pc = self.pop_from_stack(memory);

            if opcode == 0xD9 {
                self.interrupts_enabled = true;
                return;
            }

            return;
        }

        if opcode & 0x27 == 0x02 {
            // jp cond, imm16
            if condition {
                self.regs.pc = imm16;
            }
            else {
                self.regs.pc += 2;
            }
            
            return;
        }
        
        if opcode == 0xC3 {
            // jp imm16
            self.regs.pc = imm16;
            return;
        }

        if opcode == 0xE9 {
            // jp hl
            self.regs.pc = self.regs.get_hl();
            return;
        }

        if opcode == 0xCD {
            // call imm16
            self.push_to_stack(self.regs.pc, memory);
            self.regs.pc = imm16;
            return;
        } 

        if opcode & 0x07 == 0x07 {
            // rst tgst3
            let target = (opcode & 0x38) as u16;
            self.push_to_stack(self.regs.pc, memory);
            self.regs.pc = target;

            return;
        }

        let r16 = (opcode & 0x30) >> 3;
        match opcode & 0x0F {
            0x01 => {
                // pop r16stk
                let value = self.pop_from_stack(memory);
                self.regs.set_r16_stk(r16, value);
                return;
            },
            0x05 => {
                // push r16stk
                self.push_to_stack(self.regs.get_r16_stk(r16), memory);
                return;
            },
            _ => {},
        }

        match opcode {
            0xE2 => {
                // ldh [c], a
                memory[0xFF00 + self.regs.c as u16] = self.regs.a;
                return;
            },
            0xE0 => {
                // ldh [imm8], a
                memory[0xFF00 + imm8 as u16] = self.regs.a;
                self.regs.pc += 1;
                return;
            },
            0xEC => {
                // ld [imm16], a
                memory[imm16] = self.regs.a;
                self.regs.pc += 2;
                return;
            },
            0xF2 => {
                // ldh a, [c]
                self.regs.a = memory[0xFF00 + self.regs.c as u16];
                return;
            },
            0xF0 => {
                // ldh a, [imm8]
                self.regs.a = memory[0xFF00 + imm8 as u16];
                self.regs.pc += 1;
                return;
            },
            0xFC => {
                // ld a, [imm16]
                self.regs.a = memory[imm16];
                self.regs.pc += 2;
                return;
            },
            0xE8 => {
                // add sp, imm8
                self.regs.sp += imm8 as u16;
                self.regs.pc += 1;
                return;
            },
            0xF8 => {
                // ld hl, sp + imm8
                self.regs.set_hl(self.regs.sp + imm8 as u16);
                self.regs.pc += 1;
                return;
            },
            0xF9 => {
                // ld sp, hl
                self.regs.sp = self.regs.get_hl();
                return;
            },
            0xF3 => {
                // di
                self.interrupts_enabled = false;
                return;
            },
            0xFB => {
                // ei
                self.interrupts_enabled = true;
                return;
            },
            0xCB => {
                self.execute_cb_opcode(memory[self.regs.pc], memory);
                self.regs.pc += 1;
                return;
            },
            _ => {},
        }

        unsupported_opcode!(opcode, self.regs.pc);
    }

    fn execute_cb_opcode(&mut self, opcode: u8, memory: &mut Memory) {
        let reg = opcode & 0x03;
        let bit = 1 << ((opcode & 0x38) >> 3);
        
        match opcode & 0xC0 {
            0x00 => {
                match opcode & 0xF8 {
                    0x00 => {
                        // rlc r8
                        let mut value = self.regs.get_r8(reg, memory);
                        self.regs.flags = Flags::default();

                        if value & 0x80 == 1 {
                            self.regs.flags |= Flags::Carry;
                        }
                        value = value.rotate_left(1);
                        self.regs.set_r8(reg, value, memory);

                        return;
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
                        return;
                    },
                    0x10 => {
                        // rl r8
                        let value = self.regs.get_r8(reg, memory);
                        let mut shifted = value << 1;

                        if self.regs.flags.contains(Flags::Carry) {
                            shifted |= 1;
                        }
                        if value & 0x80 == 1 {
                            self.regs.flags |= Flags::Carry
                        }

                        self.regs.set_r8(reg, shifted, memory);
                        return;     
                    },
                    0x18 => {
                        // rr r8
                        let value = self.regs.get_r8(reg, memory);
                        let mut shifted = value >> 1;

                        if self.regs.flags.contains(Flags::Carry) {
                            shifted |= 0x80;
                        }
                        if value & 0x01 == 1 {
                            self.regs.flags |= Flags::Carry
                        }
                        
                        self.regs.set_r8(reg, shifted, memory);
                        return;
                    },
                    0x20 => {
                        // sla r8
                        if self.regs.get_r8(reg, memory) & 0x40 == 1 {
                            self.regs.flags |= Flags::Carry;
                        }
                        self.regs.apply_r8(reg, memory, |r| ((r as i8) << 1) as u8);
                        return;
                    },
                    0x28 => {
                        // sra r8
                        if self.regs.get_r8(reg, memory) & 0x40 == 1 {
                            self.regs.flags |= Flags::Carry;
                        }
                        self.regs.apply_r8(reg, memory, |r| ((r as i8) >> 1) as u8);
                        return;
                    },
                    0x30 => {
                        self.regs.apply_r8(reg, memory, |r| (r & 0x0F << 4) | r >> 4);
                        return;
                    }
                    0x38 => {
                        // srl r8
                        if self.regs.get_r8(reg, memory) & 0x40 == 1 {
                            self.regs.flags |= Flags::Carry;
                        }
                        self.regs.apply_r8(reg, memory, |r| r >> 1);
                        return;
                    },

                    _ => {},
                }
            },
            0x40 => {
                // bit b3, r8
                if self.regs.get_r8(reg, memory) & bit == 0 {
                    self.regs.flags |= Flags::Zero;
                }
                return;
            },
            0x80 => {
                // res b3, r8
                self.regs.apply_r8(reg, memory, |reg| reg & !bit);
                return;
            },
            0xC0 => {
                // set b3, r8
                self.regs.apply_r8(reg, memory, |reg| reg | bit);
                return;
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