mod lexer;
mod tokens;
// stdlib imports
use std::path::PathBuf;
use std::{fs::read_to_string, path::Path};
// external lib imports
use anyhow::{anyhow, Context, Result}; // error handling
use clap::Parser; // argument parsing
use rustyline::error::ReadlineError;
use rustyline::Editor;
// internal imports
use lexer::Lexer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    script: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.script {
        Some(script_path) => run_interpreter(&script_path),
        None => run_repl(),
    }?;
    Ok(())
}

fn run_repl() -> Result<()> {
    println!("starting REPL!");
    let mut rl = Editor::<()>::new()?;
    // main repl loop
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

fn run_interpreter(script_path: &Path) -> Result<()> {
    let string: String = read_to_string(script_path).context("could not open script file")?;
    println!("{string}");
    Ok(())
}
