use std::{fs, io};
use std::process::Command;

use super::RclonePath;

pub fn copy(from: &RclonePath, to: &RclonePath) -> io::Result<()> {
    let use_rclone = from.is_remote() || to.is_remote();

    let from = from.clone().to_string();
    let to = to.clone().to_string();

    if use_rclone {
        Command::new("rclone")
            .arg("copyto")
            .arg(from)
            .arg(to)
            .status()?;
    } else {
        fs::copy(from, to)?;
    }
    Ok(())
}

