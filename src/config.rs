use std::str::FromStr;

use serde::Deserialize;
use serde_with::{FromInto, serde_as};

use crate::fs_wrapper::RclonePath;

#[serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    // Allow input and output directory to be either a string or specify the remote and directory as strings
    #[serde_as(as = "FromInto<RclonePathWrapper>")]
    pub input_directory: RclonePath,
    #[serde_as(as = "FromInto<RclonePathWrapper>")]
    pub output_directory: RclonePath,
    pub temp_directory: Option<String>,
    pub extensions_to_encode: Vec<String>,
    pub encoded_extension: String,
    pub copy_covers: Option<bool>,
    pub ffmpeg_params: String,
    pub remove_round_brackets: Option<bool>,
    pub remove_square_brackets: Option<bool>,
    pub remove_curly_brackets: Option<bool>,
    pub remove_angle_brackets: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum RclonePathWrapper {
    String(String),
    NamedRclonePath(NamedRclonePath),
}

#[derive(Deserialize, Debug)]
struct NamedRclonePath {
    remote: Option<String>,
    path: Option<String>,
}

impl From<RclonePathWrapper> for RclonePath {
    fn from(rclone_path_wrapper: RclonePathWrapper) -> Self {
        match rclone_path_wrapper {
            RclonePathWrapper::String(string) => {
                RclonePath::from_str(&string).unwrap()
            }
            RclonePathWrapper::NamedRclonePath(named_rclone_path) => {
                if let Some(remote) = named_rclone_path.remote {
                    if remote == "" {
                        RclonePath::Local(named_rclone_path.path.unwrap_or(String::from("")))
                    } else {
                        RclonePath::Remote(remote, named_rclone_path.path.unwrap_or(String::from("")))
                    }
                } else {
                    RclonePath::Local(named_rclone_path.path.unwrap_or(String::from("")))
                }
            }
        }
    }
}

impl From<RclonePath> for RclonePathWrapper {
    fn from(rclone_path: RclonePath) -> Self {
        match rclone_path {
            RclonePath::Local(path) => {
                Self::String(path)
            }
            RclonePath::Remote(remote, path) => {
                Self::NamedRclonePath(
                    NamedRclonePath {
                        remote: Some(remote),
                        path: Some(path),
                    }
                )
            }
        }
    }
}