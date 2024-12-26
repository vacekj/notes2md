use anyhow::Result;
use apple_notes_exporter::{export_notes, ExportConfig};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Output directory for markdown files
    #[arg(short, long, default_value = ".")]
    output: PathBuf,

    /// Whether to use attachments folder for images
    #[arg(short, long, default_value = "true")]
    use_attachments: bool,

    /// Format for filenames
    #[arg(long, default_value = "&title")]
    filename_format: String,

    /// Format for subdirectories
    #[arg(long, default_value = "&folder")]
    subdir_format: String,

    /// Whether to use subdirectories
    #[arg(long, default_value = "true")]
    use_subdirs: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Apple Notes Exporter");
    println!("Output directory: {:?}", cli.output);

    let config = ExportConfig {
        output_dir: cli.output,
        use_attachments: cli.use_attachments,
        filename_format: cli.filename_format,
        subdir_format: cli.subdir_format,
        use_subdirs: cli.use_subdirs,
    };

    let notes = export_notes(&config)?;
    println!("Successfully exported {} notes", notes.len());

    Ok(())
}
