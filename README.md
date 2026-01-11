# rice

`rice` is a tool for "splicing" external files into a target file using special comment directives. It's designed to make it easy to manage components or dependencies within files, especially useful for AI agent context or project configuration.

## Features

- **Directives**: Use `@path/to/file` to include a file's content.
- **In-place updates**: Re-running `rice` updates the content between `[[begin path]]` and `[[end]]` markers.
- **Extensible API**: Use the Rust library to implement custom resolution logic (e.g., fetching from URLs or databases).

## Installation

```bash
cargo install --path .
```

## CLI Usage

Process a file in-place:

```bash
rice path/to/file.ext
```

### How it works

If you have a file `main.py`:

```python
# @utils.py
```

And `utils.py`:

```python
def add(a, b):
    return a + b
```

Running `rice main.py` will transform it into:

```python
# @utils.py
[[begin utils.py]]
def add(a, b):
    return a + b
[[end]]
```

If you modify `utils.py` and run `rice main.py` again, the content between the `[[begin]]` and `[[end]]` markers will be updated.

## Library Usage

Add `rice` to your `Cargo.toml`:

```toml
[dependencies]
rice = { git = "https://github.com/portal-co/rice.git" }
```

### Basic Splicing

```rust
use std::io::Cursor;

let input = "@hello.txt";
let mut output = Vec::new();

// Uses the default FileResolver
rice::splice(input, &mut output).unwrap();
```

### Custom Resolver

You can implement the `Resolver` trait to handle custom paths (e.g., virtual file systems or remote URLs).

```rust
use rice::{Resolver, splice_with};
use std::io::Write;

struct MyResolver;

impl Resolver for MyResolver {
    fn resolve(&mut self, path: &str, out: &mut (dyn Write + '_)) -> std::io::Result<()> {
        if path == "virtual:hello" {
            write!(out, "Hello from the resolver!")?;
        }
        Ok(())
    }
}

let mut output = Vec::new();
splice_with("@virtual:hello", &mut output, MyResolver).unwrap();
```

---
*AI assisted*
