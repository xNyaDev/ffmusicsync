use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum RclonePath {
    Local(String),
    Remote(String, String),
}

impl RclonePath {
    pub fn to_string(self) -> String {
        match self {
            Self::Local(path) => {
                path
            }
            Self::Remote(remote, path) => {
                format!("{}:{}", remote, path)
            }
        }
    }
    pub fn path_string(self) -> String {
        match self {
            Self::Local(path) => {
                path
            }
            Self::Remote(_, path) => {
                path
            }
        }
    }
    pub fn is_remote(&self) -> bool {
        match self {
            Self::Local(_) => false,
            Self::Remote(_, _) => true
        }
    }
    pub fn with_path(&self, path: String) -> Self {
        match self {
            Self::Local(_) => {
                Self::Local(path)
            }
            Self::Remote(remote, _) => {
                Self::Remote(
                    remote.clone(),
                    path
                )
            }
        }
    }
}

impl FromStr for RclonePath {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.contains(":") {
            let remote_and_directory = s.split(":").map(|x| x.to_string()).collect::<Vec<String>>();
            Self::Remote(
                remote_and_directory.get(0).unwrap().to_string(),
                remote_and_directory.get(1).unwrap_or(&String::from("")).to_string(),
            )
        } else {
            Self::Local(s.to_string())
        })
    }
}