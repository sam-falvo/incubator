use std::cell::RefCell;
use std::rc::Rc;

use crate::cpu::{Cpu, TrapCause};

/// Handle table entries, which refer to kernel-side resources, are of this type.
pub type HandleTableEntry<'emstate> = Option<Rc<RefCell<&'emstate mut dyn Manageable>>>;

/// Handle table is a vector of Handle Table Entries.
pub type HandleTable<'emstate> = Vec<HandleTableEntry<'emstate>>;

/// All resources are required to be "manageable."  In this way, all resources can be closed,
/// can have attributes discovered, and can have attributes altered if required.
pub trait Manageable {
    /// Closing a resource is required to set the corresponding handle table entry to None.
    /// In the process, we might take some additional actions which affects the greater emulator
    /// state.
    ///
    /// On Entry:
    /// A0 = handle to close.
    ///
    /// On Exit:
    /// A0 = non-zero if successful; zero otherwise.
    fn close(&mut self, em: &mut EmState) {
        // By default, indicate zero fields were copied.
        em.cpu.xr[10] = 0;
    }

    /// Copies zero or more resource-specific attributes into a vector of u64s located in virtual
    /// memory.  The resource to be queried is determined by the handle.  Depending on which fields
    /// are specified, additional side-effects may occur.  Refer to the documentation for the
    /// specific resource in question for more information.
    ///
    /// On Entry:
    /// A0 = handle of resource to query
    /// A1 = bitmask indicating which attributes to query
    /// A2 = address of a vector of 64-bit dwords, each element corresponding to a bit in the supplied mask.
    ///
    /// On Exit:
    /// A0 = mask of fields actually retrieved.
    ///
    /// For example:
    /// ```text
    /// ld     a0,h_resource(s0)
    /// addi   a1,x0,$005             ; retrieve attributes 0 and 2
    /// la     a2,vector_data
    /// addi   a7,x0,ecGetAttributes
    /// ecall
    ///
    /// vector_data:
    ///     dword  0 ; where attribute 0 will end up
    ///     dword  0 ; unused, since this element is turned off by the mask
    ///     dword  0 ; where attribute 2 will end up
    /// ```
    fn get_attributes(&mut self, em: &mut EmState) {
        // By default, indicate zero fields were copied.
        em.cpu.xr[10] = 0;
    }

    /// Copies zero or more resource-specific attributes from a vector of u64s located in virtual
    /// memory.  The resource to be queried is determined by the handle.  Depending on which fields
    /// are altered, additional side-effects may occur.  Refer to the documentation for the
    /// specific resource in question for more information.
    ///
    /// On Entry:
    /// A0 = handle of resource to update
    /// A1 = bitmask indicating which attributes to update
    /// A2 = address of a vector of 64-bit dwords, each element corresponding to a bit in the supplied mask.
    ///
    /// On Exit:
    /// A0 = mask of fields actually altered.
    ///
    /// For example:
    /// ```text
    /// ld     a0,h_resource(s0)
    /// addi   a1,x0,$005             ; Update attributes 0 and 2
    /// la     a2,vector_data
    /// addi   a7,x0,ecGetAttributes
    /// ecall
    ///
    /// vector_data:
    ///     dword  15   ; where attribute 0 will be sourced from
    ///     dword  0    ; unused, since this element is turned off by the mask
    ///     dword  360  ; where attribute 2 will be sourced from
    /// ```
    fn set_attributes(&mut self, em: &mut EmState) {
        // By default, indicate zero fields were copied.
        em.cpu.xr[10] = 0;
    }
}

/// "Supervisor" state.  This includes things which a typical OS kernel might like to keep track
/// of.
pub struct EmState<'emstate> {
    /// The (virtual) process' memory space.
    pub mem: Vec<u8>,
    /// The (virtual) process' registers and such.
    pub cpu: Cpu,
    /// The process' handle table.  This table provides a way of making emulator-managed resources
    /// available to the running virtual processor environment.
    pub handle_table: HandleTable<'emstate>,
    /// POSIX Return Code.  This will be copied from a ProgramInstance structure when that instance
    /// is closed.
    pub return_code: i64,
    /// Set to true when the program desires to quit.
    pub exit_requested: bool,
}

pub fn call_handler(em: &mut EmState, proc: u64) {
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
                        let which = em.cpu.xr[10] as usize;

                        let resource = &em.handle_table[which];
                        if let Some(rc_refcell_obj) = resource {
                            let the_obj = rc_refcell_obj.clone();
                            let mut obj = the_obj.borrow_mut();
                            obj.get_attributes(em);
                        }

                        em.cpu.pc = em.cpu.sepc + 4;
                        em.cpu.scause = TrapCause::None;
                    },

                    // Set Attributes on a handle
                    0x0002 => {
                        let which = em.cpu.xr[10] as usize;

                        let resource = &em.handle_table[which];
                        if let Some(rc_refcell_obj) = resource {
                            let the_obj = rc_refcell_obj.clone();
                            let mut obj = the_obj.borrow_mut();
                            obj.set_attributes(em);
                        }

                        em.cpu.pc = em.cpu.sepc + 4;
                        em.cpu.scause = TrapCause::None;
                    },

                    // Close a handle
                    0x0003 => {
                        let which = em.cpu.xr[10] as usize;

                        let resource = &em.handle_table[which];
                        if let Some(rc_refcell_obj) = resource {
                            let the_obj = rc_refcell_obj.clone();
                            let mut obj = the_obj.borrow_mut();
                            obj.close(em);
                        }

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
