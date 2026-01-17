---
name: rice-markers
description: Generates files with Rice inclusion markers for automatic content splicing. Use when creating files that should include content from other files, or when setting up AGENTS.md with file inclusions.
---

# Rice Markers

Rice processes files to splice in content from external sources using special markers.

## Marker Syntax

### Standard Markers

Use comment-wrapped markers to include file content:

```
// [[begin path/to/file.txt]]
... content from file.txt appears here ...
// [[end]]
```

The comment style should match the file type:
- `//` for Rust, JavaScript, TypeScript, C, Go
- `#` for Python, Shell, YAML
- `<!-- -->` for HTML, Markdown
- `/* */` for CSS

### Shorthand for AGENTS.md

In AGENTS.md files, use the `@` prefix shorthand:

```
@path/to/file.txt
```

This automatically expands to:

```
@path/to/file.txt
[[begin path/to/file.txt]]
... file content ...
[[end]]
```

## Usage Patterns

### Including Reference Files

```markdown
# Project Documentation

See the main library:
@src/lib.rs
```

### Including Configuration

```yaml
# config.yaml
# [[begin defaults.yaml]]
# ... defaults spliced here ...
# [[end]]
```

### Nesting in Code Comments

```rust
// [[begin LICENSE-HEADER.txt]]
// ... license text ...
// [[end]]

fn main() { }
```

## Behavior

1. Rice reads the source file
2. When it encounters `[[begin <path>]]`, it replaces everything until `[[end]]` with the current content of `<path>`
3. Lines outside markers pass through unchanged
4. The `@path` shorthand in AGENTS.md creates both the marker and the inclusion

## When to Use

- AGENTS.md files that should include excerpts from source files
- Templates that embed other file contents
- Documentation that needs to stay synced with source code
- Configuration files with shared sections
