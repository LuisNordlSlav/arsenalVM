#![allow(unused)]

#![feature(specialization)]

use std::any::TypeId;
use std::fmt::Display;

macro_rules! _instructions {
    ($($(label)? $name:ident $(: $($byte:expr),+)?;)*) => 
    {
        {
            let mut return_value = vec![];
            $(
                return_value.push($name as u8);
                return_value.push(0);
                $($(
                    return_value.extend(vec![$byte as u8]);
                )+)?
            )*
            return_value
        }
    }
}

// use instructions_macro::parse_data;

use arsenal_assembler::instructions::{Instructions::*, SysCalls::*};

pub fn get_instructions() -> Vec<u8> {
    let instructions_ = _instructions! (
        NoOperation; // do nothing

        LoadRegisterByte: 5, 0; // load 0 into r5
        AddRegisterImmediateByte: 5, 0; // add immediate 0 into r5
        IncrementRegister: 5; // increment r5

        // system call stuff
        LoadRegisterByte: 0, 5; // load register to print into r0
        SysCall: PrintRegister; // call `PrintRegisters` SysCall

        LoadRegisterByte: 15, 30; // jump over print syscall.

        SysCall: PrintRegister; // prints the register to know if the condition fires
        IncrementRegister: 5; // increment to reset condition to false

        CompareRegisterByte: 0x50; // compare r5 and r0
        JumpIfLessThan: 24,0,0,0,0,0,0,0; // jump to above syscall // 43

        LoadRegisterByte: 6, 10; // counter

        LoadRegisterByte: 7, 1;  // r7 stores last last
        LoadRegisterByte: 8, 1;  // r8 stores last
        LoadRegisterByte: 9, 1;  // r9 stores current fibonacci number. // 59

        // beginning
        MoveRegistersLong: 0x87;
        MoveRegistersLong: 0x98;
        AddRegistersLong: 0x79; // 68

        DecrementRegister: 6;

        LoadRegisterByte: 0, 9;
        SysCall: PrintRegister; // 78

        CompareRegisterByte: 0x64;
        JumpIfEqualTo: 95,0,0,0,0,0,0,0; // 91
        LoadRegisterByte: 15, 59; // 95

        // end
        Halt; // end program
    );

    instructions_
}
