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

    let mut cpu = Cpu::new(0);

    cpu.fetch(&code);
    while cpu.scause == TrapCause::None {
        println!("Insn: {:08X}", cpu.instruction);
        let opcode = (cpu.instruction >> 0) & 0x7F;
        let rd = ((cpu.instruction >> 7) & 0x1F) as usize;
        // let fn3 = (cpu.instruction >> 12) & 0x07;
        let rs1 = ((cpu.instruction >> 15) & 0x1F) as usize;
        // let rs2 = ((cpu.instruction >> 20) & 0x1F) as usize;
        // let fn7 = (cpu.instruction >> 25) & 0x7F;
        let iimm = ((cpu.instruction >> 20) & 0xFFF) as i64 as u64;
        let uimm = (cpu.instruction & 0xFFFFF000) as i64 as u64;
        let jdisp = (
            (((cpu.instruction >> 31) & 1) << 20) |
            (((cpu.instruction >> 21) & 0x3FF) << 1) |
            (((cpu.instruction >> 20) & 1) << 11) |
            (((cpu.instruction >> 12) & 0xFF) << 12)
        ) as i64 as u64;
        let bdisp = (
            (((cpu.instruction >> 31) & 1) << 12) |
            (((cpu.instruction >> 25) & 0x3F) << 5) |
            (((cpu.instruction >> 8) & 0xF) << 1) |
            (((cpu.instruction >> 7) & 1) << 11)
        ) as i64 as u64;
        let mut npc = cpu.pc + 4;

        match opcode {
            // LUI
            0x37 => cpu.xr[rd] = uimm,
            // AUIPC
            0x17 => cpu.xr[rd] = cpu.pc.wrapping_add(uimm),
            // JAL
            0x6F => cpu.xr[rd] = cpu.pc.wrapping_add(jdisp),
            // JALR
            0x67 => { cpu.xr[rd] = npc; npc = cpu.xr[rs1] + iimm },
            // BEQ/BNE BLT/BGE BLTU/BGEU
            0x63 => {
                match fn3 {
                    0 => { if cpu.xr[rs1] == cpu.xr[rs2] { npc = cpu.pc.wrapping_add(bdisp) } }
                    1 => { if cpu.xr[rs1] != cpu.xr[rs2] { npc = cpu.pc.wrapping_add(bdisp) } }
                    2 | 3 => {
                        cpu.scause = TrapCause::IllegalInstruction;
                        cpu.sepc = cpu.pc;
                        cpu.stval = cpu.instruction as u64;
                        npc = cpu.pc;
                    }
                    4 => { if (cpu.xr[rs1] as i64) < (cpu.xr[rs2] as i64) { npc = cpu.pc.wrapping_add(bdisp) } }
                    5 => { if (cpu.xr[rs1] as i64) >= (cpu.xr[rs2] as i64) { npc = cpu.pc.wrapping_add(bdisp) } }
                    6 => { if (cpu.xr[rs1] as u64) < (cpu.xr[rs2] as u64) { npc = cpu.pc.wrapping_add(bdisp) } }
                    7 => { if (cpu.xr[rs1] as u64) >= (cpu.xr[rs2] as u64) { npc = cpu.pc.wrapping_add(bdisp) } }
                }
            }
            // LB/LH/LW/LD
            // LBU/LHU/LWU/LDU
            // SB/SH/SW/SD
            // ADDI
            // SLTI
            // SLTIU
            // XORI
            // ORI
            // ANDI
            // SLLI
            // SRLI
            // SRAI
            // ADD
            // SUB
            // SLL
            // SLT
            // SLTU
            // XOR
            // SRL
            // SRA
            // OR
            // AND
            // FENCE
            // ECALL/EBREAK

            _ => {
                cpu.scause = TrapCause::IllegalInstruction;
                cpu.sepc = cpu.pc;
                cpu.stval = cpu.instruction as u64;
                npc = cpu.pc;
            }
        }

        cpu.xr[0] = 0;
        cpu.pc = npc;
        cpu.fetch(&code);
    }

    println!("TRAP!\n INSN={:08X}  SCAUSE={:?} SEPC={:016X} STVAL={:016X}", cpu.instruction, cpu.scause, cpu.sepc, cpu.stval);
    Ok(())
}


struct Cpu {
    pub instruction: u32,

    pub pc: u64,
    pub xr: [u64; 32],

    pub scause: TrapCause,
    pub sepc: u64,
    pub stval: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TrapCause {
    None,
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAddressMisaligned,
    LoadAccessFault,
    StoreAMOAddressMisaligned,
    StoreAMOAccessFault,
    EnvironmentCallFromUmode,
    EnvironmentCallFromSmode,
    EnvironmentCallFromMmode,
    InstructionPageFault,
    LoadPageFault,
    StoreAMOPageFault,
}

impl Cpu {
    fn new(initial_pc: u64) -> Cpu {
        Cpu {
            instruction: 0,
            pc: initial_pc,
            xr: [0; 32],
            scause: TrapCause::None,
            sepc: 0,
            stval: 0,
        }
    }

    fn fetch(&mut self, ram: &Vec<u8>) {
        match self.load_word(ram, self.pc) {
            Some(w) => {
                self.instruction = w;
            }
            None => {
                self.scause = match self.scause {
                    TrapCause::LoadAddressMisaligned => TrapCause::InstructionAddressMisaligned,
                    TrapCause::LoadAccessFault => TrapCause::InstructionAccessFault,
                    _ => self.scause
                };
            }
        }
    }

    fn load_word(&mut self, ram: &Vec<u8>, addr: u64) -> Option<u32> {
        if (addr & 3) != 0 {
            self.scause = TrapCause::LoadAddressMisaligned;
            self.sepc = self.pc;
            self.stval = addr;
            return None;
        }

//        if addr < 4096 {
//            self.scause = TrapCause::LoadAccessFault;
//            self.sepc = self.pc;
//            self.stval = addr;
//            return None;
//        }
//
        if addr < RAM_SIZE {
            return Some(self.load_word_unchecked(ram, addr));
        }

        self.scause = TrapCause::LoadAccessFault;
        self.sepc = self.pc;
        self.stval = addr;

        None
    }

    fn load_word_unchecked(&mut self, ram: &Vec<u8>, addr: u64) -> u32 {
        let h0 = self.load_hword_unchecked(ram, addr);
        let h1 = self.load_hword_unchecked(ram, addr+2);

        ((h1 as u32) << 16) | (h0 as u32)
    }

    fn load_hword_unchecked(&mut self, ram: &Vec<u8>, addr: u64) -> u16 {
        let b0 = self.load_byte_unchecked(ram, addr);
        let b1 = self.load_byte_unchecked(ram, addr+1);

        ((b1 as u16) << 8) | (b0 as u16)
    }

    fn load_byte_unchecked(&mut self, ram: &Vec<u8>, addr: u64) -> u8 {
        match ram.get(addr as usize) {
            Some(b) => *b,
            None => 0xCC
        }
    }
}

