pub mod io;
pub mod cpu;
mod ram;

struct Hardware {
    io: io::IO,
    cpu: cpu::CPU,
    ram: ram::RAM,
}