use anyhow::{Result, anyhow};
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use parser::generate;
use std::{fs, io::Write};

mod cli;
mod parser;
mod operations;
mod expr;
mod loops;
mod functions;

fn program() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Build { file, output }) => {
            let current_dir = std::env::current_dir()?;
            let file = current_dir.join(file);
            let code = fs::read_to_string(file)?;
            let parsed = generate(&code)?;
            let mut file = fs::File::create(output)?;
            file.write_all(parsed.as_bytes())?;
        }
        None => return Err(anyhow!("Command not found")),
    };

    Ok(())
}

fn main() {
    if let Err(e) = program() {
        eprintln!("{} {}", "Error:".red(), e);
    }
}
