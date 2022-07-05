use std::{fs, io};
use std::process::Command;

use super::RclonePath;

pub fn remove_file(path: &RclonePath) -> io::Result<()> {
    let use_rclone = path.is_remote();

    let path = path.clone().to_string();

    if use_rclone {
        Command::new("rclone")
            .arg("delete")
            .arg(path)
            .status()?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

