use std::fs;
use std::path::Path;
use std::process::Command;

use super::RclonePath;

pub fn list_files_recursively(path: &RclonePath) -> Vec<RclonePath> {
    if path.is_remote() {
        let stdout = Command::new("rclone")
            .arg("lsf")
            .arg("-R")
            .arg("--files-only")
            .arg(path.clone().to_string())
            .output()
            .expect("Failed to run rclone")
            .stdout;
        String::from_utf8_lossy(&*stdout).to_string().lines().map(
            |line| {
                match path {
                    RclonePath::Local(path) => {
                        RclonePath::Local(
                            format!(
                                "{}/{}",
                                path,
                                line.to_string()
                            )
                        )
                    }
                    RclonePath::Remote(remote, path) => {
                        RclonePath::Remote(
                            remote.clone(),
                            format!(
                                "{}/{}",
                                path,
                                line.to_string()
                            )
                        )
                    }
                }
            }
        ).collect()
    } else {
        traverse_local_directory(path.clone().to_string()).into_iter().map(
            |file| {
                RclonePath::Local(file)
            }
        ).collect()
    }
}

fn traverse_local_directory<P: AsRef<Path>>(path: P) -> Vec<String> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let mut contents = traverse_local_directory(entry.path());
                        result.append(&mut contents);
                    } else {
                        result.push(entry.path().to_string_lossy().to_string())
                    }
                }
            }
        }
    }
    result
}