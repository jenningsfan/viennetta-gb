use criterion::{black_box, criterion_group, criterion_main, Criterion};
use viennetta_gb::hardware::io::ppu::PPU;
use viennetta_gb::hardware::memory::Memory;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
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

    let mut ppu = PPU::default();

    c.bench_function("ppu.run_cycle", |b| b.iter(|| ppu.run_cycle(&memory)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);