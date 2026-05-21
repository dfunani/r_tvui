use chrono::{DateTime, Local};

pub fn format_time(input_time: DateTime<Local>) -> String{
    return input_time.format("%Y-%m-%d %H:%M:%S").to_string();
}

pub fn format_size(input_size: u64) -> String{
    let size = input_size as f64;
    println!("size: {}", size);
    match size {
        size if size < 0.0 => {
            return "0.0".to_string();
        }
        size if size < 1000.0 => {
            return format!("{:.1}", size).to_string();
        }
        size if size >= 1000.0 && size < 1000000.0 => {
            return format!("{:.1}K", size / 1000.0).to_string();
        }
        size if size >= 10000000.0 && size < 1000000000.0 => {
            return format!("{:.1}M", size / 1000000.0).to_string();
        }
        size if size >= 10000000.0 => {
            return format!("{:.1}B", size / 1000000000.0).to_string();
        }
        _ => {
            return "0.0".to_string();
        }
    };
}