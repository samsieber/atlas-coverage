//use ::serde_derive::{Serialize, Deserialize};

use crate::range::RangeStack;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct PuppeteerData {
    pub url: String,
    pub text: String,

    #[serde(flatten)]
    pub ranges: RangeStack,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct SourceMap {
    pub version: u32,
    pub file: String,
    pub sourceRoot: String,
    pub sources: Vec<String>,
    pub sourcesContent: Option<Vec<Option<String>>>,
    pub names: Vec<String>,
    pub mappings: String,
}

impl PuppeteerData {
    pub fn get_source_mapping_url(&self) -> Option<String> {
        let last_part = self.text.lines().last().unwrap_or("No Content").to_owned();
        if last_part.starts_with("//# sourceMappingURL=") {
            Some(last_part.replace("//# sourceMappingURL=", ""))
        } else {
            None
        }
    }
}
