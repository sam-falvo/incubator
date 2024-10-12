//! Minimal, yet still useful, example of a resource that can be accessed via the
//! handle table of a running VM/OS application.

use std::sync::{Arc, Mutex};

use crate::{EmState, Manageable};

/// This "resource" lets a program set the return code before quitting the program.
pub struct ProgramInstance {
    pub return_code: i64,
}

impl ProgramInstance {
    /// Constructor for a new resource
    pub fn new() -> Self {
        Self { return_code: 0 }
    }

    pub fn as_manageable(self) -> Arc<Mutex<dyn Manageable>> {
        Arc::new(Mutex::new(self))
    }
}

/// ProgramInstance resource overrides.
impl Manageable for ProgramInstance {
    fn get_attributes(&mut self, em: &mut EmState) {
        let mask = em.cpu.xr[11];
        let vecbase = em.cpu.xr[12];

        if (mask & 0x01) != 0 {
            em.cpu
                .store_dword(&mut em.mem, vecbase, self.return_code as u64);
        }

        em.cpu.xr[10] = mask & 0x01;
    }

    fn set_attributes(&mut self, em: &mut EmState) {
        let mask = em.cpu.xr[11];
        let vecbase = em.cpu.xr[12];

        if (mask & 0x01) != 0 {
            self.return_code = em.cpu.load_dword(&em.mem, vecbase) as i64;
        }
        em.cpu.xr[10] = mask & 0x01;
    }

    fn close(&mut self, em: &mut EmState) {
        em.return_code = self.return_code;
        em.exit_requested = true;
    }
}
