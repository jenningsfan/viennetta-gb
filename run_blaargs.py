import subprocess, io

TESTS = [ "01-special.gb", "02-interrupts.gb", "03-op sp,hl.gb", "04-op r,imm.gb", "05-op rp.gb",
    "06-ld r,r.gb", "07-jr,jp,call,ret,rst.gb", "08-misc instrs.gb",
    "09-op r,r.gb", "10-bit ops.gb", "11-op a,(hl).gb"
]

#TESTS = [ "09-op r,r.gb"]

subprocess.run("cargo build --release")

for test in TESTS:
    try:
        subprocess.run(["target/release/viennetta_gb.exe", f"./cpu_instrs/individual\{test}"], stdout=subprocess.PIPE, timeout=5)
        print(f"{test} crashed")
        continue
    except subprocess.TimeoutExpired as e:
        output = e.output.decode("utf-8")
        if "Passed" in output:
            print(f"{test} passed")
        elif "Failed" in output:
            print(f"{test} failed\n")
            print(f"{test} output:\n{output}")
        else:
            print(f"unexpected output: {test}")