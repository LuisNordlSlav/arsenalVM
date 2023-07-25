#![allow(unused)]

use std::slice::SliceIndex;
use std::ptr::{read_unaligned, write_unaligned};
use std::sync::Arc;

use crate::virtual_thread::*;

pub enum RegisterRoles {
    StackPointer = 14,
    ProgramCounter = 15,
}

use arsenal_globals::{Instructions, SysCalls};

pub enum ALUFlags {
    Zero = 0,
    Greater = 2,
    Lesser = 4,
    Equal = 8,
}

pub struct VirtualMachine {
    pub rules: [fn(&mut crate::virtual_thread::VirtualThread) -> (); Instructions::__END__ as usize],
    pub syscalls: [fn(&mut crate::virtual_thread::VirtualThread) -> (); SysCalls::__END__ as usize],
    pub instructions: Vec<u8>,
    pub threads: Vec<std::thread::JoinHandle<()>>,
}

impl VirtualMachine {
    pub fn new(data: &mut Vec<u8>) -> Self {
        let rules = Self::get_rules();
        let syscalls = Self::get_syscalls();
        Self {
            rules,
            syscalls,
            instructions: std::mem::take(data),
            threads: vec![],
        }
    }

    pub fn run(&mut self) {
        let thread = VirtualThread::new(self, 0, "Main".to_string());
        thread.join().unwrap();
    }

    pub fn spawn(&mut self, start: u64) {
        let thread = VirtualThread::new(self, start, "Worker".to_string());
        self.threads.push(thread);
    }

