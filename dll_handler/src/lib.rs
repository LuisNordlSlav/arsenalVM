#![allow(unused)]
#![allow(dead_code)]

use std::os::raw::c_void;
use libffi::high::*;
use libloading::Library;
use std::path::Path;

pub unsafe fn load_library_by_name(name: &str) -> *const Library {
    let library_file_name = if cfg!(windows) {
        format!("{}.dll", name)
    } else if cfg!(unix) {
        format!("lib{}.so", name)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", name)
    } else {
        return 0 as *const Library;
    };

    let library_path = Path::new(&library_file_name);
    match Library::new(library_path) {
        Ok(lib) => {
            let ptr = &lib as *const Library;
            std::mem::forget(lib);
            ptr
        },
        Err(_) => 0 as *const Library,
    }
}

pub unsafe fn delete_library(lib: *const Library) {
    if !lib.is_null() {
        // Convert the raw pointer back to a reference and let it drop when it goes out of scope
        unsafe { let _ = &*(lib as *const Library); }
    }
}

pub unsafe fn get_symbol_address(lib: *const Library, sym: &str) -> *const c_void {
    if lib.is_null() {
        return 0 as *const c_void;
    }

    unsafe {
        let library = &*(lib as *const Library);
        println!("symbol path: {}", sym);

        match library.get::<*const c_void>(sym.as_bytes()) {
            Ok(func) => *func as *const c_void,
            Err(_) => 0 as *const c_void,
        }
    }
}

use std::ptr::read_unaligned;

pub unsafe fn call_c_function_args(func: *const c_void, arguments_buffer: *const u8, arguments_count: u64, data_buffer: *const u8) -> *const c_void {
    if func as u64 == 0 as u64 {
        return 0 as *const c_void;
    }
    if arguments_count == 0 {
        unsafe {
            call::<()>(CodePtr(func as u64 as *mut c_void), &[]);
        }
        return 0 as *const c_void;
    }
    let mut data_vec = vec![];
    let mut arguments = vec![];

    unsafe {
        let mut buffer_offset = 0;
        let return_type_id = *arguments_buffer;
        match return_type_id {
            0 => {},
            1 => buffer_offset += 1,
            2 => buffer_offset += 2,
            3 => buffer_offset += 4,
            4 => buffer_offset += 8,
            5 => buffer_offset += std::mem::size_of::<usize>(),
            _ => unreachable!(),
        }
        let base_offset = buffer_offset as u64;
        for offset in 1..arguments_count {
            match *((arguments_buffer as u64 + offset as u64) as *const u8) {
                0 => {}, // void
                1 => {
                    data_vec.push(read_unaligned((data_buffer as u64 + buffer_offset as u64 - base_offset) as *const i8) as u64);
                    arguments.push(arg(
                        &*(&data_vec[data_vec.len() - 1] as *const _ as *const i8)
                    ));
                    buffer_offset += 1;
                }, // u8
                2 => {
                    data_vec.push(read_unaligned((data_buffer as u64 + buffer_offset as u64 - base_offset) as *const i16) as u64);
                    arguments.push(arg(
                        &*(&data_vec[data_vec.len() - 1] as *const _ as *const i16)
                    ));
                    buffer_offset += 2;
                }, // u16
                3 => {
                    data_vec.push(read_unaligned((data_buffer as u64 + buffer_offset as u64 - base_offset) as *const i32) as u64);
                    arguments.push(arg(
                        &*(&data_vec[data_vec.len() - 1] as *const _ as *const i32)
                    ));
                    buffer_offset += 4;
                }, // u32
                4 => {
                    data_vec.push(read_unaligned((data_buffer as u64 + buffer_offset as u64 - base_offset) as *const i64) as u64);
                    arguments.push(arg(
                        &*(&data_vec[data_vec.len() - 1] as *const _ as *const i64)
                    ));
                    buffer_offset += 8;
                }, // u64
                5 => {
                    data_vec.push(read_unaligned((data_buffer as u64 + buffer_offset as u64 - base_offset) as *const isize) as u64);
                    arguments.push(arg(
                        &*(&data_vec[data_vec.len() - 1] as *const _ as *const isize)
                    ));
                    buffer_offset += std::mem::size_of::<usize>();
                }, // usize
                _ => unreachable!(),
            }
        }

        match return_type_id {
            0 => { call::<()>(CodePtr(func as u64 as *mut c_void), &arguments[..]); 0 as *const c_void},
            1 => call::<i8>(CodePtr(func as u64 as *mut c_void), &arguments[..]) as *const c_void,
            2 => call::<i16>(CodePtr(func as u64 as *mut c_void), &arguments[..]) as *const c_void,
            3 => call::<i32>(CodePtr(func as u64 as *mut c_void), &arguments[..]) as *const c_void,
            4 => call::<i64>(CodePtr(func as u64 as *mut c_void), &arguments[..]) as *const c_void,
            5 => call::<isize>(CodePtr(func as u64 as *mut c_void), &arguments[..]) as *const c_void,
            _ => unreachable!(),
        }
    }
}

use std::os::raw::*;

extern "C" {
    pub fn LoadDLL(name: *const c_char) -> *const c_void;
    pub fn DeleteDLL(dll: *const c_void);
    pub fn LocateSymbol(dll: *const c_void, name: *const c_char) -> *const c_void;
}
