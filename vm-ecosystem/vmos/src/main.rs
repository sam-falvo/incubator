// Inspired by code found at https://github.com/d0iasm/rvemu-for-book

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

/// Default RAM size (1MiB).
pub const RAM_SIZE: u64 = 1024 * 1024;


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

    Ok(())
}
