mod cycles;
mod registers;
use cycles::*;
use registers::*;

use super::io::{Interrupts, MMU};

macro_rules! unsupported_opcode {
    ( $( $opcode:expr )+, $( $pc:expr )+ ) => {
        panic!("{:02X} is not supported at {:04X}", $($opcode),+, $($pc),+)
    };
}

#[derive(Default, Debug)]
pub struct CPU {
    pub regs: Registers,
    int_master_enable: bool,
    ei_last_instruction: bool,
    halt_mode: bool,
}

impl CPU {
    pub fn tick(&mut self, mmu: &mut MMU) -> u8 {
        if self.ei_last_instruction {
            self.int_master_enable = true;
            self.ei_last_instruction = false;
        }

        let pending_ints = mmu.int_enable & mmu.int_flag;
        if pending_ints != Interrupts::empty() {
            self.halt_mode = false;
            if self.int_master_enable {
                self.int_master_enable = false;

                // TODO: there is probably a better way of writing this
                let handler = if pending_ints.contains(Interrupts::VBlank) {
                    mmu.int_flag.remove(Interrupts::VBlank);
                    0x40
                }
                else if pending_ints.contains(Interrupts::LcdStat) {
                    mmu.int_flag.remove(Interrupts::LcdStat);
                    0x48
                }
                else if pending_ints.contains(Interrupts::Timer) {
                    mmu.int_flag.remove(Interrupts::Timer);
                    0x50
                }
                else if pending_ints.contains(Interrupts::Serial) {
                    mmu.int_flag.remove(Interrupts::Serial);
                    0x58
                }
                else if pending_ints.contains(Interrupts::Joypad) {
                    mmu.int_flag.remove(Interrupts::Joypad);
                    0x60
                }
                else {
                    panic!("invalid interrupt value")
                };

                // TODO: could be more accurate cycle wise
                // https://gbdev.io/pandocs/Interrupts.html#interrupt-handling
                self.push_to_stack(self.regs.pc, mmu);
                self.regs.pc = handler;
                //println!("going to {handler:02X}");
                return 5;
            }
        }

        if !self.halt_mode {
            self.handle_opcode(mmu)
        }
        else {
            1
        }
    }

