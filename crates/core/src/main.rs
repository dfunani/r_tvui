use chrono::Local;
use core::formatters::{format_size, format_time};

fn main() {
    println!("Hello, world!");
    let now = Local::now();
    println!("{:?}", format_size(7650600000));
    println!("{:?}", format_time(now));
}
