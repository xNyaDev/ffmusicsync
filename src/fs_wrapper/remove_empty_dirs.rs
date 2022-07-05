use std::{fs, io};
use std::path::Path;
use std::process::Command;

use super::RclonePath;

pub fn remove_empty_dirs(path: &RclonePath) -> io::Result<()> {
    let use_rclone = path.is_remote();

    let path = path.clone().to_string();

    if use_rclone {
        Command::new("rclone")
            .arg("rmdirs")
            .arg(path)
            .status()?;
    } else {
        if traverse_local_directory(&path)? {
            fs::remove_dir(path)?;
        }
    }
    Ok(())
}

fn traverse_local_directory<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                count += 1;
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if traverse_local_directory(entry.path())? {
                            fs::remove_dir(entry.path())?;
                            count -= 1;
                        }
                    }
                }
            }
        }
    }
    if count == 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}