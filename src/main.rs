#![allow(unused)]
#![allow(non_snake_case)]
extern crate arsenal_assembler;
extern crate libc;

pub mod virtual_machine;
pub mod virtual_thread;
pub mod debug;
pub mod instructions_macros;
pub mod application;

use application::AppAction::*;

use std::fs::{read, write};
use std::env::args;

fn main() {

    let arguments: Vec<String> = args().collect();
    let state = application::parse_args(arguments);

    match state.action {
        CompileRun => {
            let data = read(&state.input_file).expect(&format!("Error opening file {}: no such file", state.input_file));
            let result = arsenal_assembler::parse(data).expect(&format!("failed to parse {}", state.input_file));
            let mut vm = virtual_machine::VirtualMachine::new(result);
            vm.run();
        },
        CompileExecutable => {
            let data = read(&state.input_file).expect(&format!("Error opening file {}: no such file", state.input_file));
            let result = arsenal_assembler::parse(data).expect(&format!("failed to parse {}", state.input_file));
            write(state.output_file, result);
        },
        Run => {
            let data = read(&state.input_file).expect(&format!("Error opening file {}: no such file", state.input_file));
            let mut vm = virtual_machine::VirtualMachine::new(data);
            vm.run();
        },
        Null => panic!("input file required"),
    }
}
