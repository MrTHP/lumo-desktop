use base64::Engine;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

const MAX_BYTES: usize = 30 * 1024 * 1024;
const MAX_TEXT_CHARS: usize = 120_000;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInfo {
    pub name: String,
    pub mime: String,
    pub size: usize,
    pub kind: String,
    pub text: String,
    pub note: String,
    pub truncated: bool,
}

#[tauri::command]
pub async fn process_attachment(
    name: String,
    mime: String,
    data_base64: String,
) -> Result<AttachmentInfo, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_base64.as_bytes())
        .map_err(|error| format!("Invalid attachment data: {error}"))?;

    if bytes.len() > MAX_BYTES {
        return Err(format!(
            "Attachment is too large ({} MiB, limit {} MiB)",
            bytes.len() / 1024 / 1024,
            MAX_BYTES / 1024 / 1024
        ));
    }

    let tempdir = tempfile::Builder::new()
        .prefix("lumo-attach-")
        .tempdir()
        .map_err(|error| format!("Cannot create temporary directory: {error}"))?;
    let file_name = sanitize_filename(&name);
    let path = tempdir.path().join(&file_name);
    fs::write(&path, &bytes)
        .map_err(|error| format!("Cannot write temporary attachment: {error}"))?;

    let extension = Path::new(&name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mime = mime.to_ascii_lowercase();

    let mut kind = "binary".to_string();
    let note: String;
    let mut text = String::new();

    if mime.starts_with("image/") || is_image_extension(&extension) {
        kind = "image".to_string();
        text = ocr_image(&path).await.unwrap_or_default();
        note = if text.trim().is_empty() {
            "Image attached. OCR found no text (or tesseract is unavailable).".to_string()
        } else {
            "Image text extracted locally with OCR.".to_string()
        };
    } else if mime == "application/pdf" || extension == "pdf" {
        kind = "pdf".to_string();
        text = extract_pdf(&path, tempdir.path()).await;
        note = if text.trim().is_empty() {
            "PDF attached, but no text layer or OCR result was found.".to_string()
        } else {
            "PDF text extracted locally.".to_string()
        };
    } else if extension == "docx" || extension == "odt" {
        kind = "document".to_string();
        let entry = if extension == "docx" {
            "word/document.xml"
        } else {
            "content.xml"
        };
        if let Some(xml) = unzip_entry(&path, entry).await {
            text = strip_markup(&xml);
            note = "Document text extracted locally.".to_string();
        } else {
            note = "Document attached, but its text could not be extracted.".to_string();
        }
    } else if extension == "html" || extension == "htm" || mime == "text/html" {
        kind = "html".to_string();
        text = strip_markup(&String::from_utf8_lossy(&bytes));
        note = "HTML text extracted locally.".to_string();
    } else if is_text_like(&extension, &mime) {
        kind = "text".to_string();
        text = String::from_utf8_lossy(&bytes).to_string();
        note = "Text file read locally.".to_string();
    } else {
        note = "Binary file attached. Only its name and metadata can be used.".to_string();
    }

    let (text, truncated) = limit_text(text, MAX_TEXT_CHARS);

    Ok(AttachmentInfo {
        name,
        mime,
        size: bytes.len(),
        kind,
        text,
        note,
        truncated,
    })
}

fn is_image_extension(extension: &str) -> bool {
    matches!(
        extension,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "avif" | "svg"
    )
}

fn is_text_like(extension: &str, mime: &str) -> bool {
    const EXTENSIONS: &[&str] = &[
        "txt", "md", "markdown", "rst", "log", "csv", "tsv", "json", "jsonl", "yaml", "yml",
        "toml", "ini", "conf", "cfg", "xml", "css", "scss", "less", "js", "mjs", "cjs", "ts",
        "tsx", "jsx", "py", "rs", "go", "java", "c", "h", "cc", "cpp", "hpp", "cs", "rb", "php",
        "sh", "bash", "zsh", "fish", "sql", "tex", "r", "lua", "swift", "kt", "kts", "gradle",
        "dockerfile", "makefile", "env", "gitignore", "patch", "diff",
    ];
    EXTENSIONS.contains(&extension)
        || mime.starts_with("text/")
        || mime.contains("json")
        || mime.contains("xml")
        || mime.contains("yaml")
        || mime.contains("javascript")
        || mime.contains("x-sh")
        || mime.contains("x-python")
        || mime.contains("csv")
        || mime.contains("toml")
}

