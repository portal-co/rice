use clap::Parser;
use embedded_io_adapters::std::FromStd;

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
    let raw = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&file)?;
    let mut file = FromStd::new(raw);
    rice::splice(&s, &mut file)?;
    Ok(())
}
