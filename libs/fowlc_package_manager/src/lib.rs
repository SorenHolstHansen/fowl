use semver::Version;
use std::path::PathBuf;

pub struct Package {
    pub name: String,
    pub path: PathBuf,
    pub version: Version,
}
