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

    let mut done = false;
    while !done {
        cpu.run_until_trap(&mut code);

        done = match cpu.scause {
            TrapCause::EnvironmentCallFromUmode => {
                let function_code = cpu.xr[17];

                match function_code {
                    0x2A => {
                        print!("{}", cpu.xr[10] as u8 as char);

                        cpu.scause = TrapCause::None;
                        cpu.pc = cpu.sepc + 4;
                        false
                    }
                    _ => true,
                }
            }
            _ => true,
        }
    }

    println!("TRAP!");
    cpu.dump_regs();

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

    fn dump_regs(&self) {
        println!(
            "PC={:016X} INSN={:08X}  SCAUSE={:?} SEPC={:016X} STVAL={:016X}\n",
            self.pc, self.instruction, self.scause, self.sepc, self.stval
        );
        for x in (0..32).step_by(4) {
            print!("  X{:02} {:016X}  ", x, self.xr[x]);
            print!("  X{:02} {:016X}  ", x + 1, self.xr[x + 1]);
            print!("  X{:02} {:016X}  ", x + 2, self.xr[x + 2]);
            print!("  X{:02} {:016X}\n", x + 3, self.xr[x + 3]);
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
        let w1 = self.load_word_unchecked(ram, addr + 4);

        ((w1 as u64) << 32) | (w0 as u64)
    }

    fn load_word_unchecked(&mut self, ram: &Vec<u8>, addr: u64) -> u32 {
        let h0 = self.load_hword_unchecked(ram, addr);
        let h1 = self.load_hword_unchecked(ram, addr + 2);

        ((h1 as u32) << 16) | (h0 as u32)
    }

    fn load_hword_unchecked(&mut self, ram: &Vec<u8>, addr: u64) -> u16 {
        let b0 = self.load_byte_unchecked(ram, addr);
        let b1 = self.load_byte_unchecked(ram, addr + 1);

        ((b1 as u16) << 8) | (b0 as u16)
    }

    fn load_byte_unchecked(&mut self, ram: &Vec<u8>, addr: u64) -> u8 {
        match ram.get(addr as usize) {
            Some(b) => *b,
            None => 0xCC,
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
        self.store_word_unchecked(ram, addr + 4, hi);
    }

    fn store_word_unchecked(&mut self, ram: &mut Vec<u8>, addr: u64, val: u32) {
        let lo = (val & 0xFFFF) as u16;
        let hi = ((val >> 8) & 0xFFFF) as u16;
        self.store_hword_unchecked(ram, addr, lo);
        self.store_hword_unchecked(ram, addr + 2, hi);
    }

    fn store_hword_unchecked(&mut self, ram: &mut Vec<u8>, addr: u64, val: u16) {
        let lo = (val & 0xFF) as u8;
        let hi = ((val >> 8) & 0xFF) as u8;
        self.store_byte_unchecked(ram, addr, lo);
        self.store_byte_unchecked(ram, addr + 1, hi);
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

    fn step(&mut self, ram: &mut Vec<u8>) {
        let opcode = (self.instruction >> 0) & 0x7F;
        let rd = ((self.instruction >> 7) & 0x1F) as usize;
        let fn3 = (self.instruction >> 12) & 0x07;
        let rs1 = ((self.instruction >> 15) & 0x1F) as usize;
        let rs2 = ((self.instruction >> 20) & 0x1F) as usize;
        let _fn7 = (self.instruction >> 25) & 0x7F;
        let shamt6 = (self.instruction >> 20) & 0x3F;
        let _shamt5 = shamt6 & 0x1F;
        let iimm = sign_extend(((self.instruction >> 20) & 0xFFF) as u64, 11);
        let uimm = sign_extend((self.instruction & 0xFFFFF000) as u64, 31);
        let jdisp = sign_extend(
            ((((self.instruction >> 31) & 1) << 20)
                | (((self.instruction >> 21) & 0x3FF) << 1)
                | (((self.instruction >> 20) & 1) << 11)
                | (((self.instruction >> 12) & 0xFF) << 12)) as u64,
            20,
        );
        let bdisp = sign_extend(
            ((((self.instruction >> 31) & 1) << 12)
                | (((self.instruction >> 25) & 0x3F) << 5)
                | (((self.instruction >> 8) & 0xF) << 1)
                | (((self.instruction >> 7) & 1) << 11)) as u64,
            12,
        );
        let simm = sign_extend(
            ((((self.instruction >> 25) & 0x7F) << 5) | (((self.instruction >> 7) & 0x1F) << 0))
                as u64,
            11,
        );
        let mut npc = self.pc + 4;

        match opcode {
            // LUI
            0x37 => self.xr[rd] = uimm,
            // AUIPC
            0x17 => self.xr[rd] = self.pc.wrapping_add(uimm),
            // JAL
            0x6F => {
                self.xr[rd] = npc;
                npc = self.pc.wrapping_add(jdisp)
            }
            // JALR
            0x67 => {
                self.xr[rd] = npc;
                npc = self.xr[rs1] + iimm
            }
            // BEQ/BNE BLT/BGE BLTU/BGEU
            0x63 => match fn3 {
                0 => {
                    if self.xr[rs1] == self.xr[rs2] {
                        npc = self.pc.wrapping_add(bdisp)
                    }
                }
                1 => {
                    if self.xr[rs1] != self.xr[rs2] {
                        npc = self.pc.wrapping_add(bdisp)
                    }
                }
                2 | 3 => {
                    self.scause = TrapCause::IllegalInstruction;
                    self.sepc = self.pc;
                    self.stval = self.instruction as u64;
                    npc = self.pc;
                }
                4 => {
                    if (self.xr[rs1] as i64) < (self.xr[rs2] as i64) {
                        npc = self.pc.wrapping_add(bdisp)
                    }
                }
                5 => {
                    if (self.xr[rs1] as i64) >= (self.xr[rs2] as i64) {
                        npc = self.pc.wrapping_add(bdisp)
                    }
                }
                6 => {
                    if (self.xr[rs1] as u64) < (self.xr[rs2] as u64) {
                        npc = self.pc.wrapping_add(bdisp)
                    }
                }
                7 => {
                    if (self.xr[rs1] as u64) >= (self.xr[rs2] as u64) {
                        npc = self.pc.wrapping_add(bdisp)
                    }
                }
                8..=u32::MAX => unreachable!(),
            },
            // LB/LH/LW/LD
            // LBU/LHU/LWU/LDU
            0x03 => {
                let ea = self.xr[rs1].wrapping_add(iimm);

                self.xr[rd] = match fn3 {
                    0 => sign_extend(self.load_byte(ram, ea) as u64, 7),
                    1 => sign_extend(self.load_hword(ram, ea) as u64, 15),
                    2 => sign_extend(self.load_word(ram, ea) as u64, 31),
                    3 => sign_extend(self.load_dword(ram, ea) as u64, 63),
                    4 => self.load_byte(ram, ea) as u64,
                    5 => self.load_hword(ram, ea) as u64,
                    6 => self.load_word(ram, ea) as u64,
                    7 => self.load_dword(ram, ea) as u64,
                    8..=u32::MAX => unreachable!(),
                }
            }
            // SB/SH/SW/SD
            0x23 => {
                let ea = self.xr[rs1].wrapping_add(simm);
                let v = self.xr[rs2];

                match fn3 {
                    0 => self.store_byte(ram, ea, v as u8),
                    1 => self.store_hword(ram, ea, v as u16),
                    2 => self.store_word(ram, ea, v as u32),
                    3 => self.store_dword(ram, ea, v as u64),
                    4 | 5 | 6 | 7 => {
                        self.scause = TrapCause::IllegalInstruction;
                        self.sepc = self.pc;
                        self.stval = self.instruction as u64;
                        npc = self.pc;
                    }
                    8..=u32::MAX => unreachable!(),
                }
            }
            // ADDI SLTI SLTIU XORI ORI ANDI SLLI SRLI SRAI
            0x13 => {
                let v = iimm;
                match fn3 {
                    0 => self.xr[rd] = self.xr[rs1].wrapping_add(v),
                    1 => self.xr[rd] = self.xr[rs1] << shamt6,
                    2 => {
                        self.xr[rd] = if (self.xr[rs1] as i64) < (v as i64) {
                            1
                        } else {
                            0
                        }
                    }
                    3 => {
                        self.xr[rd] = if (self.xr[rs1] as u64) < (v as u64) {
                            1
                        } else {
                            0
                        }
                    }
                    4 => self.xr[rd] = self.xr[rs1] ^ v,
                    5 => {
                        self.xr[rd] = if ((self.instruction >> 30) & 1) != 0 {
                            ((self.xr[rs1] as i64) >> shamt6) as u64
                        } else {
                            self.xr[rs1] >> shamt6
                        }
                    }
                    6 => self.xr[rd] = self.xr[rs1] | v,
                    7 => self.xr[rd] = self.xr[rs1] & v,
                    8..=u32::MAX => unreachable!(),
                }
            }
            // ADD SUB SLL SLT SLTU XOR SRL SRA OR AND
            0x33 => {
                let v = self.xr[rs2];
                match fn3 {
                    0 => {
                        self.xr[rd] = if ((self.instruction >> 30) & 1) != 0 {
                            self.xr[rs1].wrapping_sub(v)
                        } else {
                            self.xr[rs1].wrapping_add(v)
                        }
                    }
                    1 => self.xr[rd] = self.xr[rs1] << v,
                    2 => {
                        self.xr[rd] = if (self.xr[rs1] as i64) < (v as i64) {
                            1
                        } else {
                            0
                        }
                    }
                    3 => {
                        self.xr[rd] = if (self.xr[rs1] as u64) < (v as u64) {
                            1
                        } else {
                            0
                        }
                    }
                    4 => self.xr[rd] = self.xr[rs1] ^ v,
                    5 => {
                        self.xr[rd] = if ((self.instruction >> 30) & 1) != 0 {
                            ((self.xr[rs1] as i64) >> v) as u64
                        } else {
                            self.xr[rs1] >> v
                        }
                    }
                    6 => self.xr[rd] = self.xr[rs1] | v,
                    7 => self.xr[rd] = self.xr[rs1] & v,
                    8..=u32::MAX => unreachable!(),
                }
            }
            // FENCE and friends -- friendly NOPs for us.
            0x0F => (),
            // ECALL/EBREAK
            0x73 => {
                self.scause = match iimm {
                    0 => TrapCause::EnvironmentCallFromUmode,
                    1 => TrapCause::Breakpoint,
                    _ => TrapCause::IllegalInstruction,
                };
                self.sepc = self.pc;
                self.stval = self.instruction as u64;
                npc = self.pc;
            }

            _ => {
                self.scause = TrapCause::IllegalInstruction;
                self.sepc = self.pc;
                self.stval = self.instruction as u64;
                npc = self.pc;
            }
        }

        self.xr[0] = 0;
        self.pc = npc;
        self.fetch(&ram);
    }

    fn run_until_trap(&mut self, ram: &mut Vec<u8>) {
        self.fetch(ram);
        while self.scause == TrapCause::None {
            self.step(ram);
        }
    }
}
