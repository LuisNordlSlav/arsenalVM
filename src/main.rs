#![allow(unused)]
#![allow(non_snake_case)]
extern crate arsenal_assembler;
pub mod application;
use arsenal_linker::{link, encode, decode};

use application::AppAction::*;

use std::fs::{read, write};
use std::env::args;
use std::rc::Rc;

fn main() {
    application::init_hook();

    let arguments: Vec<String> = args().collect();
    let state = application::parse_args(arguments);
    
    match state.action {
        CompileRun => {
            let data = read(&state.input_file).expect(&format!("Error opening file {}: no such file", state.input_file));
            let mut result = arsenal_assembler::new_parse(data).expect(&format!("failed to parse {}", state.input_file));
            let mut vm = arsenal_vm::virtual_machine::VirtualMachine::new(link(&mut result));
            vm.run();
        },
        CompileExecutable => {
            let data = read(&state.input_file).expect(&format!("Error opening file {}: no such file", state.input_file));
            let result = arsenal_assembler::new_parse(data).expect(&format!("failed to parse {}", state.input_file));
            write(state.output_file, encode(&result));
        },
        Run => {
            let mut data = read(&state.input_file).expect(&format!("Error opening file {}: no such file", state.input_file));
            let mut data = decode(data);
            let mut vm = arsenal_vm::virtual_machine::VirtualMachine::new(link(&mut data));
            vm.run();
        },
        Null => panic!("input file required"),
    }
}
