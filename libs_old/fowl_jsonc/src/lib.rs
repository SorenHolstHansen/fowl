use jsonc_parser::{errors::ParseError, parse_to_serde_value};
use serde::Deserialize;

pub enum Error {}

#[derive(Deserialize)]
pub struct FowlJsonc {
    name: String,
    description: String,
    version: String,
}

impl FowlJsonc {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

pub fn parse_fowl_jsonc(src: &str) -> Result<FowlJsonc, ParseError> {
    let json_value = parse_to_serde_value(src, &Default::default())?;
    let fowl_jsonc = serde_json::from_value(json_value.unwrap()).unwrap();

    Ok(fowl_jsonc)
}
