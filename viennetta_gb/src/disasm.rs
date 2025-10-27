use crate::hardware::io::MMU;

pub const OPCODES: [&str; 256] = ["nop","ld bc, imm16","ld [bc], a","inc bc","inc b","dec b","ld b, imm8","rlca","ld [imm16], sp","add hl, bc","ld a, [bc]","dec bc","inc c","dec c","ld c, imm8","rrca","stop","ld de, imm16","ld [de], a","inc de","inc d","dec d","ld d, imm8","rla","jr imm8","add hl, de","ld a, [de]","dec de","inc e","dec e","ld e, imm8","rra","jr nz, imm8","ld hl, imm16","ld [hl+], a","inc hl","inc h","dec h","ld h, imm8","daa","jr z, imm8","add hl, hl","ld a, [hl+]","dec hl","inc l","dec l","ld l, imm8","cpl","jr nc, imm8","ld sp, imm16","ld [hl-], a","inc sp","inc [hl]","dec [hl]","ld [hl], imm8","scf","jr c, imm8","add hl, sp","ld a, [hl-]","dec sp","inc a","dec a","ld a, imm8","ccf","ld b, b","ld b, c","ld b, d","ld b, e","ld b, h","ld b, l","ld b, [hl]","ld b, a","ld c, b","ld c, c","ld c, d","ld c, e","ld c, h","ld c, l","ld c, [hl]","ld c, a","ld d, b","ld d, c","ld d, d","ld d, e","ld d, h","ld d, l","ld d, [hl]","ld d, a","ld e, b","ld e, c","ld e, d","ld e, e","ld e, h","ld e, l","ld e, [hl]","ld e, a","ld h, b","ld h, c","ld h, d","ld h, e","ld h, h","ld h, l","ld h, [hl]","ld h, a","ld l, b","ld l, c","ld l, d","ld l, e","ld l, h","ld l, l","ld l, [hl]","ld l, a","ld [hl], b","ld [hl], c","ld [hl], d","ld [hl], e","ld [hl], h","ld [hl], l","halt","ld [hl], a","ld a, b","ld a, c","ld a, d","ld a, e","ld a, h","ld a, l","ld a, [hl]","ld a, a","add a, b","add a, c","add a, d","add a, e","add a, h","add a, l","add a, [hl]","add a, a","adc a, b","adc a, c","adc a, d","adc a, e","adc a, h","adc a, l","adc a, [hl]","adc a, a","sub a, b","sub a, c","sub a, d","sub a, e","sub a, h","sub a, l","sub a, [hl]","sub a, a","sbc a, b","sbc a, c","sbc a, d","sbc a, e","sbc a, h","sbc a, l","sbc a, [hl]","sbc a, a","and a, b","and a, c","and a, d","and a, e","and a, h","and a, l","and a, [hl]","and a, a","xor a, b","xor a, c","xor a, d","xor a, e","xor a, h","xor a, l","xor a, [hl]","xor a, a","or a, b","or a, c","or a, d","or a, e","or a, h","or a, l","or a, [hl]","or a, a","cp a, b","cp a, c","cp a, d","cp a, e","cp a, h","cp a, l","cp a, [hl]","cp a, a","ret nz","pop bc","jp nz, imm16","jp imm16","call nz, imm16","push bc","add a, imm8","c7","ret z","ret","jp z, imm16","CB prefix","call z, imm16","call imm16","adc a, imm8","cf","ret nc","pop de","jp nc, imm16","d3","call nc, imm16","push de","sub a, imm8","d7","ret c","reti","jp c, imm16","db","call c, imm16","dd","sbc a, imm8","df","ldh [imm8], a","pop hl","ldh [c], a","e3","e4","push hl","and a, imm8","e7","add sp, imm8","jp hl","ld [imm16], a","eb","ec","ed","xor a, imm8","ef","ldh a, [imm8]","pop af","ldh a, [c]","di","f4","push af","or a, imm8","f7","ld hl, sp + imm8","ld sp, hl","ld a, [imm16]","ei","fc","fd","cp a, imm8","ff"];
pub fn disasm(pc: u16, mem: &MMU) -> String {
    let instr = mem.read_memory(pc);
    let mut disasm = String::from(OPCODES[instr as usize]);

    if disasm.contains("imm8") {
        let imm8 = mem.read_memory(pc + 1);
        disasm = disasm.replace("imm8", &format!("{:02X}", imm8));
    }
    if disasm.contains("imm16") {
        let imm16 = mem.read_memory(pc + 1) as u16 | ((mem.read_memory(pc + 2) as u16) << 8);
        disasm = disasm.replace("imm16", &format!("{:04X}", imm16));
    }

    disasm
}