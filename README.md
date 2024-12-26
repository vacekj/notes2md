# Apple Notes Exporter

A Rust library and CLI tool for exporting Apple Notes to Markdown files.

## Features

- Export all notes from Apple Notes to Markdown files
- Preserve folder structure
- Handle embedded images and attachments
- Support for frontmatter metadata
- Clean and modern Markdown output

## Installation

### As a CLI tool

```bash
cargo install apple-notes-exporter
```

### As a library

Add this to your `Cargo.toml`:

```toml
[dependencies]
apple-notes-exporter = "0.1.0"
```

## Usage

### CLI

```bash
# Export notes to current directory
apple-notes-exporter

# Export notes to specific directory
apple-notes-exporter -o ./my-notes

# Export without using attachments folder
apple-notes-exporter --use-attachments false

# Export without subdirectories
apple-notes-exporter --use-subdirs false
```

### Library

```rust
use apple_notes_exporter::{export_notes, ExportConfig};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let config = ExportConfig {
        output_dir: PathBuf::from("./my-notes"),
        use_attachments: true,
        filename_format: String::from("&title"),
        subdir_format: String::from("&folder"),
        use_subdirs: true,
    };

    let notes = export_notes(&config)?;
    println!("Exported {} notes", notes.len());

    Ok(())
}
```

## Requirements

- macOS (uses AppleScript to access Notes)
- Rust 1.70 or later

## License

MIT