    pub fn handle_opcode(&mut self, mmu: &mut MMU) -> u8 {
        let opcode = mmu.read_memory(self.regs.pc);
        //println!("{:04X}", self.regs.pc);
        //println!("{:04X}: {:02X}{:02X}", self.regs.pc, opcode, mmu.read_memory(self.regs.pc + 1));
        let block = opcode >> 6;
        
        let cycles = if opcode == 0xCB {
            CB_CYCLES[ mmu.read_memory(self.regs.pc + 1) as usize]
        }
        else {
            INSTR_CYCLES[opcode as usize]
        };
        
        self.regs.pc += 1;
        //self.dump_regs();

        match block {
            0x0 => {
                let r16 = (opcode >> 4) & 0x03;
                let imm16 = (mmu.read_memory(self.regs.pc + 1) as u16) << 8 | mmu.read_memory(self.regs.pc) as u16;
                if opcode == 0x00 {
                    // nop
                    return cycles;
                }
        
                {
                    match opcode & 0xF {
                        0x1 => {
                            // ld r16, imm16
                            self.regs.set_r16(r16, imm16);
                            self.regs.pc += 2;
                            return cycles;
                        },
                        0x2 => {
                            // ld [r16mem], a
                            let addr = self.regs.get_r16_mem(r16);
                            mmu.write_memory(addr, self.regs.a);
                            return cycles;
                        },
                        0xA => {
                            // ld a, [r16mem]
                            let addr = self.regs.get_r16_mem(r16);
                            self.regs.a = mmu.read_memory(addr);
                            return cycles;
                        },

                        0x3 => {
                            // inc r16
                            self.regs.apply_r16(r16, |r| r + 1);
                            return cycles;
                        },
                        0xB => {
                            // dec r16
                            self.regs.apply_r16(r16, |r| r - 1);
                            return cycles;
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
                            return cycles;
                        },
        
                        _ => {},
                    };
        
                    if opcode == 0x08 {
                        // ld [imm16], sp
                        mmu.write_memory(imm16, (self.regs.sp & 0xFF) as u8);
                        mmu.write_memory(imm16 + 1, (self.regs.sp >> 8) as u8);
                        self.regs.pc += 2;
                        return cycles;
                    }
                }
        
                {
                    let r8 = (opcode >> 3) & 0x07;
                    //const HL_POINT: u8 = 6;
        
                    match opcode & 0x7 {
                        0x4 => {
                            // inc r8
                            self.regs.add_r8(r8, 1, mmu, false);
                            return cycles;
                        },
                        0x5 => {
                            // dec r8
                            self.regs.sub_r8(r8, 1, mmu, false);
                            return cycles;
                        },
                        0x6 => {
                            // ld r8, imm8
                            self.regs.set_r8(r8, mmu.read_memory(self.regs.pc), mmu);
                            self.regs.pc += 1;
                            return cycles;
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
                        return cycles;
                    },
                    0x0F => {
                        // rrca
                        self.regs.flags = Flags::default();
                        if self.regs.a & 0x01 == 1 {
                            self.regs.flags |= Flags::Carry;
                        }
                        
                        self.regs.a = self.regs.a.rotate_right(1);
                        return cycles;
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
                        return cycles;
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
                        return cycles;      
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
                        
                        self.regs.flags = self.regs.flags & (Flags::Negative);//| Flags::Carry);
                        if self.regs.a == 0 {
                            self.regs.flags |= Flags::Zero;
                        }
                        if should_carry {
                            self.regs.flags |= Flags::Carry;
                        }
                        return cycles;
                    },
                    0x2F => {
                        // cpl
                        self.regs.a = !self.regs.a;
                        self.regs.flags |= Flags::Negative;
                        self.regs.flags |= Flags::HalfCarry;
                        return cycles;
                    },
                    0x37 => {
                        // scf
                        self.regs.flags |= Flags::Carry;
                        self.regs.flags -= Flags::HalfCarry;
                        self.regs.flags -= Flags::Negative;
                        return cycles;
                    },
                    0x3F => {
                        // ccf
                        self.regs.flags ^= Flags::Carry;
                        self.regs.flags -= Flags::HalfCarry;
                        self.regs.flags -= Flags::Negative;
                        return cycles;
                    },
        
                    _ => {},
                };
        
                if opcode == 0x18 {
                    // jr imm8
                    let offset = mmu.read_memory(self.regs.pc) as i8 as i16 + 1;
                    self.regs.pc = (self.regs.pc as i16 + offset) as u16;
                    return cycles;
                }
        
                if opcode & 0x27 == 0x20 {
                    // jr cond, imm8
                    let condition = self.regs.condition((opcode & 0x18) >> 3);
                    if condition {
                        let offset = mmu.read_memory(self.regs.pc) as i8 as i16 + 1;
                        self.regs.pc = (self.regs.pc as i16 + offset) as u16;
                    }
                    else {
                        self.regs.pc += 1; // imm8
                    }
                    return cycles;
                }
        
                if opcode == 0x10 {
                    // stop
                    todo!("Stop opcode");
                }
            },
            0x1 => {
                if opcode == 0x76 {
                    // TODO: halt bug + more accuracy
                    self.halt_mode = true;
                }
        
                let source_reg = opcode & 0x07;
                let dest_reg = (opcode >> 3) & 0x07;
        
                self.regs.set_r8(dest_reg, self.regs.get_r8(source_reg, mmu), mmu);
                return cycles;
            },
            0x2 => {
                // All 8-bit arithmetic
                let operand = self.regs.get_r8(opcode & 0x7, mmu);

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

                    _ => {},
                }
                return cycles;
            },
            0x3 => {
                //dbg!(opcode);
                let imm8 = mmu.read_memory(self.regs.pc);
                let imm16 = (mmu.read_memory(self.regs.pc + 1) as u16) << 8 | (mmu.read_memory(self.regs.pc) as u16);
                let condition = self.regs.condition((opcode & 0x18) >> 3);

                self.regs.pc += 1;
                match opcode {
                    0xC6 => {
                        // add a, imm8
                        self.regs.add_acc(imm8);
                        return cycles;
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
                        return cycles;
                    },
                    0xD6 => {
                        // sub a, imm8
                        self.regs.sub_acc(imm8);
                        return cycles;
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
                        return cycles;
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
                        return cycles;
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
                        return cycles;     
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
                        return cycles;
                    }
                    0xFE => {
                        // cp a, imm8
                        let old_a = self.regs.a;
                        self.regs.sub_acc(imm8);
                        self.regs.a = old_a;
                        return cycles;
                    },

                    _ => {},
                }
                self.regs.pc -= 1; // this is to counteract the increase because it wasn't needed

                if opcode == 0xC9 {
                    // ret
                    self.regs.pc = self.pop_from_stack(mmu);
                    return cycles;
                }

                if opcode == 0xD9 {
                    // reti
                    self.regs.pc = self.pop_from_stack(mmu);
                    self.int_master_enable = true;
                    return cycles;
                }

                if opcode & 0x27 == 0x00 {
                    // ret cond
                    if condition {
                        self.regs.pc = self.pop_from_stack(mmu);
                    }
                    return cycles;
                }

                if opcode & 0x27 == 0x02 {
                    // jp cond, imm16
                    if condition {
                        self.regs.pc = imm16;
                    }
                    else {
                        self.regs.pc += 2;
                    }
                    return cycles;
                }
                
                if opcode == 0xC3 {
                    // jp imm16
                    //dbg!(imm16);
                    self.regs.pc = imm16;
                    return cycles;
                }

                if opcode == 0xE9 {
                    // jp hl
                    self.regs.pc = self.regs.get_hl();
                    return cycles;
                }

                if opcode == 0xCD {
                    // call imm16
                    self.push_to_stack(self.regs.pc + 2, mmu);
                    self.regs.pc = imm16;
                    return cycles;
                }

                if opcode & 0x07 == 0x04 {
                    if condition {
                        self.push_to_stack(self.regs.pc + 2, mmu);
                        self.regs.pc = imm16;
                    }
                    else {
                        self.regs.pc += 2;
                    }
                    return cycles;
                }

                if opcode & 0x07 == 0x07 {
                    // rst tgst3
                    let target = (opcode & 0x38) as u16;
                    self.push_to_stack(self.regs.pc, mmu);
                    self.regs.pc = target;
                    return cycles;
                }

                let r16 = (opcode & 0x30) >> 4;
                match opcode & 0x0F {
                    0x01 => {
                        // pop r16stk
                        let value = self.pop_from_stack(mmu);
                        self.regs.set_r16_stk(r16, value);
                        return cycles;
                    },
                    0x05 => {
                        // push r16stk                      
                        self.push_to_stack(self.regs.get_r16_stk(r16), mmu);
                        return cycles;
                    },
                    _ => {},
                }

                match opcode {
                    0xE2 => {
                        // ldh [c], a
                        mmu.write_memory(0xFF00 + self.regs.c as u16, self.regs.a);
                        return cycles;
                    },
                    0xE0 => {
                        // ldh [imm8], a
                        mmu.write_memory(0xFF00 + imm8 as u16, self.regs.a);
                        self.regs.pc += 1;
                        return cycles;
                    },
                    0xEA => {
                        // ld [imm16], a
                        mmu.write_memory(imm16, self.regs.a);
                        self.regs.pc += 2;
                        return cycles;
                    },
                    0xF2 => {
                        // ldh a, [c]
                        self.regs.a = mmu.read_memory(0xFF00 + self.regs.c as u16);
                        return cycles;
                    },
                    0xF0 => {
                        // ldh a, [imm8]
                        self.regs.a = mmu.read_memory(0xFF00 + imm8 as u16);
                        self.regs.pc += 1;
                        return cycles;
                    },
                    0xFA => {
                        // ld a, [imm16]
                        self.regs.a = mmu.read_memory(imm16);
                        self.regs.pc += 2;
                        return cycles;
                    },
                    0xE8 => {
                        // add sp, imm8
                        self.regs.sp = self.regs.add_sp_signed(imm8);
                        self.regs.pc += 1;
                        return cycles;
                    },
                    0xF8 => {
                        // ld hl, sp + imm8
                        let result = self.regs.add_sp_signed(imm8);
                        self.regs.set_hl(result);
                        self.regs.pc += 1;
                        return cycles;
                    },
                    0xF9 => {
                        // ld sp, hl
                        self.regs.sp = self.regs.get_hl();
                        return cycles;
                    },
                    0xF3 => {
                        // di
                        self.int_master_enable = false;
                        return cycles;
                    },
                    0xFB => {
                        // ei
                        self.ei_last_instruction = true;
                        return cycles;
                    },
                    0xCB => {
                        self.execute_cb_opcode(mmu.read_memory(self.regs.pc), mmu);
                        self.regs.pc += 1;
                        return cycles;
                    },
                    _ => {},
                }
            },

            _ => panic!("opcode block should only be 2 bits wide")
        };

        unsupported_opcode!(opcode, self.regs.pc);
    }

