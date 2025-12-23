use std::io::Write;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    args.next();
    let file = args.next().unwrap();
    let s = std::fs::read_to_string(&file)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file)?;
    rice::splice(&s, &mut file)?;
    Ok(())
}
