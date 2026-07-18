use rtvui_core::Artifact;
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

const MAX_PREVIEW_BYTES: usize = 64 * 1024;

pub enum Previewer {
    Empty,
    Folder(FolderPreviewer),
    Preview(PreviewPreviewer),
}

pub struct FolderPreviewer {
    pub title: String,
    pub artifacts: Vec<Artifact>,
}

pub struct PreviewPreviewer {
    pub title: String,
    pub body: String,
}

pub fn read_preview_body(path: &Path) -> Result<String> {
    if is_image_path(path) {
        return Ok(image_preview_summary(path));
    }
    read_text_prefix(path)
}

pub fn read_text_prefix(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; MAX_PREVIEW_BYTES];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "ico"
            )
        })
        .unwrap_or(false)
}

fn image_preview_summary(path: &Path) -> String {
    let meta = std::fs::metadata(path).ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let dims = probe_image_dimensions(path);
    let mut lines = vec![
        "Image preview".to_string(),
        format!("File: {}", path.display()),
        format!("Size: {size} bytes"),
    ];
    if let Some((w, h)) = dims {
        lines.push(format!("Dimensions: {w}×{h}"));
    } else {
        lines.push("Dimensions: (unavailable)".to_string());
    }
    lines.push(String::new());
    lines.push("Tip: press Enter to open with the system app, or e for $EDITOR.".to_string());
    lines.join("\n")
}

/// Lightweight magic-byte probes for common formats (no image crate dependency).
fn probe_image_dimensions(path: &Path) -> Option<(u32, u32)> {
    let mut file = File::open(path).ok()?;
    let mut header = [0u8; 24];
    let n = file.read(&mut header).ok()?;
    if n < 10 {
        return None;
    }
    // PNG: 8-byte sig + IHDR width/height at offset 16
    if header.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']) && n >= 24 {
        let w = u32::from_be_bytes(header[16..20].try_into().ok()?);
        let h = u32::from_be_bytes(header[20..24].try_into().ok()?);
        return Some((w, h));
    }
    // GIF
    if (header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a")) && n >= 10 {
        let w = u16::from_le_bytes(header[6..8].try_into().ok()?) as u32;
        let h = u16::from_le_bytes(header[8..10].try_into().ok()?) as u32;
        return Some((w, h));
    }
    // JPEG: scan for SOF0/SOF2
    if header.starts_with(&[0xff, 0xd8]) {
        return jpeg_dimensions(path);
    }
    None
}

fn jpeg_dimensions(path: &Path) -> Option<(u32, u32)> {
    let data = std::fs::read(path).ok()?;
    let mut i = 2usize;
    while i + 9 < data.len() {
        if data[i] != 0xff {
            i += 1;
            continue;
        }
        let marker = data[i + 1];
        i += 2;
        if marker == 0xd8 || marker == 0xd9 || marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        if i + 2 > data.len() {
            break;
        }
        let len = u16::from_be_bytes([data[i], data[i + 1]]) as usize;
        if len < 2 || i + len > data.len() {
            break;
        }
        // SOF0 / SOF2
        if matches!(marker, 0xc0 | 0xc2) && len >= 7 {
            let h = u16::from_be_bytes([data[i + 3], data[i + 4]]) as u32;
            let w = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
            return Some((w, h));
        }
        i += len;
    }
    None
}
