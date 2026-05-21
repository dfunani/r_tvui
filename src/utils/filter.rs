use rtvui_core::paths::File;

pub fn apply_name_filter(entries: &[File], query: &str) -> Vec<File> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return entries.to_vec();
    }

    entries
        .iter()
        .filter(|e| e.is_parent_link || e.name.to_lowercase().contains(&query))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rtvui_core::paths::{AbsolutePath, File, FileType};
    use std::path::PathBuf;

    fn file(name: &str) -> File {
        File {
            path: AbsolutePath(PathBuf::from(format!("/tmp/{name}"))),
            name: name.to_string(),
            kind: FileType::File,
            size: None,
            modified: None,
            hidden: false,
            is_parent_link: false,
            git_status: None,
        }
    }

    #[test]
    fn keeps_parent_link_when_filtering() {
        let mut parent = file("..");
        parent.is_parent_link = true;
        let entries = vec![parent, file("alpha.txt"), file("beta.rs")];
        let filtered = apply_name_filter(&entries, "alp");
        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].is_parent_link);
        assert_eq!(filtered[1].name, "alpha.txt");
    }
}
