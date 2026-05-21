use chrono::{DateTime, Local};

pub fn format_time(input_time: DateTime<Local>) -> String {
    input_time.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_size(input_size: u64) -> String {
    const KB: f64 = 1000.0;
    const MB: f64 = KB * 1000.0;
    const GB: f64 = MB * 1000.0;

    let size = input_size as f64;
    if size < KB {
        format!("{input_size} B")
    } else if size < MB {
        format!("{:.1} KB", size / KB)
    } else if size < GB {
        format!("{:.1} MB", size / MB)
    } else {
        format!("{:.1} GB", size / GB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(512), "512 B");
    }

    #[test]
    fn format_size_kilobytes() {
        assert_eq!(format_size(1500), "1.5 KB");
    }
}
