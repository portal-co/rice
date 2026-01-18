use std::io::Write;

#[cfg(feature = "zip")]
use std::io::{Read, Seek};

/// A trait for resolving inclusions.
pub trait Resolver {
    fn resolve(&mut self, path: &str, out: &mut (dyn Write + '_)) -> std::io::Result<()>;
}

impl<F> Resolver for F
where
    F: FnMut(&str, &mut (dyn Write + '_)) -> std::io::Result<()>,
{
    fn resolve(&mut self, path: &str, out: &mut (dyn Write + '_)) -> std::io::Result<()> {
        self(path, out)
    }
}

/// Default resolver that reads from the filesystem.
pub struct FileResolver;

impl Resolver for FileResolver {
    fn resolve(&mut self, path: &str, out: &mut (dyn Write + '_)) -> std::io::Result<()> {
        if let Ok(mut f) = std::fs::File::open(path) {
            std::io::copy(&mut f, out)?;
        }
        Ok(())
    }
}

/// Resolver that reads from a zip archive.
#[cfg(feature = "zip")]
pub struct ZipResolver<R: Read + Seek> {
    archive: zip::ZipArchive<R>,
}

#[cfg(feature = "zip")]
impl<R: Read + Seek> ZipResolver<R> {
    pub fn new(reader: R) -> zip::result::ZipResult<Self> {
        Ok(Self {
            archive: zip::ZipArchive::new(reader)?,
        })
    }
}

#[cfg(feature = "zip")]
impl<R: Read + Seek> Resolver for ZipResolver<R> {
    fn resolve(&mut self, path: &str, out: &mut (dyn Write + '_)) -> std::io::Result<()> {
        if let Ok(mut file) = self.archive.by_name(path) {
            std::io::copy(&mut file, out)?;
        }
        Ok(())
    }
}

/// Run splice on a string, writing the result to a file-like object.
pub fn splice<W: Write>(s: &str, file: &mut W) -> std::io::Result<()> {
    splice_with(s, file, FileResolver)
}

/// Splicing with a custom resolver.
pub fn splice_with<W, R>(s: &str, file: &mut W, mut resolver: R) -> std::io::Result<()>
where
    W: Write,
    R: Resolver,
{
    let mut go = true;
    let mut lx = None;
    for l in s.lines() {
        if go {
            writeln!(file, "{l}")?;
        }
        if let Some(path) = l.strip_prefix("@") {
            writeln!(file, "[[begin {path}]]")?;
            resolver.resolve(path.trim(), file)?;
            writeln!(file, "[[end]]")?;
            lx = Some(path);
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
                        resolver.resolve(c.trim(), file)?;
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
