use std::fs;
use std::path::Path;

use filesystem::utils::{classify_file_type, is_hidden_file_type};
use filesystem::directories::{list_directories_async, DirectoryListOptions, DirectorySortOrder};
use core::paths::AbsolutePath;
use tokio::runtime::Runtime;

fn main() {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
    println!("Hello, world!");
    let path = Path::new("src");
    let meta = fs::metadata(path).unwrap();
    let file_type = classify_file_type(path, &meta);
    let hidden = is_hidden_file_type(path, &file_type);
    println!("file_type: {:?}", file_type);
    println!("hidden: {:?}", hidden);
    let path = AbsolutePath(path.to_path_buf());
    let result = list_directories_async(path, DirectoryListOptions {
        show_hidden: false,
        sort: DirectorySortOrder::Name,
    }).await;
    match result {
        Ok(result) => println!("result: {:?}", result),
        Err(e) => println!("error: {:?}", e),
    }
    });
}