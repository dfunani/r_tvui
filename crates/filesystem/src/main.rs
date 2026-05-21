use rtvui_core::paths::AbsolutePath;
use filesystem::directories::{list_directories_async, DirectoryListOptions, DirectorySortOrder};
use filesystem::utils::{classify_file_type, is_hidden_name};
use std::fs;
use std::path::Path;
use tokio::runtime::Runtime;

fn main() {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
        let path = Path::new("src");
        let meta = fs::metadata(path).unwrap();
        let file_type = classify_file_type(path, &meta);
        let hidden = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(is_hidden_name)
            .unwrap_or(false);
        println!("file_type: {file_type:?}");
        println!("hidden: {hidden}");
        let path = AbsolutePath(path.to_path_buf());
        let result = list_directories_async(
            path,
            DirectoryListOptions {
                show_hidden: false,
                sort: DirectorySortOrder::Name,
                include_parent_link: true,
            },
        )
        .await;
        match result {
            Ok(result) => println!("entries: {}", result.entries.len()),
            Err(e) => println!("error: {e:?}"),
        }
    });
}
