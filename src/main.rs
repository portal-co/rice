use std::io::{Read, Write};
fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    args.next();
    let file = args.next().unwrap();
    let s = std::fs::read_to_string(&file)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file)?;
    let mut go = true;
    for l in s.lines() {
        if go {
            writeln!(file, "{l}")?;
        }
        if let Some((_, b2)) = l.split_once("rice[[")
            && let Some((b, _)) = b2.rsplit_once("]]rice")
        {
            let b = b.trim();
            match &*b {
                "end" => {
                    if !go {
                        writeln!(file, "{l}")?;
                    }
                    go = true
                }
                b => match b.strip_prefix("begin ") {
                    Some(c) if go => match std::fs::File::open(c) {
                        Ok(mut file2) => {
                            let mut a = [0u8];
                            while file2.read(&mut a)? == 1 {
                                file.write_all(&a)?;
                            }
                            go = false;
                        },
                        Err(file_error) => {

                        }
                    },
                    _ => {}
                },
            }
        }
    }
    Ok(())
}
