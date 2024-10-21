use std::collections::HashMap;

use sdl2::event::Event;
use sdl2::libc;
use snowflake::ProcessUniqueId;

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
pub type HandleTableEntry = Option<Box<dyn Manageable>>;

/// Handle table is a vector of Handle Table Entries.
pub type HandleTable = Vec<HandleTableEntry>;

/// Stores `HandleTable`s for each `EmState` in the current process.
#[derive(Default)]
pub struct HandleTableRepo {
    /// Each `EmState` has a unique id (within the current process), which can be used
    /// to get at the `EmState`'s `HandleTable`
    /// This table provides a way of making emulator-managed resources
    /// available to the running virtual processor environment.
    em_state_handle_table_map: HashMap<EmStateUniqueProcessId, HandleTable>,
}

impl HandleTableRepo {
    /// Create a new `HandleTableRepo`
    pub fn new() -> Self {
        Default::default()
    }

    /// Insert a new `HandleTable` with the provided `prepopulated_entry` and associate it
    /// with the given `EmStateUniqueProcessId` (which originates from an `EmState`).
    ///
    /// If the repo did not have this `EmStateUniqueProcessId` present, `None` is returned.
    /// If the repo did have this `EmStateUniqueProcessId` present, the `HandleTable` is updated,
    /// and the old `HandleTable` is returned.
    pub fn insert_new_handle_table(
        &mut self,
        id: EmStateUniqueProcessId,
        prepopulated_entry: Box<dyn Manageable>,
    ) -> Option<HandleTable> {
        // Create initial handle table.  We pre-populate handle 4 with a handle to
        // the currently running program.
        let mut v: Vec<_> = (0..64).map(|_| None).collect();
        v[4] = Some(prepopulated_entry);
        self.em_state_handle_table_map.insert(id, v)
    }

    pub fn call_handler(&mut self, em: &mut EmState, proc: u64) {
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
                            em.cpu.pc = em.cpu.sepc + 4;
                            em.cpu.scause = TrapCause::None;

                            let which = em.cpu.xr[10] as usize;

                            let resource = self
                                .em_state_handle_table_map
                                .get_mut(&em.unique_process_id)
                                .and_then(|handle_table| handle_table.get_mut(which));
                            if let Some(Some(r)) = resource {
                                r.get_attributes(em);
                            }
                        }

                        // Set Attributes on a handle
                        0x0002 => {
                            em.cpu.pc = em.cpu.sepc + 4;
                            em.cpu.scause = TrapCause::None;

                            let which = em.cpu.xr[10] as usize;

                            let resource = self
                                .em_state_handle_table_map
                                .get_mut(&em.unique_process_id)
                                .and_then(|handle_table| handle_table.get_mut(which));
                            if let Some(Some(r)) = resource {
                                r.set_attributes(em);
                            }
                        }

                        // Close a handle
                        0x0003 => {
                            em.cpu.pc = em.cpu.sepc + 4;
                            em.cpu.scause = TrapCause::None;

                            let which = em.cpu.xr[10] as usize;

                            let mut resource = self
                                .em_state_handle_table_map
                                .get_mut(&em.unique_process_id)
                                .and_then(|handle_table| {
                                    handle_table.get_mut(which).map(Option::take)
                                });
                            if let Some(Some(r)) = resource.as_mut() {
                                r.close(em);
                            }
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

                                    match self.find_free_handle(em.unique_process_id) {
                                        Some((which, handle_table)) => {
                                            handle_table[which] = Some(
                                                TimerTicker::new(
                                                    signal,
                                                    period,
                                                    enabled,
                                                    &mut em.timer_subsystem,
                                                    &mut em.event_subsystem,
                                                    em.timer_tick,
                                                )
                                                .as_manageable(),
                                            );
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

    /// Answer with the next free handle index together with the actual handle table, or else `None`.
    /// Note: Under the assumption that no entry of the `HandleTable` is removed after this method returns,
    /// indexing into it with the simultaneously returned index is always valid and will never panic.
    fn find_free_handle(
        &mut self,
        em_unique_proc_id: EmStateUniqueProcessId,
    ) -> Option<(usize, &mut HandleTable)> {
        if let Some(handle_table) = self.em_state_handle_table_map.get_mut(&em_unique_proc_id) {
            let free_idx = handle_table
                .into_iter()
                .enumerate()
                // find the first free entry and return it's index
                .find_map(|(i, m)| m.is_none().then_some(i));
            free_idx.zip(Some(handle_table))
        } else {
            None
        }
    }
}

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

pub type EmStateUniqueProcessId = ProcessUniqueId;

/// "Supervisor" state.  This includes things which a typical OS kernel might like to keep track
/// of.
pub struct EmState {
    /// An id that is unique for the whole process and is used
    /// to identiy an `EmState` instance
    pub unique_process_id: EmStateUniqueProcessId,
    /// The (virtual) process' memory space.
    pub mem: Vec<u8>,
    /// The (virtual) process' registers and such.
    pub cpu: Cpu,
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

struct TimerTicker<'a> {
    signal: SigBit,
    period: u32,
    enabled: bool,
    ticker: Option<Box<sdl2::timer::Timer<'a, 'a>>>,
}

impl<'a> Manageable for TimerTicker<'a> {}

impl<'a> TimerTicker<'a> {
    pub fn new(
        signal: SigBit,
        period: u32,
        enabled: bool,
        timer_subsystem: &mut sdl2::TimerSubsystem,
        event_subsystem: &mut sdl2::EventSubsystem,
        timer_tick: u32,
    ) -> Self {
        let mut ticker = None;

        if enabled {
            ticker = Some(Box::new(timer_subsystem.add_timer(
                period,
                Box::new(move || {
                    let _ = event_subsystem
                        .push_event(Event::User {
                            timestamp: 0,
                            window_id: 0,
                            type_: timer_tick,
                            code: signal.bit() as i32,
                            data1: 0 as *mut libc::c_void,
                            data2: 0 as *mut libc::c_void,
                        })
                        .unwrap();

                    period
                }),
            )));
        }

        Self {
            signal,
            period,
            enabled,
            ticker,
        }
    }

    pub fn as_manageable(self) -> Box<dyn Manageable + 'a> {
        Box::new(self)
    }
}
