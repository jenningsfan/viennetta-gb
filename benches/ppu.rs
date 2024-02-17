use criterion::{black_box, criterion_group, criterion_main, Criterion};
use viennetta_gb::hardware::io::ppu::{PPU, Colour};
use viennetta_gb::hardware::memory::Memory;

fn set_up_ppu() -> (Memory, PPU) {
    let mut memory = Memory::default();
    memory[0x8000] = 0x3C_u8;
    memory[0x8001] = 0x7E_u8;
    memory[0x8002] = 0x42_u8;
    memory[0x8003] = 0x42_u8;
    memory[0x8004] = 0x42_u8;
    memory[0x8005] = 0x42_u8;
    memory[0x8006] = 0x42_u8;
    memory[0x8007] = 0x42_u8;
    memory[0x8008] = 0x7E_u8;
    memory[0x8009] = 0x5E_u8;
    memory[0x8010] = 0x7E_u8;
    memory[0x8011] = 0x0A_u8;
    memory[0x8012] = 0x7C_u8;
    memory[0x8013] = 0x56_u8;
    memory[0x8014] = 0x38_u8;
    memory[0x8015] = 0x7C_u8;
    let ppu = PPU::default();

    (memory, ppu)
}

fn run_cycle(c: &mut Criterion) {
    let (memory, mut ppu) = set_up_ppu();
    c.bench_function("ppu.run_cycle", |b| b.iter(|| ppu.run_cycle()));
}

criterion_group!(benches, run_cycle);
criterion_main!(benches);