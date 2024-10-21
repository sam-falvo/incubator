// Inspired by code found at https://github.com/d0iasm/rvemu-for-book

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process::exit;

use sdl2::event::Event;

use cpu::Cpu;
use emul_state::{EmState, EmStateUniqueProcessId, HandleTableRepo, Manageable};
use program_instance::ProgramInstance;

/// Default RAM size (1MiB).
pub const RAM_SIZE: u64 = 1024 * 1024;

/// Default screen width * height
pub const SCR_W: u32 = 1024;
pub const SCR_H: u32 = 768;

/// Extend the memory range of the virtual CPU's RAM to the size determined by the `RAM_SIZE`
/// constant.
fn extend_to_ram_size(code: &mut Vec<u8>) {
    while code.len() < (RAM_SIZE as usize) {
        code.push(0xCC);
    }
}

fn main() -> io::Result<()> {
    // Read in executable to run, then expand it as needed to become the
    // entire RAM compliment of the virtual machine.
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: {} <filename>", args[0]);
    }

    let mut file = File::open(&args[1])?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;
    extend_to_ram_size(&mut code);

    // Create SDL bindings.
    let sdl = match sdl2::init() {
        Ok(sdl_handle) => sdl_handle,
        Err(_) => {
            eprintln!("Failed to open SDL2.");
            exit(1);
        }
    };
    let mut event_pump = sdl.event_pump().unwrap();

    // Begin emulation
    let cpu = Cpu::new(0);

    let event_subsystem = sdl.event().unwrap();

    let mut em = EmState {
        unique_process_id: EmStateUniqueProcessId::new(),
        mem: code,
        cpu,
        return_code: 0,
        exit_requested: false,
        event_subsystem: sdl.event().unwrap(),
        timer_subsystem: sdl.timer().unwrap(),
        timer_tick: unsafe { event_subsystem.register_event().unwrap() },
    };

    let mut handle_table_repo = HandleTableRepo::new();
    handle_table_repo
        .insert_new_handle_table(em.unique_process_id, ProgramInstance::new().as_manageable());

    // First callback to run is the initialization callback at address 0.  Its job is to draw the
    // initial screen, set the initial callback handlers for various event sources of interest, and
    // then to just return back to the central event loop.
    //
    // Note that if the initialization code quits the program, it takes effect immediately; the
    // event loop never gets a chance to run.
    //
    // On entry:
    // A0 = handle to the program itself.
    // SP = top of memory.
    //
    // On exit:
    // All registers except for SP are undefined.

    let next_proc_to_run: u64 = 0;
    em.cpu.xr[10] = 4;
    em.cpu.xr[2] = RAM_SIZE;

    handle_table_repo.call_handler(&mut em, next_proc_to_run);

    while em.exit_requested == false {
        for event in event_pump.wait_iter() {
            match event {
                Event::Quit { .. } => {
                    em.exit_requested = true;
                    break;
                }
                _ => {}
            };
        }
    }

    exit(em.return_code as i32); // will never return!
}

mod cpu;
mod emul_state;
mod program_instance;
