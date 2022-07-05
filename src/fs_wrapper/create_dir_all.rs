use std::{fs, io};
use std::process::Command;

use super::RclonePath;

pub fn create_dir_all(path: &RclonePath) -> io::Result<()> {
    let use_rclone = path.is_remote();

    let path = path.clone().to_string();

    if use_rclone {
        Command::new("rclone")
            .arg("mkdir")
            .arg(path)
            .status()?;
    } else {
        fs::create_dir_all(path)?;
    }
    Ok(())
}
