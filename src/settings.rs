use util;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub public_url_base: String,
    pub dist_path: String,

    pub dist_coverage_path: String,
    pub dist_coverage_url: String,
    pub patterns_base: String,

    pub patterns: Vec<String>,
}

impl Settings {
    pub fn new(
        url_base: impl Into<String>,
        serving_base: impl Into<String>,
        code_base: impl Into<String>,
        code_serve_base: impl Into<String>,
        file_base: impl Into<String>,
        patterns: Vec<impl Into<String>>,
    ) -> Settings {
        Settings {
            public_url_base: url_base.into(),
            dist_path: serving_base.into(),
            dist_coverage_path: code_base.into(),
            dist_coverage_url: code_serve_base.into(),
            patterns_base: file_base.into(),
            patterns: patterns.into_iter().map(|s| s.into()).collect(),
        }
    }
}

pub fn from_root() -> Settings {
    use std::env;

// We assume that we are in a valid directory.
    let path = env::current_dir().unwrap();
    let settings_path = path.join("settings.json");
    util::deserialize_object(settings_path).unwrap()
}