use std::process::ExitCode;

use cargo_async_doctor::{
    cli::{self, Command},
    explain, render, scan,
};

fn main() -> ExitCode {
    match try_main() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn try_main() -> anyhow::Result<ExitCode> {
    match cli::try_parse_from(std::env::args_os()) {
        Ok(cli) => match &cli.command {
            Some(Command::Explain(command)) => {
                let report = explain::explain(&command.check_id);
                let output = render::render_explain_report(cli.message_format, &report)?;
                println!("{output}");

                Ok(if report.found {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(2)
                })
            }
            None => {
                let report = scan::scan(&cli)?;
                let output = render::render_scan_report(cli.message_format, &report)?;
                println!("{output}");
                Ok(ExitCode::SUCCESS)
            }
        },
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
