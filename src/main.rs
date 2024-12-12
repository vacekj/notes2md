use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use clap::Parser;
use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use html2md::parse_html;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Deserialize, Serialize, Debug)]
struct Note {
    title: String,
    content: String,
    folder: String,
    account: String,
    id: String,
    created: String,
    modified: String,
}

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

    // Create output directory if it doesn't exist
    fs::create_dir_all(&cli.output).context("Failed to create output directory")?;

    println!("Notes Exporter Rust");
    println!("Output directory: {:?}", cli.output);

    // Get notes data from AppleScript
    let notes = get_notes()?;

    // Process each note
    for note in notes {
        let markdown = process_note(&note, &cli)?;
        save_note(&note, &markdown, &cli)?;
    }

    println!("Export complete!");
    Ok(())
}

fn get_notes() -> Result<Vec<Note>> {
    let script_path = Path::new("export-notes.applescript");
    if !script_path.exists() {
        return Err(anyhow!(
            "export-notes.applescript not found in current directory"
        ));
    }

    let output = Command::new("osascript")
        .arg(script_path)
        .output()
        .context("Failed to execute AppleScript")?;

    if !output.status.success() {
        return Err(anyhow!(
            "AppleScript execution failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json_str =
        String::from_utf8(output.stdout).context("Failed to parse AppleScript output as UTF-8")?;

    let notes: Vec<Note> =
        serde_json::from_str(&json_str).context("Failed to parse JSON output from AppleScript")?;

    println!("Successfully parsed {} notes", notes.len());

    Ok(notes)
}

fn process_note(note: &Note, cli: &Cli) -> Result<String> {
    println!(
        "Processing note: '{}' from folder '{}'",
        note.title, note.folder
    );

    // Extract images and get updated HTML
    let html_with_local_images = extract_and_save_images(
        &note.content,
        &get_note_path(note, cli)?,
        cli.use_attachments,
    )?;

    // Save the HTML for investigation
    save_html(&note, &html_with_local_images, cli)?;

    // Convert to markdown and return
    let markdown = parse_html(&html_with_local_images);

    // For this specific note, let's try to fix the h1 splitting
    if note.content.contains("<h1>") {
        // Join all the h1 content together
        let doc = Html::parse_document(&html_with_local_images);
        let h1_selector = Selector::parse("h1").unwrap();
        let h1_texts: Vec<String> = doc
            .select(&h1_selector)
            .map(|el| el.text().collect::<String>())
            .collect();

        // If we found split h1s, replace the markdown
        if !h1_texts.is_empty() {
            let joined_text = h1_texts.join("");
            if !joined_text.trim().is_empty() {
                return Ok(format!(
                    "# {}\n\n{}",
                    joined_text.trim(),
                    markdown
                        .lines()
                        .filter(|line| !line.starts_with('#'))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
            }
        }
    }

    Ok(markdown)
}

fn get_note_path(note: &Note, cli: &Cli) -> Result<PathBuf> {
    let mut path = cli.output.clone();

    if cli.use_subdirs {
        path = path.join(&note.folder);
    }

    Ok(path)
}

fn save_note(note: &Note, markdown: &str, cli: &Cli) -> Result<()> {
    let mut output_path = get_note_path(note, cli)?;
    fs::create_dir_all(&output_path)
        .with_context(|| format!("Failed to create directory: {:?}", output_path))?;

    // Create filename from title (sanitize it)
    let safe_title = note
        .title
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-");
    output_path = output_path.join(format!("{}.md", safe_title));

    // Create frontmatter
    let mut content = String::new();
    content.push_str("---\n");
    content.push_str(&format!("title: \"{}\"\n", note.title));
    content.push_str(&format!("folder: \"{}\"\n", note.folder));
    content.push_str(&format!("account: \"{}\"\n", note.account));
    content.push_str(&format!("id: \"{}\"\n", note.id));
    content.push_str(&format!("created: \"{}\"\n", note.created));
    content.push_str(&format!("modified: \"{}\"\n", note.modified));
    content.push_str("---\n\n");

    // Add the markdown content
    content.push_str(markdown);

    // Write the complete content
    fs::write(&output_path, content.as_bytes())
        .with_context(|| format!("Failed to write file: {:?}", output_path))?;

    println!(
        "Created MD: {}",
        output_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    Ok(())
}

fn save_html(note: &Note, html: &str, cli: &Cli) -> Result<()> {
    let mut output_path = get_note_path(note, cli)?;
    fs::create_dir_all(&output_path)
        .with_context(|| format!("Failed to create directory: {:?}", output_path))?;

    // Create filename from title (sanitize it)
    let safe_title = note
        .title
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-");
    output_path = output_path.join(format!("{}.html", safe_title));

    // Write the HTML content
    fs::write(&output_path, html.as_bytes())
        .with_context(|| format!("Failed to write HTML file: {:?}", output_path))?;

    println!(
        "Created HTML: {}",
        output_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    Ok(())
}

fn extract_and_save_images(
    html_content: &str,
    output_dir: &PathBuf,
    use_attachments: bool,
) -> Result<String> {
    let document = Html::parse_document(html_content);
    let img_selector = Selector::parse("img").unwrap();
    let mut modified_html = html_content.to_string();
    let mut img_counter = 0;

    // Determine attachments directory
    let attachments_dir = if use_attachments {
        output_dir.join("attachments")
    } else {
        output_dir.to_owned()
    };

    // Create attachments directory if it doesn't exist and we're using it
    if use_attachments {
        fs::create_dir_all(&attachments_dir).with_context(|| {
            format!(
                "Failed to create attachments directory: {:?}",
                attachments_dir
            )
        })?;
    }

    // Find all img tags
    for img in document.select(&img_selector) {
        if let Some(src) = img.value().attr("src") {
            if src.starts_with("data:image") {
                img_counter += 1;

                // Extract image format and data
                let parts: Vec<&str> = src.split(',').collect();
                if parts.len() != 2 {
                    continue; // Skip malformed data URLs
                }

                // Get format from header (e.g., "data:image/jpeg;base64" -> "jpeg")
                let format = parts[0]
                    .split('/')
                    .nth(1)
                    .and_then(|s| s.split(';').next())
                    .unwrap_or("png");

                // Decode base64 data
                let image_data = base64
                    .decode(parts[1])
                    .with_context(|| "Failed to decode base64 image data")?;

                // Generate filename
                let filename = format!("attachment-{:03}.{}", img_counter, format);
                let image_path = attachments_dir.join(&filename);

                // Save the image
                fs::write(&image_path, image_data)
                    .with_context(|| format!("Failed to write image file: {:?}", image_path))?;

                println!("Saved image: {:?}", image_path);

                // Update HTML to reference the local file
                let new_src = if use_attachments {
                    format!("attachments/{}", filename)
                } else {
                    filename
                };

                modified_html = modified_html.replace(src, &new_src);
            }
        }
    }

    Ok(modified_html)
}
