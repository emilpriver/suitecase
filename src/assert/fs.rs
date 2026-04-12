use std::path::Path;

#[track_caller]
pub fn file_exists(path: &Path) {
    match path.metadata() {
        Ok(m) if m.is_file() => {}
        Ok(_) => panic!(
            "assertion failed: `file_exists`: path exists but is not a file: {:?}",
            path
        ),
        Err(e) => panic!("assertion failed: `file_exists` {:?}: {e}", path),
    }
}

#[track_caller]
pub fn no_file_exists(path: &Path) {
    match path.metadata() {
        Ok(m) if m.is_file() => {
            panic!(
                "assertion failed: `no_file_exists`: file unexpectedly exists: {:?}",
                path
            );
        }
        Ok(_) => {}
        Err(_) => {}
    }
}

#[track_caller]
pub fn dir_exists(path: &Path) {
    match path.metadata() {
        Ok(m) if m.is_dir() => {}
        Ok(_) => panic!(
            "assertion failed: `dir_exists`: path exists but is not a directory: {:?}",
            path
        ),
        Err(e) => panic!("assertion failed: `dir_exists` {:?}: {e}", path),
    }
}

#[track_caller]
pub fn no_dir_exists(path: &Path) {
    match path.metadata() {
        Ok(m) if m.is_dir() => {
            panic!(
                "assertion failed: `no_dir_exists`: directory unexpectedly exists: {:?}",
                path
            );
        }
        Ok(_) => {}
        Err(_) => {}
    }
}
