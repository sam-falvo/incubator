use std::sync::{Arc, Mutex};

use sdl2::event::Event;
use sdl2::libc;

use crate::cpu::{Cpu, TrapCause};

/// Bounds-checked signal bit identifier.
pub struct SigBit(usize);

impl SigBit {
    /// Check signal bit to see if it falls in the range [0, 64).  If not, answer with None;
    /// otherwise, answer with the SigBit instance.
    pub fn new(bit: u64) -> Option<Self> {
        if bit >= 64 {
            None
        } else {
            Some(Self(bit as usize))
        }
    }

    /// Answer with the signal bit value.
    pub fn bit(&self) -> usize {
        self.0
    }
}

/// Handle table entries, which refer to kernel-side resources, are of this type.
pub type HandleTableEntry = Option<Arc<Mutex<dyn Manageable>>>;

/// Handle table is a vector of Handle Table Entries.
pub type HandleTable = Vec<HandleTableEntry>;

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
pub struct EmState {
    /// The (virtual) process' memory space.
    pub mem: Vec<u8>,
    /// The (virtual) process' registers and such.
    pub cpu: Cpu,
    /// The process' handle table.  This table provides a way of making emulator-managed resources
    /// available to the running virtual processor environment.
    pub handle_table: HandleTable,
    /// POSIX Return Code.  This will be copied from a ProgramInstance structure when that instance
    /// is closed.
    pub return_code: i64,
    /// Set to true when the program desires to quit.
    pub exit_requested: bool,
    /// SDL2 Timer subsystem.  This allows us to allocate new timers.
    pub timer_subsystem: sdl2::TimerSubsystem,
    /// SDL2 Event subsystem.  This allows us to inject new messages.
    pub event_subsystem: sdl2::EventSubsystem,
    /// SDL2 custom event for registering timer ticks.
    pub timer_tick: u32,
}

impl EmState {
    /// Answer with the next free handle, or else None.
    fn find_free_handle(&self) -> Option<usize> {
        let mut i = 0;
        while i < self.handle_table.len() {
            if self.handle_table[i].is_none() {
                return Some(i);
            }

            i = i + 1;
        }
        None
    }
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
                            let mut the_obj = rc_refcell_obj.lock().unwrap();
                            the_obj.get_attributes(em);
                        }

                        em.cpu.pc = em.cpu.sepc + 4;
                        em.cpu.scause = TrapCause::None;
                    }

                    // Set Attributes on a handle
                    0x0002 => {
                        let which = em.cpu.xr[10] as usize;

                        let resource = &em.handle_table[which];
                        if let Some(rc_refcell_obj) = resource {
                            let mut the_obj = rc_refcell_obj.lock().unwrap();
                            the_obj.set_attributes(em);
                        }

                        em.cpu.pc = em.cpu.sepc + 4;
                        em.cpu.scause = TrapCause::None;
                    }

                    // Close a handle
                    0x0003 => {
                        let which = em.cpu.xr[10] as usize;

                        let resource = &em.handle_table[which];
                        if let Some(rc_refcell_obj) = resource {
                            let mut the_obj = rc_refcell_obj.lock().unwrap();
                            the_obj.close(em);
                        }

                        em.handle_table[which] = None;

                        em.cpu.pc = em.cpu.sepc + 4;
                        em.cpu.scause = TrapCause::None;
                    }

                    0x002A => {
                        print!("{}", em.cpu.xr[10] as u8 as char);

                        em.cpu.scause = TrapCause::None;
                        em.cpu.pc = em.cpu.sepc + 4;
                    }

                    // Create a timer.
                    //
                    // On Entry:
                    // A0 = Signal bit to trigger when timer expires.
                    // A1 = Period (note: only low 32 bits are recognized in SDL-backed versions of
                    // VM/OS).
                    // A2 = Non-zero if the timer is enabled; zero if the timer is disabled.
                    //
                    // On Exit:
                    // A0 = non-zero if successful; zero otherwise.
                    // A1 = If successful, the handle of the created timer instance.  Otherwise, a
                    // reason code for the failure.
                    //
                    // If the timer could not be created, then A1 holds the reason code:
                    //
                    // - 0 -- No more handles available.
                    // - 1 -- Signal bit out of range.

                    0x0100 => {
                        match SigBit::new(em.cpu.xr[10]) {
                            Some(sb) => {
                                let signal = sb;
                                let period = em.cpu.xr[11] as u32;
                                let enabled = if em.cpu.xr[12] != 0 { true } else { false };

                                match em.find_free_handle() {
                                    Some(which) => {
                                        em.handle_table[which] = Some(TimerTicker::new(&em, signal, period, enabled).as_manageable());
                                        em.cpu.xr[10] = 1;
                                        em.cpu.xr[11] = which as u64;
                                    }

                                    None => {
                                        em.cpu.xr[10] = 0;
                                        em.cpu.xr[11] = 0;
                                    }
                                };
                            }

                            None => {
                                em.cpu.xr[10] = 0;
                                em.cpu.xr[11] = 1;
                            }
                        }

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

struct TimerTicker<'a> {
    signal: SigBit,
    period: u32,
    enabled: bool,
    ticker: Option<sdl2::timer::Timer<'a, 'a>>,
}

impl<'a> Manageable for TimerTicker<'a> {
}

impl<'a> TimerTicker<'a> {
    pub fn new(em: &EmState, signal: SigBit, period: u32, enabled: bool) -> Self {
        let mut ticker: Option<sdl2::timer::Timer> = None;

        if enabled {
            ticker = Some(
                em.timer_subsystem.add_timer(
                    period,
                    Box::new(|| {
                        let _ = em.event_subsystem
                            .push_event(Event::User {
                                timestamp: 0,
                                window_id: 0,
                                type_: em.timer_tick,
                                code: signal.bit() as i32,
                                data1: 0 as *mut libc::c_void,
                                data2: 0 as *mut libc::c_void,
                            })
                            .unwrap();

                        period
                    }),
                )
            );
        }

        Self {
            signal,
            period,
            enabled,
            ticker,
        }
    }

    pub fn as_manageable(self) -> Arc<Mutex<dyn Manageable>> {
        Arc::new(Mutex::new(self))
    }
}


