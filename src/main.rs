mod app;
mod audio;
mod cli;
mod tap;
mod tempo;
mod ui;

use crate::cli::Cli;
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    if let Err(error) = app::run(cli) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