async fn ocr_image(path: &Path) -> Option<String> {
    for languages in ["fra+eng", "eng"] {
        let output = Command::new("tesseract")
            .arg(path)
            .arg("stdout")
            .arg("-l")
            .arg(languages)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
            .ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
    }

    let output = Command::new("tesseract")
        .arg(path)
        .arg("stdout")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }
    None
}

async fn extract_pdf(path: &Path, tempdir: &Path) -> String {
    if let Ok(output) = Command::new("pdftotext")
        .arg("-layout")
        .arg(path)
        .arg("-")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            if text.chars().filter(|character| !character.is_whitespace()).count() > 20 {
                return text;
            }
        }
    }

    // Scanned PDF fallback: render the first pages and OCR them.
    let prefix = tempdir.join("page");
    let output = Command::new("pdftoppm")
        .args(["-png", "-r", "150", "-f", "1", "-l", "3"])
        .arg(path)
        .arg(&prefix)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .await;
    if let Ok(output) = output {
        if output.status.success() {
            let mut text = String::new();
            for page in 1..=3 {
                let page_path = tempdir.join(format!("page-{page}.png"));
                if page_path.is_file() {
                    if let Some(page_text) = ocr_image(&page_path).await {
                        text.push_str(&page_text);
                        text.push_str("\n\n");
                    }
                }
            }
            if !text.trim().is_empty() {
                return format!("{text}\n[OCR applied to the first 3 pages]");
            }
        }
    }

    String::new()
}

async fn unzip_entry(path: &Path, entry: &str) -> Option<String> {
    let output = Command::new("unzip")
        .arg("-p")
        .arg(path)
        .arg(entry)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

fn strip_markup(input: &str) -> String {
    let mut text = input
        .replace("</w:p>", "\n")
        .replace("</w:tr>", "\n")
        .replace("</text:p>", "\n")
        .replace("</text:h>", "\n")
        .replace("</a:p>", "\n")
        .replace("<w:br/>", "\n")
        .replace("<w:tab/>", "\t")
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n")
        .replace("</div>", "\n")
        .replace("</li>", "\n")
        .replace("</h1>", "\n")
        .replace("</h2>", "\n")
        .replace("</h3>", "\n")
        .replace("</tr>", "\n")
        .replace("</td>", "\t");

    let mut output = String::with_capacity(text.len());
    let mut in_tag = false;
    for character in text.drain(..) {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }

    output
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}

fn limit_text(input: String, max: usize) -> (String, bool) {
    let count = input.chars().count();
    if count <= max {
        return (input, false);
    }
    let mut limited: String = input.chars().take(max).collect();
    limited.push_str("\n\n[... attachment text truncated ...]");
    (limited, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(bytes: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    #[tokio::test]
    async fn extracts_plain_text() {
        let info = process_attachment(
            "note.txt".into(),
            "text/plain".into(),
            encode(b"hello attachment"),
        )
        .await
        .unwrap();
        assert_eq!(info.kind, "text");
        assert!(info.text.contains("hello attachment"));
    }

    #[tokio::test]
    async fn extracts_pdf_if_available() {
        let path = "/usr/share/doc/fontconfig/fontconfig-user.pdf";
        if !Path::new(path).is_file() {
            return;
        }
        let bytes = fs::read(path).unwrap();
        let info = process_attachment(
            "fontconfig-user.pdf".into(),
            "application/pdf".into(),
            encode(&bytes),
        )
        .await
        .unwrap();
        assert_eq!(info.kind, "pdf");
        assert!(!info.text.trim().is_empty());
    }

    #[tokio::test]
    async fn ocr_image_if_available() {
        let path = "/tmp/lumo-ocr-test.png";
        if !Path::new(path).is_file() {
            return;
        }
        let bytes = fs::read(path).unwrap();
        let info = process_attachment(
            "ocr.png".into(),
            "image/png".into(),
            encode(&bytes),
        )
        .await
        .unwrap();
        assert_eq!(info.kind, "image");
        assert!(!info.text.trim().is_empty());
    }
}

fn sanitize_filename(name: &str) -> String {
    let base = Path::new(name)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("attachment");
    let cleaned: String = base
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.trim_matches('_').is_empty() {
        "attachment".to_string()
    } else {
        cleaned
    }
}

