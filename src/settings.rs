use util;
use globset::{Glob, GlobSetBuilder, GlobSet};
use serde::Deserializer;
use serde::de::Visitor;
use std::fmt;
use serde::de::SeqAccess;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub public_url_base: String,
    pub dist_path: String,

    pub dist_coverage_path: String,
    pub dist_coverage_url: String,

    pub sources: Sources,
}

#[derive(Debug, Deserialize)]
pub struct Sources {
    pub base: String,
    #[serde(deserialize_with="deserialize_globset")]
    pub dirs: GlobSet,
    #[serde(deserialize_with="deserialize_globset")]
    pub excludes: GlobSet,
}

fn deserialize_globset<'de, D>(deserializer: D) -> Result<GlobSet, D::Error>
    where D: Deserializer<'de>,
{
    struct GlobSetVisitor{};

    impl<'de> Visitor<'de> for GlobSetVisitor {
        type Value = GlobSet;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "expected a list of glob strings")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<GlobSet, S::Error> where S: SeqAccess<'de>,
        {
            let mut builder = GlobSetBuilder::new();

            while let Some(value) = seq.next_element::<String>()? {
                builder.add(Glob::new(&value).unwrap());
            }

            Ok(builder.build().unwrap())
        }
    }

    // Create the visitor and ask the deserializer to drive it. The
    // deserializer will call visitor.visit_seq() if a seq is present in
    // the input data.
    deserializer.deserialize_seq(GlobSetVisitor{})
}
pub fn from_root() -> Settings {
    use std::env;

// We assume that we are in a valid directory.
    let path = env::current_dir().unwrap();
    let settings_path = path.join("settings.json");
    util::deserialize_object(settings_path).unwrap()
}