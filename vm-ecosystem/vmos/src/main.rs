// Inspired by code found at https://github.com/d0iasm/rvemu-for-book

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process::exit;
use std::rc::Rc;
use std::cell::RefCell;

use cpu::{Cpu, TrapCause};

use sdl_state::SdlState;
use sdl2::event::Event;
use sdl2::EventPump;

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

/// Handle table entries are of this type
type HandleTableEntry<'emstate> = Option<Rc<RefCell<&'emstate mut dyn Manageable>>>;

/// "Supervisor" state.  This includes things which a typical OS kernel might like to keep track
/// of.
struct EmState<'emstate> {
    /// The (virtual) process' memory space.
    pub mem: Vec<u8>,
    /// The (virtual) process' registers and such.
    pub cpu: Cpu,
    /// The process' handle table.  This table provides a way of making emulator-managed resources
    /// available to the running virtual processor environment.
    pub handle_table: Vec<HandleTableEntry<'emstate>>,
    /// POSIX Return Code.  This will be copied from a ProgramInstance structure when that instance
    /// is dropped.
    pub return_code: i64,
    /// SDL context.
    pub sdl: SdlState,
    /// Event pump.
    pub event_pump: EventPump,
}

/// This "resource" lets a program set the return code before quitting the program.
struct ProgramInstance {
    pub return_code: i64,
}

/// All resources are required to be "manageable."
pub trait Manageable {
    /// Closing a resource is required to set the corresponding handle table entry to None.
    /// In the process, we might take some additional actions which affects the greater emulator
    /// state.
    fn close(&mut self, em: &mut EmState) {
        // By default, indicate zero fields were copied.
        em.cpu.xr[10] = 0;
    }

    /// Copies zero or more resource-specific attributes into a vector of u64s located in virtual
    /// memory.  The resource to be queried is determined by the handle.  Depending on which fields
    /// are specified, additional side-effects may occur.  Refer to the documentation for the
    /// specific resource in question for more information.
    fn get_attributes(&mut self, em: &mut EmState) {
        // By default, indicate zero fields were copied.
        em.cpu.xr[10] = 0;
    }

    /// Copies zero or more resource-specific attributes from a vector of u64s located in virtual
    /// memory.  The resource to be queried is determined by the handle.  Depending on which fields
    /// are altered, additional side-effects may occur.  Refer to the documentation for the
    /// specific resource in question for more information.
    fn set_attributes(&mut self, em: &mut EmState) {
        // By default, indicate zero fields were copied.
        em.cpu.xr[10] = 0;
    }
}

/// ProgramInstance resource overrides.
impl Manageable for ProgramInstance {
    fn close(&mut self, em: &mut EmState) {
        eprintln!("CLOSED");
        em.return_code = self.return_code;
    }
}

fn main() -> io::Result<()> {
    // Read in executable to run
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: {} <filename>", args[0]);
    }

    let mut file = File::open(&args[1])?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;
    extend_to_ram_size(&mut code);

    // Create SDL bindings
    let sdl = SdlState::new(&args[1], SCR_W, SCR_H);
    let _event_subsystem = sdl.context.event().unwrap();
    let _timer_subsystem = sdl.context.timer().unwrap();
    let event_pump = sdl.context.event_pump().unwrap();

    // Create initial handle table
    let mut handle_table = vec![None; 64];
    let mut pi = ProgramInstance { return_code: 0, };
    let pi = RefCell::<&mut dyn Manageable>::new(&mut pi);
    let pi = Rc::new(pi);
    handle_table[4] = Some(pi);

    // Begin emulation
    let cpu = Cpu::new(0);

    let mut em = EmState {
        mem: code,
        cpu,
        handle_table,
        return_code: 0,
        sdl,
        event_pump,
    };

    em.cpu.xr[10] = 4;     // Let's pretend we have a handle table for now.
    call_handler(&mut em, 0);

    'main_event_loop:  loop {
        for event in em.event_pump.wait_iter() {
            match event {
                Event::Quit { .. } => { break 'main_event_loop }
                _ => {},
            };
        };
    };

    exit(em.return_code as i32);    // will never return!
}

fn call_handler(em: &mut EmState, proc: u64) {
    em.cpu.pc = proc;

    loop {
        em.cpu.run_until_trap(&mut em.mem);

        match em.cpu.scause {
            TrapCause::EnvironmentCallFromUmode => {
                let function_code = em.cpu.xr[17];

                match function_code {
                    // Return from event handler
                    0x0000 => return,

                    // Get Attributes on a handle
                    0x0001 => {
                        em.cpu.xr[10] = 0;     // No fields read
                        em.cpu.pc = em.cpu.sepc + 4;
                        em.cpu.scause = TrapCause::None;
                    },

                    // Set Attributes on a handle
                    0x0002 => {
                        em.cpu.xr[10] = 0;     // No fields written
                        em.cpu.pc = em.cpu.sepc + 4;
                        em.cpu.scause = TrapCause::None;
                    },

                    // Close a handle
                    0x0003 => {
                        let which = em.cpu.xr[10] as usize;

                        // The following block of code will not compile.  I do not know why.
                        // The purpose of this code is to invoke the Manageable::close() method
                        // for the resource selected by which, if any.  (If none, just ignore it.)
                        // However, for some reason, there ends up being multiple mutable borrows
                        // involved in some places, and other type errors which happen in other
                        // places (which is uncovered if you manipulate the code to avoid the
                        // mutable borrows).  This ... just doesn't work.  And I am rather upset at
                        // not being able to figure out why.  Compiler errors are just not helpful
                        // either.
                        // // //
                        let resource = &em.handle_table[which];
                        match resource {
                            Some(rc_refcell_obj) => {
                                let the_obj = rc_refcell_obj.clone();
                                let mut obj = the_obj.borrow_mut();
                                obj.close(em);
                            }

                            // Handle is not open.
                            None => (),
                        };


                        em.handle_table[which] = None;

                        em.cpu.pc = em.cpu.sepc + 4;
                        em.cpu.scause = TrapCause::None;
                    }

                    0x2A => {
                        print!("{}", em.cpu.xr[10] as u8 as char);

                        em.cpu.scause = TrapCause::None;
                        em.cpu.pc = em.cpu.sepc + 4;
                    }

                    _ => break,
                }
            }

            _ => break,
        }
    }
}

mod cpu;
mod sdl_state;

