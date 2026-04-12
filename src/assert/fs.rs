//! Filesystem existence checks for tests.

use std::path::Path;

/// Asserts that `path` exists and is a regular file.
///
/// # Panics
///
/// Panics when the path is missing, not a file, or metadata cannot be read.
///
/// # Examples
///
/// ```
/// use std::fs;
/// use suitecase::assert::file_exists;
///
/// let path = std::env::temp_dir().join("suitecase_doc_file_exists");
/// let _ = fs::remove_file(&path);
/// fs::write(&path, b"x").unwrap();
/// file_exists(&path);
/// let _ = fs::remove_file(&path);
/// ```
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

/// Asserts that `path` does not refer to an existing regular file.
///
/// # Panics
///
/// Panics when `path` exists and is a file.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use suitecase::assert::no_file_exists;
///
/// no_file_exists(Path::new("/unlikely/path/that/does/not/exist/12345"));
/// ```
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

/// Asserts that `path` exists and is a directory.
///
/// # Panics
///
/// Panics when the path is missing, not a directory, or metadata cannot be read.
///
/// # Examples
///
/// ```
/// use std::fs;
/// use suitecase::assert::dir_exists;
///
/// let path = std::env::temp_dir().join("suitecase_doc_dir_exists");
/// let _ = fs::remove_dir_all(&path);
/// fs::create_dir_all(&path).unwrap();
/// dir_exists(&path);
/// let _ = fs::remove_dir_all(&path);
/// ```
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

/// Asserts that `path` does not refer to an existing directory.
///
/// # Panics
///
/// Panics when `path` exists and is a directory.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use suitecase::assert::no_dir_exists;
///
/// no_dir_exists(Path::new("/unlikely/directory/not/here/99999"));
/// ```
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
