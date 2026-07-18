pub mod errors;
pub mod utils;
#[cfg(test)]
mod tests {
    mod core;
}

use std::path::PathBuf;
use std::time::SystemTime;

const KILOBYTE: u64 = 1024;
const MEGABYTE: u64 = KILOBYTE * 1024;
const GIGABYTE: u64 = MEGABYTE * 1024;

#[cfg(not(test))]
const MAX_ENTRIES: usize = 50_000;

#[cfg(test)]
const MAX_ENTRIES: usize = 100;

#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactType {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ArtifactSort {
    #[default]
    Name,
    Size,
    Modified,
}

#[derive(Default)]
pub struct ArtifactOptions {
    pub show_hidden: bool,
    pub sort: ArtifactSort,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArtifactListResult {
    pub artifacts: Vec<Artifact>,
    pub partial: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Artifact {
    pub name: String,
    pub path: PathBuf,
    pub artifact_type: ArtifactType,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

impl Artifact {
    pub fn format_size(&self) -> String {
        match self.size {
            0..=KILOBYTE => format!("{} B", self.size),
            n if n < MEGABYTE => format!("{:.1} KB", n as f64 / KILOBYTE as f64),
            n if n < GIGABYTE => format!("{:.1} MB", n as f64 / MEGABYTE as f64),
            n => format!("{:.1} GB", n as f64 / GIGABYTE as f64),
        }
    }

    pub fn format_modified(&self) -> String {
        let Some(modified) = self.modified else {
            return "-".to_string();
        };
        let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) else {
            return "-".to_string();
        };
        let secs = duration.as_secs() as i64;
        // UTC YYYY-MM-DD HH:MM without extra deps.
        const DAY: i64 = 86_400;
        const HOUR: i64 = 3_600;
        const MIN: i64 = 60;
        let days = secs / DAY;
        let mut year = 1970;
        let mut rem_days = days;
        loop {
            let diy = if is_leap(year) { 366 } else { 365 };
            if rem_days < diy {
                break;
            }
            rem_days -= diy;
            year += 1;
        }
        let month_lengths = [
            31,
            if is_leap(year) { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        let mut month = 1;
        for length in month_lengths {
            if rem_days < length {
                break;
            }
            rem_days -= length;
            month += 1;
        }
        let day = rem_days + 1;
        let day_secs = secs % DAY;
        let hour = day_secs / HOUR;
        let minute = (day_secs % HOUR) / MIN;
        format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
    }
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
