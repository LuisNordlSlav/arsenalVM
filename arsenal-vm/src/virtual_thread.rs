#![allow(unused)]

use std::{thread, time::Duration, ptr::read_unaligned, sync::Arc};
use crate::virtual_machine::*;

pub struct VirtualThread {
    pub parent: Arc<VirtualMachine>,
    pub registers: [u64; 16],
    pub running: bool,
    pub alu_flags: u8,
    pub stack: Vec<u8>,
}

impl VirtualThread {
    pub fn new(vm: &mut VirtualMachine, from: u64, name: String) -> thread::JoinHandle<()> {
        unsafe {
            let ptr = Arc::from_raw(vm as *mut _);

            thread::Builder::new().name(name).spawn(move || {
                unsafe {
                    let mut registers = [0; 16];
                    registers[RegisterRoles::ProgramCounter as usize] = from;
                    let mut instance = Self { 
                        parent: ptr,
                        registers,
                        running: true,
                        alu_flags: 0,
                        stack: vec![],
                    };
                    instance.run();
                };
            }).unwrap()
        }
    }

    pub fn next<T>(&mut self) -> T {
        self.registers[RegisterRoles::ProgramCounter as usize] += std::mem::size_of::<T>() as u64;
        self.current::<T>()
    }
    pub fn last<T>(&mut self) -> T {
        let ret = self.current::<T>();
        self.registers[RegisterRoles::ProgramCounter as usize] += std::mem::size_of::<T>() as u64;
        ret
    }
    pub fn current<T>(&self) -> T {
        unsafe {
            read_unaligned((&self.parent.as_ref().instructions[0] as *const u8 as usize).wrapping_add(self.registers[RegisterRoles::ProgramCounter as usize] as usize) as *const T)
        }
    }

    pub fn run(&mut self) {
        while self.running {
            unsafe {
                assert!(!(self.parent.as_ref().instructions.len() < self.registers[RegisterRoles::ProgramCounter as usize] as usize), "ran out of instructions at index {} of array of length {}", self.registers[RegisterRoles::ProgramCounter as usize], self.parent.as_ref().instructions.len());
                let instruction = self.last::<u16>();
                assert!(instruction < arsenal_globals::Instructions::__END__ as u16, "unidentified instruction id {} at {}", instruction, self.registers[RegisterRoles::ProgramCounter as usize]);
                self.parent.as_ref().rules[instruction as usize](self);
            }
        }
    }
}
