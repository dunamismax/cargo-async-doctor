use std::process::ExitCode;

use cargo_async_doctor::{cli, render, scan};

fn main() -> ExitCode {
    match try_main() {
        Ok(ExitCode::SUCCESS) => ExitCode::SUCCESS,
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn try_main() -> anyhow::Result<ExitCode> {
    match cli::try_parse_from(std::env::args_os()) {
        Ok(cli) => {
            let report = scan::scan(&cli)?;
            let output = render::render_report(cli.message_format, &report)?;
            println!("{output}");
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => {
            if error.use_stderr() {
                eprint!("{error}");
                Ok(ExitCode::from(2))
            } else {
                print!("{error}");
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}
