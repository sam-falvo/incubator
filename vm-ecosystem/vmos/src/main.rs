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


fn sign_extend(input: u64, from_bit: usize) -> u64 {
    let bit = (input >> from_bit) & 1;
    let mask = !0 << from_bit;

    if bit != 0 {
        input | mask
    } else {
        input
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
        let fn3 = (cpu.instruction >> 12) & 0x07;
        let rs1 = ((cpu.instruction >> 15) & 0x1F) as usize;
        let rs2 = ((cpu.instruction >> 20) & 0x1F) as usize;
        let _fn7 = (cpu.instruction >> 25) & 0x7F;
        let shamt6 = (cpu.instruction >> 20) & 0x3F;
        let _shamt5 = shamt6 & 0x1F;
        let iimm = sign_extend(((cpu.instruction >> 20) & 0xFFF) as u64, 11);
        let uimm = sign_extend((cpu.instruction & 0xFFFFF000) as u64, 31);
        let jdisp = sign_extend(
            (
                (((cpu.instruction >> 31) & 1) << 20) |
                (((cpu.instruction >> 21) & 0x3FF) << 1) |
                (((cpu.instruction >> 20) & 1) << 11) |
                (((cpu.instruction >> 12) & 0xFF) << 12)
            ) as u64,
            20
        );
        let bdisp = sign_extend(
            (
                (((cpu.instruction >> 31) & 1) << 12) |
                (((cpu.instruction >> 25) & 0x3F) << 5) |
                (((cpu.instruction >> 8) & 0xF) << 1) |
                (((cpu.instruction >> 7) & 1) << 11)
            ) as u64,
            12
        );
        let simm = sign_extend(
            (
                (((cpu.instruction >> 25) & 0x7F) << 5) |
                (((cpu.instruction >> 7) & 0x1F) << 0)
            ) as u64,
            11
        );
        let mut npc = cpu.pc + 4;

        match opcode {
            // LUI
            0x37 => cpu.xr[rd] = uimm,
            // AUIPC
            0x17 => cpu.xr[rd] = cpu.pc.wrapping_add(uimm),
            // JAL
            0x6F => { cpu.xr[rd] = npc; npc = cpu.pc.wrapping_add(jdisp) },
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
                    8..=u32::MAX => unreachable!(),
                }
            }
            // LB/LH/LW/LD
            // LBU/LHU/LWU/LDU
            0x03 => {
                let ea = cpu.xr[rs1].wrapping_add(iimm);

                cpu.xr[rd] = match fn3 {
                    0 => sign_extend(cpu.load_byte(&code, ea) as u64, 7),
                    1 => sign_extend(cpu.load_hword(&code, ea) as u64, 15),
                    2 => sign_extend(cpu.load_word(&code, ea) as u64, 31),
                    3 => sign_extend(cpu.load_dword(&code, ea) as u64, 63),
                    4 => cpu.load_byte(&code, ea) as u64,
                    5 => cpu.load_hword(&code, ea) as u64,
                    6 => cpu.load_word(&code, ea) as u64,
                    7 => cpu.load_dword(&code, ea) as u64,
                    8..=u32::MAX => unreachable!(),
                }
            }
            // SB/SH/SW/SD
            0x23 => {
                let ea = cpu.xr[rs1].wrapping_add(simm);
                let v = cpu.xr[rs2];

                match fn3 {
                    0 => cpu.store_byte(&mut code, ea, v as u8),
                    1 => cpu.store_hword(&mut code, ea, v as u16),
                    2 => cpu.store_word(&mut code, ea, v as u32),
                    3 => cpu.store_dword(&mut code, ea, v as u64),
                    4 | 5 | 6 | 7 => {
                        cpu.scause = TrapCause::IllegalInstruction;
                        cpu.sepc = cpu.pc;
                        cpu.stval = cpu.instruction as u64;
                        npc = cpu.pc;
                    }
                    8..=u32::MAX => unreachable!(),
                }
            }
            // ADDI SLTI SLTIU XORI ORI ANDI SLLI SRLI SRAI
            0x13 => {
                let v = iimm;
                match fn3 {
                    0 => cpu.xr[rd] = cpu.xr[rs1].wrapping_add(v),
                    1 => cpu.xr[rd] = cpu.xr[rs1] << shamt6,
                    2 => cpu.xr[rd] = if (cpu.xr[rs1] as i64) < (v as i64) { 1 } else { 0 },
                    3 => cpu.xr[rd] = if (cpu.xr[rs1] as u64) < (v as u64) { 1 } else { 0 },
                    4 => cpu.xr[rd] = cpu.xr[rs1] ^ v,
                    5 => cpu.xr[rd] = if ((cpu.instruction >> 30) & 1) != 0 {
                        ((cpu.xr[rs1] as i64) >> shamt6) as u64
                    } else {
                        cpu.xr[rs1] >> shamt6
                    },
                    6 => cpu.xr[rd] = cpu.xr[rs1] | v,
                    7 => cpu.xr[rd] = cpu.xr[rs1] & v,
                    8..=u32::MAX => unreachable!(),
                }
            }
            // ADD SUB SLL SLT SLTU XOR SRL SRA OR AND
            0x33 => {
                let v = cpu.xr[rs2];
                match fn3 {
                    0 => cpu.xr[rd] = if ((cpu.instruction >> 30) & 1) != 0 {
                        cpu.xr[rs1].wrapping_sub(v)
                    } else {
                        cpu.xr[rs1].wrapping_add(v)
                    },
                    1 => cpu.xr[rd] = cpu.xr[rs1] << v,
                    2 => cpu.xr[rd] = if (cpu.xr[rs1] as i64) < (v as i64) { 1 } else { 0 },
                    3 => cpu.xr[rd] = if (cpu.xr[rs1] as u64) < (v as u64) { 1 } else { 0 },
                    4 => cpu.xr[rd] = cpu.xr[rs1] ^ v,
                    5 => cpu.xr[rd] = if ((cpu.instruction >> 30) & 1) != 0 {
                        ((cpu.xr[rs1] as i64) >> v) as u64
                    } else {
                        cpu.xr[rs1] >> v
                    },
                    6 => cpu.xr[rd] = cpu.xr[rs1] | v,
                    7 => cpu.xr[rd] = cpu.xr[rs1] & v,
                    8..=u32::MAX => unreachable!(),
                }
            }
            // FENCE and friends -- friendly NOPs for us.
            0x0F => (),
            // ECALL/EBREAK
            0x73 => {
                cpu.scause = if ((cpu.instruction >> 30) & 1) != 0 {
                    TrapCause::Breakpoint
                } else {
                    TrapCause::EnvironmentCallFromUmode
                };
                cpu.sepc = cpu.pc;
                cpu.stval = cpu.instruction as u64;
                npc = cpu.pc;
            },

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

    println!("TRAP!\n INSN={:08X}  SCAUSE={:?} SEPC={:016X} STVAL={:016X}\n", cpu.instruction, cpu.scause, cpu.sepc, cpu.stval);
    for x in (0..32).step_by(4) {
        print!("  X{:02}={:016X} ", x, cpu.xr[x]);
        print!("  X{:02}={:016X} ", x+1, cpu.xr[x+1]);
        print!("  X{:02}={:016X} ", x+2, cpu.xr[x+2]);
        print!("  X{:02}={:016X}\n", x+3, cpu.xr[x+3]);
    }

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
        self.instruction = self.load_word(ram, self.pc);
        self.scause = match self.scause {
            TrapCause::LoadAddressMisaligned => TrapCause::InstructionAddressMisaligned,
            TrapCause::LoadAccessFault => TrapCause::InstructionAccessFault,
            _ => self.scause,
        };
    }

    fn load_dword(&mut self, ram: &Vec<u8>, addr: u64) -> u64 {
        self.check_load(addr, 7);
        self.load_dword_unchecked(ram, addr)
    }

    fn load_word(&mut self, ram: &Vec<u8>, addr: u64) -> u32 {
        self.check_load(addr, 3);
        self.load_word_unchecked(ram, addr)
    }

    fn load_hword(&mut self, ram: &Vec<u8>, addr: u64) -> u16 {
        self.check_load(addr, 1);
        self.load_hword_unchecked(ram, addr)
    }

    fn load_byte(&mut self, ram: &Vec<u8>, addr: u64) -> u8 {
        self.check_load(addr, 0);
        self.load_byte_unchecked(ram, addr)
    }

    fn load_dword_unchecked(&mut self, ram: &Vec<u8>, addr: u64) -> u64 {
        let w0 = self.load_word_unchecked(ram, addr);
        let w1 = self.load_word_unchecked(ram, addr+4);

        ((w1 as u64) << 32) | (w0 as u64)
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

    fn store_dword(&mut self, ram: &mut Vec<u8>, addr: u64, val: u64) {
        if self.check_store(addr, 7) {
            self.store_dword_unchecked(ram, addr, val);
        }
    }

    fn store_word(&mut self, ram: &mut Vec<u8>, addr: u64, val: u32) {
        if self.check_store(addr, 3) {
            self.store_word_unchecked(ram, addr, val);
        }
    }

    fn store_hword(&mut self, ram: &mut Vec<u8>, addr: u64, val: u16) {
        if self.check_store(addr, 1) {
            self.store_hword_unchecked(ram, addr, val);
        }
    }

    fn store_byte(&mut self, ram: &mut Vec<u8>, addr: u64, val: u8) {
        if self.check_store(addr, 0) {
            self.store_byte_unchecked(ram, addr, val);
        }
    }

    fn store_dword_unchecked(&mut self, ram: &mut Vec<u8>, addr: u64, val: u64) {
        let lo = (val & 0xFFFFFFFF) as u32;
        let hi = ((val >> 8) & 0xFFFFFFFF) as u32;
        self.store_word_unchecked(ram, addr, lo);
        self.store_word_unchecked(ram, addr+4, hi);
    }

    fn store_word_unchecked(&mut self, ram: &mut Vec<u8>, addr: u64, val: u32) {
        let lo = (val & 0xFFFF) as u16;
        let hi = ((val >> 8) & 0xFFFF) as u16;
        self.store_hword_unchecked(ram, addr, lo);
        self.store_hword_unchecked(ram, addr+2, hi);
    }

    fn store_hword_unchecked(&mut self, ram: &mut Vec<u8>, addr: u64, val: u16) {
        let lo = (val & 0xFF) as u8;
        let hi = ((val >> 8) & 0xFF) as u8;
        self.store_byte_unchecked(ram, addr, lo);
        self.store_byte_unchecked(ram, addr+1, hi);
    }

    fn store_byte_unchecked(&mut self, ram: &mut Vec<u8>, addr: u64, val: u8) {
        if let Some(b) = ram.get_mut(addr as usize) {
            *b = val;
        }
    }

    fn check_store(&mut self, addr: u64, align_mask: u64) -> bool {
        if (addr & align_mask) != 0 {
            self.scause = TrapCause::StoreAMOAddressMisaligned;
            self.sepc = self.pc;
            self.stval = addr;
            return false;
        }

    //    if addr < 4096 {
    //        self.scause = TrapCause::StoreAMOAccessFault;
    //        self.sepc = self.pc;
    //        self.stval = addr;
    //        return false;
    //    }

        if addr >= RAM_SIZE {
            self.scause = TrapCause::StoreAMOAccessFault;
            self.sepc = self.pc;
            self.stval = addr;
            return false;
        }

        true
    }

    fn check_load(&mut self, addr: u64, align_mask: u64) {
        if (addr & align_mask) != 0 {
            self.scause = TrapCause::LoadAddressMisaligned;
            self.sepc = self.pc;
            self.stval = addr;
            return;
        }

    //    if addr < 4096 {
    //        self.scause = TrapCause::LoadAccessFault;
    //        self.sepc = self.pc;
    //        self.stval = addr;
    //        return None;
    //    }

        if addr >= RAM_SIZE {
            self.scause = TrapCause::LoadAccessFault;
            self.sepc = self.pc;
            self.stval = addr;
        }
    }
}

