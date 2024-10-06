// Inspired by code found at https://github.com/d0iasm/rvemu-for-book

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use cpu::{Cpu, TrapCause};

/// Default RAM size (1MiB).
pub const RAM_SIZE: u64 = 1024 * 1024;

/// Extend the memory range of the virtual CPU's RAM to the size determined by the `RAM_SIZE`
/// constant.
fn extend_to_ram_size(code: &mut Vec<u8>) {
    while code.len() < (RAM_SIZE as usize) {
        code.push(0xCC);
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: {} <filename>", args[0]);
    }

    let mut file = File::open(&args[1])?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;
    extend_to_ram_size(&mut code);

    let mut cpu = Cpu::new(0);

    let mut done = false;
    while !done {
        cpu.run_until_trap(&mut code);

        done = match cpu.scause {
            TrapCause::EnvironmentCallFromUmode => {
                let function_code = cpu.xr[17];

                match function_code {
                    0x2A => {
                        print!("{}", cpu.xr[10] as u8 as char);

                        cpu.scause = TrapCause::None;
                        cpu.pc = cpu.sepc + 4;
                        false
                    }
                    _ => true,
                }
            }
            _ => true,
        }
    }

    println!("TRAP!");
    cpu.dump_regs();

    Ok(())
}

mod cpu;