    pub fn get_rules() -> [fn(&mut VirtualThread) -> (); Instructions::__END__ as usize] {
        use arsenal_globals::Instructions::*;

        let mut rules = [(|_|{}) as fn(&mut VirtualThread) -> (); __END__ as usize];
        rules[Halt as usize] = |thread| { thread.running = false; };
        rules[SysCall as usize] = |thread| {
            let call_id = thread.last::<u8>();
            assert!(((call_id as usize) < __END__ as usize), "syscall of id {call_id} does not exist.");
            thread.parent.as_ref().syscalls[call_id as usize](thread);
        };
        rules[LoadRegisterByte as usize] = |thread| {
            let register_id = thread.last::<u8>();
            let value = thread.last::<u8>();
            thread.registers[register_id as usize] = value as u64;
        };
        rules[LoadRegisterShort as usize] = |thread| {
            let register_id = thread.last::<u8>();
            let value = thread.last::<u16>();
            thread.registers[register_id as usize] = value as u64;
        };
        rules[LoadRegisterInt as usize] = |thread| {
            let register_id = thread.last::<u8>();
            let value = thread.last::<u32>();
            thread.registers[register_id as usize] = value as u64;
        };
        rules[LoadRegisterLong as usize] = |thread| {
            let register_id = thread.last::<u8>();
            let value = thread.last::<u64>();
            thread.registers[register_id as usize] = value;
        };
        rules[SubtractRegistersByte as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = registers & 0x0f;
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u8;
            unsafe {
                write_unaligned(data, read_unaligned(data).wrapping_sub(thread.registers[r1 as usize] as u8));
            }
        };
        rules[SubtractRegistersShort as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = registers & 0x0f;
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u16;
            unsafe {
                write_unaligned(data, read_unaligned(data).wrapping_sub(thread.registers[r1 as usize] as u16));
            }
        };
        rules[SubtractRegistersInt as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = registers & 0x0f;
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u32;
            unsafe {
                write_unaligned(data, read_unaligned(data).wrapping_sub(thread.registers[r1 as usize] as u32));
            }
        };
        rules[SubtractRegistersLong as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = registers & 0x0f;
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u64;
            unsafe {
                write_unaligned(data, read_unaligned(data).wrapping_sub(thread.registers[r1 as usize] as u64));
            }
        };

        rules[AddRegistersByte as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = registers & 0x0f;
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u8;
            unsafe {
                write_unaligned(data, read_unaligned(data).wrapping_add(thread.registers[r1 as usize] as u8));
            }
        };
        rules[AddRegistersShort as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = registers & 0x0f;
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u16;
            unsafe {
                write_unaligned(data, read_unaligned(data).wrapping_add(thread.registers[r1 as usize] as u16));
            }
        };
        rules[AddRegistersInt as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = registers & 0x0f;
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u32;
            unsafe {
                write_unaligned(data, read_unaligned(data).wrapping_add(thread.registers[r1 as usize] as u32));
            }
        };
        rules[AddRegistersLong as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = registers & 0x0f;
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u64;
            unsafe {
                write_unaligned(data, read_unaligned(data).wrapping_add(thread.registers[r1 as usize] as u64));
            }
        };
        rules[AddRegisterImmediateByte as usize] = |thread| {
            let register = thread.last::<u8>();
            let data = thread.last::<u8>();
            unsafe {
                let reg = ((&mut thread.registers[(register & 0x0f) as usize]) as *mut _ as *mut u8);
                write_unaligned(reg, read_unaligned(reg).wrapping_add(data));
            }
        };
        rules[AddRegisterImmediateShort as usize] = |thread| {
            let register = thread.last::<u8>();
            let data = thread.last::<u16>();
            unsafe {
                let reg = ((&mut thread.registers[(register & 0x0f) as usize]) as *mut _ as *mut u16);
                write_unaligned(reg, read_unaligned(reg).wrapping_add(data));
            }
        };
        rules[AddRegisterImmediateInt as usize] = |thread| {
            let register = thread.last::<u8>();
            let data = thread.last::<u32>();
            unsafe {
                let reg = ((&mut thread.registers[(register & 0x0f) as usize]) as *mut _ as *mut u32);
                write_unaligned(reg, read_unaligned(reg).wrapping_add(data));
            }
        };
        rules[AddRegisterImmediateLong as usize] = |thread| {
            let register = thread.last::<u8>();
            let data = thread.last::<u64>();
            unsafe {
                let reg = ((&mut thread.registers[(register & 0x0f) as usize]) as *mut _ as *mut u64);
                write_unaligned(reg, read_unaligned(reg).wrapping_add(data));
            }
        };
        rules[SubtractRegisterImmediateByte as usize] = |thread| {
            let register = thread.last::<u8>();
            let data = thread.last::<u8>();
            unsafe {
                let reg = ((&mut thread.registers[(register & 0x0f) as usize]) as *mut _ as *mut u8);
                write_unaligned(reg, read_unaligned(reg).wrapping_sub(data));
            }
        };
        rules[SubtractRegisterImmediateShort as usize] = |thread| {
            let register = thread.last::<u8>();
            let data = thread.last::<u16>();
            unsafe {
                let reg = ((&mut thread.registers[(register & 0x0f) as usize]) as *mut _ as *mut u16);
                write_unaligned(reg, read_unaligned(reg).wrapping_sub(data));
            }
        };
        rules[SubtractRegisterImmediateInt as usize] = |thread| {
            let register = thread.last::<u8>();
            let data = thread.last::<u32>();
            unsafe {
                let reg = ((&mut thread.registers[(register & 0x0f) as usize]) as *mut _ as *mut u32);
                write_unaligned(reg, read_unaligned(reg).wrapping_sub(data));
            }
        };
        rules[SubtractRegisterImmediateLong as usize] = |thread| {
            let register = thread.last::<u8>();
            let data = thread.last::<u64>();
            unsafe {
                let reg = ((&mut thread.registers[(register & 0x0f) as usize]) as *mut _ as *mut u64);
                write_unaligned(reg, read_unaligned(reg).wrapping_sub(data));
            }
        };
        rules[DecrementRegister as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            thread.registers[register as usize] = thread.registers[register as usize].wrapping_sub(1);
        };
        rules[IncrementRegister as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            thread.registers[register as usize] = thread.registers[register as usize].wrapping_add(1);
        };
        rules[CompareRegisterByte as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = thread.registers[((registers & 0xf0) >> 4) as usize] as u8;
            let r2 = thread.registers[(registers & 0x0f) as usize] as u8;
            thread.alu_flags = 0
                | if (r1==r2) {ALUFlags::Equal as u8} else {0}
                | if (r1>r2) {ALUFlags::Greater as u8} else {0}
                | if (r1<r2) {ALUFlags::Lesser as u8} else {0}
                | if (r1==0) {ALUFlags::Zero as u8} else {0};
        };
        rules[CompareRegisterShort as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = thread.registers[((registers & 0xf0) >> 4) as usize] as u16;
            let r2 = thread.registers[(registers & 0x0f) as usize] as u16;
            thread.alu_flags = 0
                | if (r1==r2) {ALUFlags::Equal as u8} else {0}
                | if (r1>r2) {ALUFlags::Greater as u8} else {0}
                | if (r1<r2) {ALUFlags::Lesser as u8} else {0}
                | if (r1==0) {ALUFlags::Zero as u8} else {0};
        };
        rules[CompareRegisterInt as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = thread.registers[((registers & 0xf0) >> 4) as usize] as u32;
            let r2 = thread.registers[(registers & 0x0f) as usize] as u32;
            thread.alu_flags = 0
                | if (r1==r2) {ALUFlags::Equal as u8} else {0}
                | if (r1>r2) {ALUFlags::Greater as u8} else {0}
                | if (r1<r2) {ALUFlags::Lesser as u8} else {0}
                | if (r1==0) {ALUFlags::Zero as u8} else {0};
        };
        rules[CompareRegisterLong as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = thread.registers[((registers & 0xf0) >> 4) as usize] as u64;
            let r2 = thread.registers[(registers & 0x0f) as usize] as u64;
            thread.alu_flags = 0
                | if (r1==r2) {ALUFlags::Equal as u8} else {0}
                | if (r1>r2) {ALUFlags::Greater as u8} else {0}
                | if (r1<r2) {ALUFlags::Lesser as u8} else {0}
                | if (r1==0) {ALUFlags::Zero as u8} else {0};
        };
        rules[CompareRegisterLiteralByte as usize] = |thread| {
            let register = thread.registers[thread.last::<u8>() as usize] as u8;
            let data = thread.last::<u8>();
            thread.alu_flags = 0
                | if (register==data) {ALUFlags::Equal as u8} else {0}
                | if (register>data) {ALUFlags::Greater as u8} else {0}
                | if (register<data) {ALUFlags::Lesser as u8} else {0}
                | if (register==0) {ALUFlags::Zero as u8} else {0};
        };
        rules[CompareRegisterLiteralShort as usize] = |thread| {
            let register = thread.registers[thread.last::<u8>() as usize] as u16;
            let data = thread.last::<u16>();
            thread.alu_flags = 0
                | if (register==data) {ALUFlags::Equal as u8} else {0}
                | if (register>data) {ALUFlags::Greater as u8} else {0}
                | if (register<data) {ALUFlags::Lesser as u8} else {0}
                | if (register==0) {ALUFlags::Zero as u8} else {0};
        };
        rules[CompareRegisterLiteralInt as usize] = |thread| {
            let register = thread.registers[thread.last::<u8>() as usize] as u32;
            let data = thread.last::<u32>();
            thread.alu_flags = 0
                | if (register==data) {ALUFlags::Equal as u8} else {0}
                | if (register>data) {ALUFlags::Greater as u8} else {0}
                | if (register<data) {ALUFlags::Lesser as u8} else {0}
                | if (register==0) {ALUFlags::Zero as u8} else {0};
        };
        rules[CompareRegisterLiteralLong as usize] = |thread| {
            let register = thread.registers[thread.last::<u8>() as usize] as u64;
            let data = thread.last::<u64>();
            thread.alu_flags = 0
                | if (register==data) {ALUFlags::Equal as u8} else {0}
                | if (register>data) {ALUFlags::Greater as u8} else {0}
                | if (register<data) {ALUFlags::Lesser as u8} else {0}
                | if (register==0) {ALUFlags::Zero as u8} else {0};
        };
        rules[JumpIfEqualTo as usize] = |thread| {
            let address = thread.last::<u64>();
            if (thread.alu_flags & ALUFlags::Equal as u8) != 0 {
                thread.registers[RegisterRoles::ProgramCounter as usize] = address;
            }
        };
        rules[JumpIfGreaterThan as usize] = |thread| {
            let address = thread.last::<u64>();
            if (thread.alu_flags & ALUFlags::Greater as u8) != 0 {
                thread.registers[RegisterRoles::ProgramCounter as usize] = address;
            }
        };
        rules[JumpIfLessThan as usize] = |thread| {
            let address = thread.last::<u64>();
            if (thread.alu_flags & ALUFlags::Lesser as u8) != 0 {
                thread.registers[RegisterRoles::ProgramCounter as usize] = address;
            }
        };
        rules[JumpIfZero as usize] = |thread| {
            let address = thread.last::<u64>();
            if (thread.alu_flags & ALUFlags::Zero as u8) != 0 {
                thread.registers[RegisterRoles::ProgramCounter as usize] = address;
            }
        };
        rules[JumpTo as usize] = |thread| {
            let address = thread.last::<u64>();
            thread.registers[RegisterRoles::ProgramCounter as usize] = address;
        };
        rules[MoveRegistersByte as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = (registers & 0x0f);
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u8;
            unsafe {
                write_unaligned(data, (thread.registers[r1 as usize] as u8) as u8);
            }
        };
        rules[MoveRegistersShort as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = (registers & 0x0f);
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u16;
            unsafe {
                write_unaligned(data, (thread.registers[r1 as usize] as u16) as u16);
            }
        };
        rules[MoveRegistersInt as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = (registers & 0x0f);
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u32;
            unsafe {
                write_unaligned(data, (thread.registers[r1 as usize] as u32) as u32);
            }
        };
        rules[MoveRegistersLong as usize] = |thread| {
            let registers = thread.last::<u8>();
            let r1 = (registers & 0xf0) >> 4;
            let r2 = (registers & 0x0f);
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u64;
            unsafe {
                write_unaligned(data, (thread.registers[r1 as usize] as u64) as u64);
            }
        };
        rules[PushRegisterByte as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let reg_val = thread.registers[register as usize];
            push_byte_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], reg_val as u8);
        };
        rules[PushRegisterShort as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let reg_val = thread.registers[register as usize];
            push_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], reg_val as u16);
        };
        rules[PushRegisterInt as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let reg_val = thread.registers[register as usize];
            push_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], reg_val as u32);
        };
        rules[PushRegisterLong as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let reg_val = thread.registers[register as usize];
            push_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], reg_val);
        };
        rules[PopRegisterByte as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            unsafe {
                let location = (&mut thread.registers[register as usize]) as *mut u64 as *mut u8 as usize;
                pop_byte_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], &mut *(location as *mut u8));
            }
        };
        rules[PopRegisterShort as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            unsafe {
                let location = &mut thread.registers[register as usize] as *mut _ as *mut u16;
                pop_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], &mut *location);
            }
        };
        rules[PopRegisterInt as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            unsafe {
                let location = &mut thread.registers[register as usize] as *mut _ as *mut u32;
                pop_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], &mut *location);
            }
        };
        rules[PopRegisterLong as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            unsafe {
                let location = &mut thread.registers[register as usize] as *mut _ as *mut u64;
                pop_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], &mut *location);
            }
        };
        rules[MoveMemoryRegisterByte as usize] = |thread| {
            let address = thread.last::<u64>();
            let register = thread.last::<u8>() & 0x0f;
            let base_pointer = &thread.parent.as_ref().instructions[0] as *const u8 as u64;
            unsafe {
                write_unaligned(&mut thread.registers[register as usize] as *mut _ as *mut u8, read_unaligned((base_pointer.wrapping_add(address)) as *const u8));
            }
        };
        rules[MoveMemoryRegisterShort as usize] = |thread| {
            let address = thread.last::<u64>();
            let register = thread.last::<u8>() & 0x0f;
            let base_pointer = &thread.parent.as_ref().instructions[0] as *const u8 as u64;
            unsafe {
                write_unaligned(&mut thread.registers[register as usize] as *mut _ as *mut u16, read_unaligned((base_pointer.wrapping_add(address)) as *const u16));
            }
        };
        rules[MoveMemoryRegisterInt as usize] = |thread| {
            let address = thread.last::<u64>();
            let register = thread.last::<u8>() & 0x0f;
            let base_pointer = &thread.parent.as_ref().instructions[0] as *const u8 as u64;
            unsafe {
                write_unaligned(&mut thread.registers[register as usize] as *mut _ as *mut u32, read_unaligned((base_pointer.wrapping_add(address)) as *const u32));
            }
        };
        rules[MoveMemoryRegisterLong as usize] = |thread| {
            let address = thread.last::<u64>();
            let register = thread.last::<u8>() & 0x0f;
            let base_pointer = &thread.parent.as_ref().instructions[0] as *const u8 as u64;
            unsafe {
                write_unaligned(&mut thread.registers[register as usize] as *mut _ as *mut u64, read_unaligned((base_pointer.wrapping_add(address)) as *const u64));
            }
        };
        rules[MoveRegisterMemoryByte as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let address = thread.last::<u64>();
            let base_pointer = &thread.parent.as_ref().instructions[0] as *const u8 as u64;
            unsafe {
                write_unaligned((base_pointer.wrapping_add(address)) as *mut u8, read_unaligned(&mut thread.registers[register as usize] as *mut _ as *mut u8));
            }
        };
        rules[MoveRegisterMemoryShort as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let address = thread.last::<u64>();
            let base_pointer = &thread.parent.as_ref().instructions[0] as *const u8 as u64;
            unsafe {
                write_unaligned((base_pointer.wrapping_add(address)) as *mut u16, read_unaligned(&mut thread.registers[register as usize] as *mut _ as *mut u16));
            }
        };
        rules[MoveRegisterMemoryInt as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let address = thread.last::<u64>();
            let base_pointer = &thread.parent.as_ref().instructions[0] as *const u8 as u64;
            unsafe {
                write_unaligned((base_pointer.wrapping_add(address)) as *mut u32, read_unaligned(&mut thread.registers[register as usize] as *mut _ as *mut u32));
            }
        };
        rules[MoveRegisterMemoryLong as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let address = thread.last::<u64>();
            let base_pointer = &thread.parent.as_ref().instructions[0] as *const u8 as u64;
            unsafe {
                write_unaligned((base_pointer.wrapping_add(address)) as *mut u64, read_unaligned(&mut thread.registers[register as usize] as *mut _ as *mut u64));
            }
        };
        rules[PushMemoryByte as usize] = |thread| {
            let address = thread.last::<u64>();
            unsafe {
                let data = read_unaligned((&thread.parent.as_ref().instructions[0] as *const _ as u64).wrapping_add(address) as *const u64);
                push_byte_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], data as u8);
            }
        };
        rules[PushMemoryShort as usize] = |thread| {
            let address = thread.last::<u64>();
            unsafe {
                let data = read_unaligned((&thread.parent.as_ref().instructions[0] as *const _ as u64).wrapping_add(address) as *const u64);
                push_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], data as u16);
            }
        };
        rules[PushMemoryInt as usize] = |thread| {
            let address = thread.last::<u64>();
            unsafe {
                let data = read_unaligned((&thread.parent.as_ref().instructions[0] as *const _ as u64).wrapping_add(address) as *const u64);
                push_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], data as u32);
            }
        };
        rules[PushMemoryLong as usize] = |thread| {
            let address = thread.last::<u64>();
            unsafe {
                let data = read_unaligned((&thread.parent.as_ref().instructions[0] as *const _ as u64).wrapping_add(address) as *const u64);
                push_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], data as u64);
            }
        };
        rules[PopMemoryByte as usize] = |thread| {
            let address = thread.last::<u64>();
            unsafe {
                let address = ((&thread.parent.as_ref().instructions[0] as *const _ as u64).wrapping_add(address) as *mut u8);
                pop_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], &mut read_unaligned(address));
            }
        };
        rules[PopMemoryShort as usize] = |thread| {
            let address = thread.last::<u64>();
            unsafe {
                let address = ((&thread.parent.as_ref().instructions[0] as *const _ as u64).wrapping_add(address) as *mut u16);
                pop_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], &mut read_unaligned(address));
            }
        };
        rules[PopMemoryInt as usize] = |thread| {
            let address = thread.last::<u64>();
            unsafe {
                let address = ((&thread.parent.as_ref().instructions[0] as *const _ as u64).wrapping_add(address) as *mut u32);
                pop_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], &mut read_unaligned(address));
            }
        };
        rules[PopMemoryLong as usize] = |thread| {
            let address = thread.last::<u64>();
            unsafe {
                let address = ((&thread.parent.as_ref().instructions[0] as *const _ as u64).wrapping_add(address) as *mut u64);
                pop_stack(&mut thread.stack, &mut thread.registers[RegisterRoles::StackPointer as usize], &mut read_unaligned(address));
            }
        };
        rules[BitwiseAndRegistersByte as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u8;
            unsafe {
                write_unaligned(data, read_unaligned(data) & (thread.registers[r1 as usize] as u8));
            }
        };
        rules[BitwiseOrRegistersByte as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u8;
            unsafe {
                write_unaligned(data, read_unaligned(data) | (thread.registers[r1 as usize] as u8));
            }
        };
        rules[BitwiseXOrRegistersByte as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u8;
            unsafe {
                write_unaligned(data, read_unaligned(data) ^ (thread.registers[r1 as usize] as u8));
            }
        };
        rules[BitwiseAndRegistersShort as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u16;
            unsafe {
                write_unaligned(data, read_unaligned(data) & (thread.registers[r1 as usize] as u16));
            }
        };
        rules[BitwiseOrRegistersShort as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u16;
            unsafe {
                write_unaligned(data, read_unaligned(data) | (thread.registers[r1 as usize] as u16));
            }
        };
        rules[BitwiseXOrRegistersShort as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u16;
            unsafe {
                write_unaligned(data, read_unaligned(data) ^ (thread.registers[r1 as usize] as u16));
            }
        };
        rules[BitwiseAndRegistersInt as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u32;
            unsafe {
                write_unaligned(data, read_unaligned(data) & (thread.registers[r1 as usize] as u32));
            }
        };
        rules[BitwiseOrRegistersInt as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u32;
            unsafe {
                write_unaligned(data, read_unaligned(data) | (thread.registers[r1 as usize] as u32));
            }
        };
        rules[BitwiseXOrRegistersInt as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u32;
            unsafe {
                write_unaligned(data, read_unaligned(data) ^ (thread.registers[r1 as usize] as u32));
            }
        };
        rules[BitwiseAndRegistersLong as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u64;
            unsafe {
                write_unaligned(data, read_unaligned(data) & (thread.registers[r1 as usize] as u64));
            }
        };
        rules[BitwiseOrRegistersLong as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u64;
            unsafe {
                write_unaligned(data, read_unaligned(data) | (thread.registers[r1 as usize] as u64));
            }
        };
        rules[BitwiseXOrRegistersLong as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let data = &mut thread.registers[r2 as usize] as *mut _ as *mut u64;
            unsafe {
                write_unaligned(data, read_unaligned(data) ^ (thread.registers[r1 as usize] as u64));
            }
        };
        rules[BitwiseNotRegisterByte as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = &mut thread.registers[register as usize] as *mut _ as *mut u8;
            unsafe {
                write_unaligned(data, !read_unaligned(data));
            };
        };
        rules[BitwiseNotRegisterShort as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = &mut thread.registers[register as usize] as *mut _ as *mut u16;
            unsafe {
                write_unaligned(data, !read_unaligned(data));
            };
        };
        rules[BitwiseNotRegisterInt as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = &mut thread.registers[register as usize] as *mut _ as *mut u32;
            unsafe {
                write_unaligned(data, !read_unaligned(data));
            };
        };
        rules[BitwiseNotRegisterLong as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = &mut thread.registers[register as usize] as *mut _ as *mut u64;
            unsafe {
                write_unaligned(data, !read_unaligned(data));
            };
        };
        rules[BitwiseAndRegisterImmediateByte as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u8>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u8;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) & data);
            };
        };
        rules[BitwiseOrRegisterImmediateByte as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u8>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u8;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) | data);
            };
        };
        rules[BitwiseXOrRegisterImmediateByte as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u8>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u8;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) ^ data);
            };
        };
        rules[BitwiseAndRegisterImmediateShort as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u16>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u16;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) & data);
            };
        };
        rules[BitwiseOrRegisterImmediateShort as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u16>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u16;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) | data);
            };
        };
        rules[BitwiseXOrRegisterImmediateShort as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u16>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u16;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) ^ data);
            };
        };
        rules[BitwiseAndRegisterImmediateInt as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u32>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u32;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) & data);
            };
        };
        rules[BitwiseOrRegisterImmediateInt as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u32>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u32;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) | data);
            };
        };
        rules[BitwiseXOrRegisterImmediateInt as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u32>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u32;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) ^ data);
            };
        };
        rules[BitwiseAndRegisterImmediateLong as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u64>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u64;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) & data);
            };
        };
        rules[BitwiseOrRegisterImmediateLong as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u64>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u64;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) | data);
            };
        };
        rules[BitwiseXOrRegisterImmediateLong as usize] = |thread| {
            let register = thread.last::<u8>() & 0x0f;
            let data = thread.last::<u64>();
            let to_modify = &mut thread.registers[register as usize] as *mut _ as *mut u64;
            unsafe {
                write_unaligned(to_modify, read_unaligned(to_modify) ^ data);
            };
        };
        rules[MoveAddressedRegisterRegisterByte as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned(val2 as *mut u8, read_unaligned((read_unaligned(val1).wrapping_add(base(thread))) as *const u8));
            }
        };
        rules[MoveAddressedRegisterRegisterShort as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned(val2 as *mut u16, read_unaligned((read_unaligned(val1).wrapping_add(base(thread))) as *const u16));
            }
        };
        rules[MoveAddressedRegisterRegisterInt as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned(val2 as *mut u32, read_unaligned((read_unaligned(val1).wrapping_add(base(thread))) as *const u32));
            }
        };
        rules[MoveAddressedRegisterRegisterLong as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned(val2 as *mut u64, read_unaligned((read_unaligned(val1).wrapping_add(base(thread))) as *const u64));
            }
        };
        rules[MoveRegisterAddressedRegisterByte as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned((read_unaligned(val2).wrapping_add(base(thread))) as *mut u8, read_unaligned(val1 as *const u8));
            }
        };
        rules[MoveRegisterAddressedRegisterShort as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned((read_unaligned(val2).wrapping_add(base(thread))) as *mut u16, read_unaligned(val1 as *const u16));
            }
        };
        rules[MoveRegisterAddressedRegisterInt as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned((read_unaligned(val2).wrapping_add(base(thread))) as *mut u32, read_unaligned(val1 as *const u32));
            }
        };
        rules[MoveRegisterAddressedRegisterLong as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned((read_unaligned(val2).wrapping_add(base(thread))) as *mut u64, read_unaligned(val1 as *const u64));
            }
        };
        rules[MoveAddressedRegistersByte as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned((read_unaligned(val2).wrapping_add(base(thread))) as *mut u8, read_unaligned((read_unaligned(val1) + base(thread)) as *const u8));
            }
        };
        rules[MoveAddressedRegistersShort as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned((read_unaligned(val2).wrapping_add(base(thread))) as *mut u16, read_unaligned((read_unaligned(val1) + base(thread)) as *const u16));
            }
        };
        rules[MoveAddressedRegistersInt as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned((read_unaligned(val2).wrapping_add(base(thread))) as *mut u32, read_unaligned((read_unaligned(val1) + base(thread)) as *const u32));
            }
        };
        rules[MoveAddressedRegistersLong as usize] = |thread| {
            let registers = thread.last::<u8>();
            let (r1, r2) = ((registers & 0xf0) >> 4, (registers & 0x0f));
            let (val1, val2) = (&mut thread.registers[r1 as usize] as *mut u64, &mut thread.registers[r2 as usize] as *mut u64);
            unsafe {
                write_unaligned((read_unaligned(val2).wrapping_add(base(thread))) as *mut u64, read_unaligned((read_unaligned(val1) + base(thread)) as *const u64));
            }
        };
        rules
    }
    pub fn get_syscalls() -> [fn(&mut crate::virtual_thread::VirtualThread) -> (); SysCalls::__END__ as usize] {
        use arsenal_globals::SysCalls::*;

        let mut syscalls = [(|_|{}) as fn(&mut VirtualThread) -> (); __END__ as usize];
        syscalls[PrintRegister as usize] = |thread| {
            print!("{}", thread.registers[thread.registers[0] as usize]);
        };
        syscalls[PrintRegisterSigned as usize] = |thread| {
            print!("{}", thread.registers[thread.registers[0] as usize] as i64);
        };
        syscalls[PrintCString as usize] = |thread| {
            unsafe {
                use std::ffi::{CStr, c_char};
                let ptr = (thread.registers[0].wrapping_add(&thread.parent.instructions[0] as *const _ as *const u8 as u64)) as usize as *const c_char;
                print!("{}", CStr::from_ptr(ptr).to_str().expect(&format!("error parsing {}", *ptr)));
            }
        };
        syscalls[MemoryAllocate as usize] = |thread| {
            use std::alloc::{alloc, Layout};
            let size = thread.registers[0];
            let layout = Layout::from_size_align(size.try_into().unwrap(), std::mem::align_of::<u8>()).unwrap();
            unsafe {
                thread.registers[0] = (alloc(layout) as u64).wrapping_sub(base(thread));
            }
        };
        syscalls[MemoryFree as usize] = |thread| {
            use std::alloc::{dealloc, Layout};
            let (ptr, size) = (thread.registers[0], thread.registers[1]);
            let layout = Layout::from_size_align(size.try_into().unwrap(), std::mem::align_of::<u8>()).unwrap();
            unsafe {
                dealloc((ptr.wrapping_add(base(thread))) as *mut u8, layout);
            }
        };
        syscalls[FOpen as usize] = |thread| {
            use std::ffi::c_char;
            let ptr1 = (thread.registers[0].wrapping_add(base(thread))) as usize as *const c_char;
            let ptr2 = (thread.registers[1].wrapping_add(base(thread))) as usize as *const c_char;
            unsafe {
                let ptr = libc::fopen(ptr1, ptr2);
                thread.registers[0] = (ptr as u64).wrapping_sub(base(thread));
            }
        };
        syscalls[FClose as usize] = |thread| {
            use std::ffi::c_char;
            let ptr1 = (thread.registers[0].wrapping_add(&thread.parent.instructions[0] as *const _ as *const u8 as u64)) as usize;
            unsafe {
                let ptr = libc::fclose((ptr1 as u64) as *mut libc::FILE);
            }
        };
        syscalls[FGetC as usize] = |thread| {
            use std::ffi::c_char;
            let ptr1 = (thread.registers[0].wrapping_add(&thread.parent.instructions[0] as *const _ as *const u8 as u64)) as usize;
            unsafe {
                thread.registers[1] = libc::fgetc(ptr1 as *mut libc::FILE) as u64;
            }
        };
        syscalls[FTell as usize] = |thread| {
            use std::ffi::c_char;
            let ptr1 = (thread.registers[0].wrapping_add(&thread.parent.instructions[0] as *const _ as *const u8 as u64)) as usize;
            unsafe {
                thread.registers[1] = libc::ftell((ptr1 as u64) as *mut libc::FILE) as u64;
            }
        };
        syscalls[FSeek as usize] = |thread| {
            use std::ffi::c_char;
            let ptr1 = (thread.registers[0].wrapping_add(&thread.parent.instructions[0] as *const _ as *const u8 as u64)) as usize;
            let offset = thread.registers[1];
            let whence = thread.registers[2];
            unsafe {
                thread.registers[1] = libc::fseek((ptr1 as u64) as *mut libc::FILE, offset as i64 as i32, whence as i32) as u64;
            }
        };
        syscalls
    }
}

