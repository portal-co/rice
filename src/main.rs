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
    let mut lx = None;
    for l in s.lines() {
        if go {
            writeln!(file, "{l}")?;
        }
        macro_rules! resolve {
            ($c:expr) => {
                match $c {
                    c => match std::fs::File::open(c) {
                        Ok(mut file2) => {
                            let mut a = [0u8];
                            while file2.read(&mut a)? == 1 {
                                file.write_all(&a)?;
                            }
                        }
                        Err(file_error) => {}
                    },
                }
            };
        }
        if let Some(l) = l.strip_prefix("@") {
            writeln!(file, "[[begin {l}]]")?;
            resolve!(l.trim());
            writeln!(file, "[[end]]")?;
            lx = Some(l);
            continue;
        }
        if let Some((_, b2)) = l.split_once("[[")
            && let Some((b, _)) = b2.rsplit_once("]]")
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
                    Some(c) if go && lx.take().is_none_or(|a| a != c) => {
                        resolve!(c.trim());
                        go = false;
                    }
                    _ => {}
                },
            }
        }
        lx = None;
    }
    Ok(())
}
