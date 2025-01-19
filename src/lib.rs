//! Apple Notes Exporter
//!
//! A library for exporting Apple Notes to Markdown files with support for images,
//! attachments, and metadata preservation.
//!
//! # Example
//! ```no_run
//! use apple_notes_exporter::{export_notes, ExportConfig};
//! use std::path::PathBuf;
//!
//! fn main() -> anyhow::Result<()> {
//!     let config = ExportConfig::default();
//!     let notes = export_notes(&config)?;
//!     println!("Exported {} notes", notes.len());
//!     Ok(())
//! }
//! ```

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Represents a single Apple Note with its metadata and content.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Note {
    /// The title of the note
    pub title: String,
    /// The HTML content of the note
    pub content: String,
    /// The folder containing the note
    pub folder: String,
    /// The account the note belongs to (e.g., "iCloud")
    pub account: String,
    /// Unique identifier for the note
    pub id: String,
    /// Creation date as a string
    pub created: String,
    /// Last modification date as a string
    pub modified: String,
}

/// Configuration options for the export process.
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Directory where notes will be exported
    pub output_dir: PathBuf,
    /// Whether to store images in a separate attachments folder
    pub use_attachments: bool,
    /// Format string for filenames (supports &title, &folder, &account, &id)
    pub filename_format: String,
    /// Format string for subdirectories (supports &title, &folder, &account, &id)
    pub subdir_format: String,
    /// Whether to organize notes in subdirectories
    pub use_subdirs: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            use_attachments: true,
            filename_format: String::from("&title"),
            subdir_format: String::from("&folder"),
            use_subdirs: true,
        }
    }
}

/// Exports all notes from Apple Notes to Markdown files.
///
/// This function:
/// 1. Creates the output directory if it doesn't exist
/// 2. Retrieves all notes using AppleScript
/// 3. Processes each note (converts HTML to Markdown, handles images)
/// 4. Saves notes with their metadata as Markdown files
///
/// # Arguments
/// * `config` - Configuration options for the export process
///
/// # Returns
/// * `Result<Vec<Note>>` - A vector of all exported notes on success
///
/// # Errors
/// * If the output directory cannot be created
/// * If the AppleScript execution fails
/// * If any note processing or saving fails
pub fn export_notes(config: &ExportConfig) -> Result<Vec<Note>> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(&config.output_dir).context("Failed to create output directory")?;

    // Get notes data from AppleScript
    let notes = get_notes()?;

    // Process each note
    for note in &notes {
        let markdown = process_note(note, config)?;
        save_note(note, &markdown, config)?;
    }

    Ok(notes)
}

/// Retrieves all notes from Apple Notes using AppleScript.
///
/// # Returns
/// * `Result<Vec<Note>>` - A vector of all notes on success
///
/// # Errors
/// * If the AppleScript file is not found
/// * If the AppleScript execution fails
/// * If the output cannot be parsed as JSON
pub fn get_notes() -> Result<Vec<Note>> {
    let script_path = PathBuf::from("export-notes.applescript");
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

    Ok(notes)
}

