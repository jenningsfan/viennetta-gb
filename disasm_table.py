R8 = ["b", "c", "d", "e", "h", "l", "[hl]", "a"]
R16 = ["bc", "de", "hl", "sp"]
R16STK = ["bc", "de", "hl", "af"]
R16MEM = ["bc", "de", "hl+", "hl-"]
COND = ["nz", "z", "nc", "c"]

with open("instructions.csv") as file:
    instrs = file.readlines()

table = []

for i in range(256):
    table.append(hex(i)[2:])

def replace_r8(ins):
    for (i, reg) in enumerate(R8):
        new_list = list(ins) + [[]] * (9 - len(ins))
        r8_pos = new_list.index("r8")
        new_list[r8_pos] = str((i >> 2) & 1)
        new_list[r8_pos + 1] = str((i >> 1) & 1)
        new_list[r8_pos + 2] = str(i & 1)

        table[(int("".join(new_list[1:]), base=2))] = new_list[0].strip('"').replace("r8", reg)
        
def replace_4_opts(ins, opt, opts):
    for (i, reg) in enumerate(opts):
        new_list = list(ins) + [[]] * (9 - len(ins))
        r8_pos = new_list.index(opt)
        new_list[r8_pos] = str((i >> 1) & 1)
        new_list[r8_pos + 1] = str(i & 1)

        table[(int("".join(new_list[1:]), base=2))] = new_list[0].strip('"').replace(opt, reg)

for (i, r8_1) in enumerate(R8):
    for (j, r8_2) in enumerate(R8):
        table[int((0x40 | (i << 3) | j))] = f"ld {r8_1}, {r8_2}"

for ins in instrs:
    ins = ins.strip().split("\t")

    if not all(ins) or len(ins) != 9:
        if "r8" in ins[0]:
            replace_r8(ins)  
        elif "r16stk" in ins[0]:
            replace_4_opts(ins, "r16stk", R16STK)
        elif "r16mem" in ins[0]:
            replace_4_opts(ins, "r16mem", R16MEM)
        elif "cond" in ins[0]:
            replace_4_opts(ins, "cond", COND)
        elif "r16" in ins[0]:
            replace_4_opts(ins, "r16", R16)
        continue

    table[(int("".join(ins[1:]), base=2))] = ins[0].strip('"')

with open("viennetta_gb\src\disasm.rs", "w") as file:
    table_str = ",".join([f'"{i}"' for i in table])
    file.write(f'pub const OPCODES: [&str; 256] = [{table_str}];')