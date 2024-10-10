// Inspired by code found at https://github.com/d0iasm/rvemu-for-book

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process::exit;
use std::rc::Rc;
use std::cell::RefCell;

use sdl2::event::Event;

use cpu::Cpu;
use program_instance::ProgramInstance;
use emul_state::{call_handler, EmState, HandleTable, Manageable};


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

    // Create initial handle table.  We pre-populate handle 4 with a handle to
    // the currently running program.
    let mut handle_table: HandleTable = vec![None; 64];
    let mut pi = ProgramInstance::new();
    let pi = RefCell::<&mut dyn Manageable>::new(&mut pi);
    let pi = Rc::new(pi);
    handle_table[4] = Some(pi);

    // Create SDL bindings.
    //
    //let _event_subsystem = sdl.event().unwrap();
    //let _timer_subsystem = sdl.timer().unwrap();

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

    let mut em = EmState {
        mem: code,
        cpu,
        handle_table,
        return_code: 0,
        exit_requested: false,
    };

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

    call_handler(&mut em, next_proc_to_run);

    while em.exit_requested == false {
        for event in event_pump.wait_iter() {
            match event {
                Event::Quit { .. } => { em.exit_requested = true; break; }
                _ => {},
            };
        };
    };

    exit(em.return_code as i32);    // will never return!
}

mod cpu;
mod program_instance;
mod emul_state;