    pub fn dump_regs(&self) {
        println!("AF: {:02x}{:02x} ({})", self.regs.a, self.regs.flags, self.regs.flags.to_string());
        println!("BC: {:02x}{:02x}", self.regs.b, self.regs.c);
        println!("DE: {:02x}{:02x}", self.regs.d, self.regs.e);
        println!("HL: {:02x}{:02x}", self.regs.h, self.regs.l);
        println!("SP: {:04x}", self.regs.sp);
        println!("PC: {:04x} ", self.regs.pc);
    }

    pub fn trace_regs(&self) {
        eprintln!("A:{:02x} F:{} BC:{:02x}{:02x} DE:{:02x}{:02x} HL:{:02x}{:02x} SP:{:04x} PC:{:04x}",
            self.regs.a, self.regs.flags.to_string_trace(), self.regs.b, self.regs.c, self.regs.d,
            self.regs.e, self.regs.h, self.regs.l, self.regs.sp, self.regs.pc);
    }

    fn execute_cb_opcode(&mut self, opcode: u8, mmu: &mut MMU) {
        let reg = opcode & 0x07;
        let bit = 1 << ((opcode & 0x38) >> 3);
        
        match opcode & 0xC0 {
            0x00 => {
                match opcode & 0xF8 {
                    0x00 => {
                        // rlc r8
                        let mut value = self.regs.get_r8(reg, mmu);
                        self.regs.flags = Flags::default();

                        if value & 0x80 == 0x80 {
                            self.regs.flags |= Flags::Carry;
                        }
                        value = value.rotate_left(1);
                        self.regs.set_r8(reg, value, mmu);
                        if value == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                    },
                    0x08 => {
                        // rrc r8
                        let mut value = self.regs.get_r8(reg, mmu);
                        self.regs.flags = Flags::default();
                        if value & 0x01 == 1 {
                            self.regs.flags |= Flags::Carry;
                        }
                        
                        value = value.rotate_right(1);
                        self.regs.set_r8(reg, value, mmu);
                        if value == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                    },
                    0x10 => {
                        // rl r8
                        let value = self.regs.get_r8(reg, mmu);
                        
                        let mut shifted = value << 1;
                        
                        if self.regs.flags.contains(Flags::Carry) {
                            shifted |= 1;
                        }
                        self.regs.flags = Flags::default();
                        if value & 0x80 == 0x80 {
                            self.regs.flags |= Flags::Carry
                        }

                        self.regs.set_r8(reg, shifted, mmu);
                        if shifted == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                    },
                    0x18 => {
                        // rr r8
                        let value = self.regs.get_r8(reg, mmu);
                        
                        let mut shifted = value >> 1;
                        
                        if self.regs.flags.contains(Flags::Carry) {
                            shifted |= 0x80;
                        }
                        self.regs.flags = Flags::default();
                        if value & 0x01 == 1 {
                            self.regs.flags |= Flags::Carry;
                        }
                        
                        self.regs.set_r8(reg, shifted, mmu);
                        if shifted == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                    },
                    0x20 => {
                        // sla r8
                        self.regs.flags = Flags::default();
                        if self.regs.get_r8(reg, mmu) & 0x80 == 0x80 {
                            self.regs.flags |= Flags::Carry;
                        }
                        self.regs.apply_r8(reg, mmu, |r| r << 1);
                        if self.regs.get_r8(reg, mmu) == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                    },
                    0x28 => {
                        // sra r8
                        self.regs.flags = Flags::default();
                        if self.regs.get_r8(reg, mmu) & 0x01 == 0x01 {
                            self.regs.flags |= Flags::Carry;
                        }
                        self.regs.apply_r8(reg, mmu, |r| ((r as i8) >> 1) as u8);
                        if self.regs.get_r8(reg, mmu) == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                    },
                    0x30 => {
                        // swap r8
                        self.regs.apply_r8(reg, mmu, |r| ((r & 0x0F) << 4) | (r >> 4));
                        if self.regs.get_r8(reg, mmu) == 0 {
                            self.regs.flags = Flags::Zero;
                        }
                        else {
                            self.regs.flags = Flags::default();
                        }
                    }
                    0x38 => {
                        // srl r8
                        self.regs.flags = Flags::default();
                        if self.regs.get_r8(reg, mmu) & 0x01 == 0x01 {
                            self.regs.flags |= Flags::Carry;
                        }
                        self.regs.apply_r8(reg, mmu, |r| r >> 1);
                        if self.regs.get_r8(reg, mmu) == 0x00 {
                            self.regs.flags |= Flags::Zero;
                        }
                    },

                    _ => {},
                }
            },
            0x40 => {
                // bit b3, r8
                self.regs.flags &= Flags::Carry;
                self.regs.flags |= Flags::HalfCarry;

                if self.regs.get_r8(reg, mmu) & bit == 0 {
                    self.regs.flags |= Flags::Zero;
                }
            },
            0x80 => {
                // res b3, r8
                self.regs.apply_r8(reg, mmu, |reg| reg & !bit);
            },
            0xC0 => {
                // set b3, r8
                self.regs.apply_r8(reg, mmu, |reg| reg | bit);
            },
            _ => {},
        }

        //unsupported_opcode!(0xCB00 | opcode as u16, self.regs.pc);
    }

    fn push_to_stack(&mut self, value: u16, mmu: &mut MMU) {
        mmu.write_memory(self.regs.sp - 1, (value >> 8) as u8);
        mmu.write_memory(self.regs.sp - 2, (value & 0xFF) as u8);
        self.regs.sp -= 2;
    }

    fn pop_from_stack(&mut self, mmu: &mut MMU) -> u16 {
        let value = ((mmu.read_memory(self.regs.sp + 1) as u16) << 8) | mmu.read_memory(self.regs.sp) as u16;
        self.regs.sp += 2;
        value
    }
}