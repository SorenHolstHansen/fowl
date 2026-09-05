use jsonc_parser::{errors::ParseError, parse_to_serde_value};
use semver::VersionReq;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct PathDependency {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ManifestDependency {
    Version(VersionReq),
    Path(PathDependency),
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    name: String,
    description: String,
    version: String,
    dependencies: HashMap<String, ManifestDependency>,
}

impl Manifest {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn add_dependency<N: Into<String>>(&mut self, name: N, dependency: ManifestDependency) {
        self.dependencies.insert(name.into(), dependency);
    }

    pub fn parse_str(src: &str) -> Result<Manifest, ParseError> {
        let json_value = parse_to_serde_value(src, &Default::default())?;
        let fowl_jsonc = serde_json::from_value(json_value.unwrap()).unwrap();

        Ok(fowl_jsonc)
    }
}
