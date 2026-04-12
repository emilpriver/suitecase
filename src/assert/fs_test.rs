use std::fs;
use std::io::Write;
use std::path::PathBuf;

use super::fs::*;

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("suitecase_assert_{}_{}", name, std::process::id()))
}

#[test]
fn file_exists_ok() {
    let p = temp_path("file_exists");
    let _ = fs::remove_file(&p);
    let mut f = fs::File::create(&p).unwrap();
    f.write_all(b"x").unwrap();
    drop(f);
    file_exists(&p);
    let _ = fs::remove_file(&p);
}

#[test]
fn no_file_exists_ok() {
    let p = temp_path("no_file");
    let _ = fs::remove_file(&p);
    no_file_exists(&p);
}

#[test]
fn dir_exists_ok() {
    let p = temp_path("dir_exists");
    let _ = fs::remove_dir_all(&p);
    fs::create_dir(&p).unwrap();
    dir_exists(&p);
    let _ = fs::remove_dir_all(&p);
}

#[test]
fn no_dir_exists_ok() {
    let p = temp_path("no_dir");
    let _ = fs::remove_dir_all(&p);
    no_dir_exists(&p);
}
