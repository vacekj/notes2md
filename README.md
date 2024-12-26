# Apple Notes Exporter

A Rust command-line tool to export Apple Notes to Markdown files with proper formatting and metadata preservation.


Based on: https://github.com/storizzi/notes-exporter

## Features

- Exports Apple Notes to Markdown files
- Preserves metadata in YAML frontmatter
- Handles images and attachments
- Supports folder organization
- Preserves note creation and modification dates
- Handles Czech characters and special formatting

## Usage

```bash
# Export notes to current directory
notes-exporter-rs

# Export to specific directory
notes-exporter-rs -o /path/to/output

# Export without using subdirectories
notes-exporter-rs --use-subdirs false
```

## CLI Options

```
Options:
  -o, --output <PATH>            Output directory for markdown files [default: .]
  -u, --use-attachments <BOOL>   Whether to use attachments folder for images [default: true]
      --filename-format <STR>    Format for filenames [default: &title]
      --subdir-format <STR>      Format for subdirectories [default: &folder]
      --use-subdirs <BOOL>       Whether to use subdirectories [default: true]
  -h, --help                     Print help
  -V, --version                  Print version
```

### Format Variables

The following variables can be used in filename and subdirectory formats:
- `&title`: Note title
- `&folder`: Folder name
- `&account`: Account name
- `&id`: Note ID

## Output Format

Each note is exported as a Markdown file with YAML frontmatter:

```markdown
---
title: "Note Title"
folder: "Folder Name"
account: "Account Name"
id: "note-id"
created: "Creation Date"
modified: "Modification Date"
---

Note content in Markdown format...
```

## Requirements

- macOS (uses AppleScript to access Notes)
- Rust toolchain for building

## License

MIT License