/// Processes a single note, converting it to Markdown and handling attachments.
///
/// # Arguments
/// * `note` - The note to process
/// * `config` - Export configuration options
///
/// # Returns
/// * `Result<String>` - The processed Markdown content
///
/// # Errors
/// * If image extraction fails
/// * If HTML processing fails
pub fn process_note(note: &Note, config: &ExportConfig) -> Result<String> {
    // Extract images and get updated HTML
    let html_with_local_images = extract_and_save_images(
        &note.content,
        &get_note_path(note, config)?,
        config.use_attachments,
    )?;

    // Save the HTML for investigation
    save_html(note, &html_with_local_images, config)?;

    // Convert to markdown
    let markdown = html2md::parse_html(&html_with_local_images);

    // Handle split h1s if present
    if note.content.contains("<h1>") {
        let doc = Html::parse_document(&html_with_local_images);
        let h1_selector = Selector::parse("h1").unwrap();
        let h1_texts: Vec<String> = doc
            .select(&h1_selector)
            .map(|el| el.text().collect::<String>())
            .collect();

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

fn get_note_path(note: &Note, config: &ExportConfig) -> Result<PathBuf> {
    let mut path = config.output_dir.clone();

    if config.use_subdirs {
        path = path.join(&note.folder);
    }

    Ok(path)
}

fn save_note(note: &Note, markdown: &str, config: &ExportConfig) -> Result<()> {
    let mut output_path = get_note_path(note, config)?;
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

    Ok(())
}

fn save_html(note: &Note, html: &str, config: &ExportConfig) -> Result<()> {
    let mut output_path = get_note_path(note, config)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_export_config_default() {
        let config = ExportConfig::default();
        assert_eq!(config.output_dir, PathBuf::from("."));
        assert!(config.use_attachments);
        assert_eq!(config.filename_format, "&title");
        assert_eq!(config.subdir_format, "&folder");
        assert!(config.use_subdirs);
    }

    #[test]
    fn test_process_note_with_images() -> Result<()> {
        let temp_dir = tempdir()?;
        let config = ExportConfig {
            output_dir: temp_dir.path().to_path_buf(),
            use_attachments: true,
            filename_format: String::from("&title"),
            subdir_format: String::from("&folder"),
            use_subdirs: true,
        };

        let note = Note {
            title: String::from("Test Note"),
            content: String::from(
                r#"<p>Test content</p><img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII="/>"#,
            ),
            folder: String::from("Test Folder"),
            account: String::from("Test Account"),
            id: String::from("test-id"),
            created: String::from("2024-01-01"),
            modified: String::from("2024-01-01"),
        };

        let markdown = process_note(&note, &config)?;
        assert!(markdown.contains("![](attachments/attachment-001.png)"));

        // Check if image was saved
        let image_path = temp_dir
            .path()
            .join("Test Folder")
            .join("attachments")
            .join("attachment-001.png");
        assert!(image_path.exists());

        Ok(())
    }

    #[test]
    fn test_process_note_with_h1() -> Result<()> {
        let temp_dir = tempdir()?;
        let config = ExportConfig {
            output_dir: temp_dir.path().to_path_buf(),
            use_attachments: true,
            filename_format: String::from("&title"),
            subdir_format: String::from("&folder"),
            use_subdirs: true,
        };

        let note = Note {
            title: String::from("Test Note"),
            content: String::from(
                "<h1>Title 1</h1><p>Content 1</p><h1>Title 2</h1><p>Content 2</p>",
            ),
            folder: String::from("Test Folder"),
            account: String::from("Test Account"),
            id: String::from("test-id"),
            created: String::from("2024-01-01"),
            modified: String::from("2024-01-01"),
        };

        let markdown = process_note(&note, &config)?;
        assert!(markdown.starts_with("# Title 1Title 2\n\n"));
        assert!(markdown.contains("Content 1"));
        assert!(markdown.contains("Content 2"));

        Ok(())
    }

    #[test]
    fn test_get_note_path() -> Result<()> {
        let temp_dir = tempdir()?;
        let config = ExportConfig {
            output_dir: temp_dir.path().to_path_buf(),
            use_attachments: true,
            filename_format: String::from("&title"),
            subdir_format: String::from("&folder"),
            use_subdirs: true,
        };

        let note = Note {
            title: String::from("Test Note"),
            content: String::from("Test content"),
            folder: String::from("Test Folder"),
            account: String::from("Test Account"),
            id: String::from("test-id"),
            created: String::from("2024-01-01"),
            modified: String::from("2024-01-01"),
        };

        let path = get_note_path(&note, &config)?;
        assert_eq!(path, temp_dir.path().join("Test Folder"));

        let config_no_subdirs = ExportConfig {
            use_subdirs: false,
            ..config
        };
        let path_no_subdirs = get_note_path(&note, &config_no_subdirs)?;
        assert_eq!(path_no_subdirs, temp_dir.path());

        Ok(())
    }
}
