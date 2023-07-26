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
        let _ = Box::from_raw(lib as *mut Library);
    }
}

pub unsafe fn get_symbol_address(lib: *const Library, sym: &str) -> *const c_void {
    if lib.is_null() {
        return 0 as *const c_void;
    }

    unsafe {
        let library = &*(lib as *const Library);

        match library.get::<*const c_void>(sym.as_bytes()) {
            Ok(func) => *func as *const c_void,
            Err(_) => 0 as *const c_void,
        }
    }
}

pub unsafe fn call_c_function(func: *const c_void, buffer: *const u8, buff_size: u64) -> *const c_void {
    assert!(buff_size % 8 == 0);
    let mut arguments = vec![];

    for i in 0..(buff_size / 8) {
        unsafe {
            arguments.push(arg(
                &*((buffer as u64 + i as u64) as *const u64)
            ))
        }
    }

    unsafe {
        call::<*const c_void>(CodePtr(func as u64 as *mut c_void), &arguments[..])
    }
}
