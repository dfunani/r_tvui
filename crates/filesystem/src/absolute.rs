use std::path::Path;
use core::paths::AbsolutePath;
use std::env::current_dir;

pub fn absolute(path: &Path) -> AbsolutePath{
    if path.is_absolute(){
        return AbsolutePath(path.to_path_buf());
    }
    let current_dir = current_dir().unwrap();
    let absolute_path = current_dir.join(&path);
    return AbsolutePath(absolute_path);
}
pub fn parent(path: AbsolutePath) -> Option<AbsolutePath>{
    path.0.parent().map(|p| absolute(&p))
}

pub fn join(parent: &AbsolutePath, name: &str) -> AbsolutePath{
    AbsolutePath(parent.0.join(name))
}

/// Jail mode (config): ensure canonical path starts with allowed root
pub fn assert_allowed(path: &AbsolutePath, roots: &[AbsolutePath]) -> bool{
    for root in roots{
        if !path.0.starts_with(&root.0){
            return false;
        }
    }
    return true;
}
