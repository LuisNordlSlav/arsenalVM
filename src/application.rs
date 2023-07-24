pub enum AppAction {
    Run,
    CompileRun,
    CompileExecutable,
    Null,
}

pub struct AppState {
    pub input_file: String,
    pub output_file: String,
    pub action: AppAction,
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

    AppState {
        input_file: input,
        output_file: output,
        action: action,
    }
}
