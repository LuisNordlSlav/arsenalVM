#[derive(Debug)]
pub enum AppAction {
    Run,
    CompileRun,
    CompileExecutable,
    Null,
}

#[derive(Debug)]
pub struct AppState {
    pub input_file: String,
    pub output_file: String,
    pub action: AppAction,
    pub base: String,
}

pub fn parse_args(args: Vec<String>) -> AppState {
    let mut input = "in.ars".to_string();
    let mut output = "out.arc".to_string();
    let mut action = AppAction::Null;

    let mut arg_iter = args[1..].iter().peekable();

    while let Some(arg) = arg_iter.next() {
        if arg.starts_with("-") {
            match arg.as_str() {
                "-o" => {
                    output = arg_iter.next().expect("expected file after -o").to_string();
                    if let AppAction::Null = action { action = AppAction::CompileRun }
                },
                "-c" => { action = AppAction::CompileExecutable; },
                _ => {},
            }
        }
        else {
            input = arg.to_string();
            if let AppAction::Null = action {
                if arg.ends_with(".ars") {
                    action = AppAction::CompileRun;
                } else if arg.ends_with(".arc") {
                    action = AppAction::Run;
                }
            }
        }
    }

    let base = match get_folder_path(&input) {
        Some(s) => s,
        None => "".to_string(),
    };

    let state = AppState {
        input_file: input,
        output_file: output,
        action: action,
        base: base,
    };
    state
}

use std::path::Path;

fn get_folder_path(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    if path.is_file() {
        match path.parent() {
            Some(parent_path) => Some(parent_path.canonicalize().ok()?.to_string_lossy().to_string()),
            None => None,
        }
    } else {
        None
    }
}


use std::{panic, env};

pub fn custom_panic_hook(info: &panic::PanicInfo) {
    if cfg!(not(debug_assertions)) {
        // If this is a release build, print a custom error message without backtrace.
        let error_message = extract_error_message(info);
        eprintln!("Error: {}", error_message);
    } else {
        // If this is a debug build, use the default panic behavior with backtrace.
        panic::take_hook()(info);
    }
}

fn extract_error_message(info: &panic::PanicInfo) -> String {
    if let Some(message) = info.payload().downcast_ref::<&str>() {
        // If the payload is a string reference, return it as the error message.
        message.to_string()
    } else if let Some(message) = info.payload().downcast_ref::<String>() {
        // If the payload is a `String`, return its reference as the error message.
        message.to_string()
    } else {
        // If the payload is of some other type, return a generic error message.
        "Unknown error occurred.".to_string()
    }
}

pub fn init_hook() {
    #[cfg(not(debug_assertions))]
    {
        panic::set_hook(Box::new(custom_panic_hook));
    }
}
