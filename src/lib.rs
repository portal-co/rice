#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "zip")]
use std::io::{Read, Seek};

use embedded_io::Write;

// ---------------------------------------------------------------------------
// Helper: a tiny fixed-capacity string for building marker lines without alloc
// ---------------------------------------------------------------------------

/// A stack-allocated string buffer capped at `N` bytes.
struct IStr<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> IStr<N> {
    fn new() -> Self {
        Self { buf: [0u8; N], len: 0 }
    }

    /// Append `s`, silently truncating if the buffer would overflow.
    fn push_str(&mut self, s: &str) -> bool {
        let bytes = s.as_bytes();
        let available = N - self.len;
        if bytes.len() > available {
            return false;
        }
        self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        true
    }

    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

// ---------------------------------------------------------------------------
// Internal helper: write a line + newline
// ---------------------------------------------------------------------------

fn write_line<W: Write + ?Sized>(out: &mut W, line: &str) -> Result<(), W::Error> {
    out.write_all(line.as_bytes())?;
    out.write_all(b"\n")
}

fn write_line_bytes<W: Write + ?Sized>(out: &mut W, line: &[u8]) -> Result<(), W::Error> {
    out.write_all(line)?;
    out.write_all(b"\n")
}

// ---------------------------------------------------------------------------
// Resolver trait
// ---------------------------------------------------------------------------

/// A trait for resolving inclusions.
pub trait Resolver {
    type Error: embedded_io::Error;
    fn resolve(
        &mut self,
        path: &str,
        out: &mut dyn Write<Error = Self::Error>,
    ) -> Result<(), Self::Error>;
}

// ---------------------------------------------------------------------------
// FileResolver  (std only)
// ---------------------------------------------------------------------------

/// Default resolver that reads from the filesystem.
///
/// Only available with the `std` feature.
#[cfg(feature = "std")]
pub struct FileResolver;

#[cfg(feature = "std")]
impl Resolver for FileResolver {
    type Error = std::io::Error;

    fn resolve(
        &mut self,
        path: &str,
        out: &mut dyn Write<Error = std::io::Error>,
    ) -> std::io::Result<()> {
        if let Ok(content) = std::fs::read(path) {
            out.write_all(&content)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ZipResolver  (zip + std only)
// ---------------------------------------------------------------------------

/// Resolver that reads from a zip archive.
///
/// Only available with the `zip` feature (which implies `std`).
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
    type Error = std::io::Error;

    fn resolve(
        &mut self,
        path: &str,
        out: &mut dyn Write<Error = std::io::Error>,
    ) -> std::io::Result<()> {
        use std::io::Read as _;
        if let Ok(mut file) = self.archive.by_name(path) {
            let mut buf = [0u8; 4096];
            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                out.write_all(&buf[..n])?;
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// splice  (std convenience wrapper)
// ---------------------------------------------------------------------------

/// Splice a string using the default [`FileResolver`].
///
/// Only available with the `std` feature. For `no_std` use [`splice_with`]
/// and supply your own [`Resolver`].
#[cfg(feature = "std")]
pub fn splice<W: Write<Error = std::io::Error>>(s: &str, file: &mut W) -> std::io::Result<()> {
    splice_with(s, file, FileResolver)
}

// ---------------------------------------------------------------------------
// splice_with  (no_std compatible core)
// ---------------------------------------------------------------------------

/// Splice a string, resolving inclusions with a custom [`Resolver`].
///
/// Works in `no_std` environments — only requires [`embedded_io::Write`] for
/// the output and a [`Resolver`] whose error type matches.
pub fn splice_with<W, R>(s: &str, file: &mut W, mut resolver: R) -> Result<(), W::Error>
where
    W: Write,
    R: Resolver<Error = W::Error>,
{
    let mut go = true;
    let mut lx: Option<&str> = None;

    for l in s.lines() {
        if go {
            write_line(file, l)?;
        }

        // `@path` shorthand: immediately emit begin/content/end markers
        if let Some(path) = l.strip_prefix("@") {
            let path = path.trim();

            // Build "[[begin <path>]]" without heap allocation
            let mut marker: IStr<512> = IStr::new();
            marker.push_str("[[begin ");
            marker.push_str(path);
            marker.push_str("]]");
            write_line_bytes(file, marker.as_bytes())?;

            resolver.resolve(path, file)?;
            write_line(file, "[[end]]")?;
            lx = Some(path);
            continue;
        }

        // `[[begin path]]` / `[[end]]` markers
        if let Some((_, b2)) = l.split_once("[[")
            && let Some((b, _)) = b2.rsplit_once("]]")
        {
            let b = b.trim();
            match b {
                "end" => {
                    if !go {
                        write_line(file, l)?;
                    }
                    go = true;
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