fn push_byte_stack(stack: &mut Vec<u8>, sp: &mut u64, byte: u8) {
    while stack.len() <= (*sp).try_into().unwrap() {
        stack.push(0xff);
    }
    unsafe {
        *(((&mut stack[0] as *mut u8 as u64) + *sp) as *mut u8) = byte;
    }
    *sp += 1;
}

fn push_stack<T>(stack: &mut Vec<u8>, sp: &mut u64, data: T) {
    let t_size = std::mem::size_of::<T>();
    for i in 0..t_size {
        let byte = ((&(data) as *const _ as usize) + i) as *mut u8;
        unsafe {
            push_byte_stack(stack, sp, *byte);
        }
    }
}

fn pop_byte_stack(stack: &mut Vec<u8>, sp: &mut u64, location: &mut u8) {
    if *sp < (stack.len() as u64 / 2) {
        stack.shrink_to(stack.len() / 2);
    }
    *sp -= 1;
    unsafe {
        *location = *(((&mut stack[0] as *mut u8 as u64) + *sp) as *mut u8);
    }
}

fn pop_stack<T>(stack: &mut Vec<u8>, sp: &mut u64, location: &mut T) {
    let t_size = std::mem::size_of::<T>();
    for i in (0..t_size) {
        unsafe {
            pop_byte_stack(stack, sp, &mut *(((location as *mut _ as usize) + ((t_size - i) - 1)) as *mut u8))
        }
    }
}

fn base(thread: &VirtualThread) -> u64 {
    &thread.parent.as_ref().instructions[0] as *const _ as u64
}
