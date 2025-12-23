use std::io::Write;
use clap::Parser;

/// Simple CLI to run rice::splice on a file in-place.
#[derive(Debug, Parser)]
#[command(about = "Run rice splice on a file in-place")]
struct Args {
    /// File to process
    file: std::path::PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let file = args.file;
    let s = std::fs::read_to_string(&file)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&file)?;
    rice::splice(&s, &mut file)?;
    Ok(())
}
