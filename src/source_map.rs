use util::fast_read;
use model::SourceMap;
use settings::Settings;
use std::path::Path;
use std::path::PathBuf;
use util;

#[derive(Debug, Clone)]
pub enum SourceType {
    NodeModules,
    Generated,
    User,
}

#[derive(Debug, Clone)]
pub enum FilePath {
//    Conflict(PathBuf),
    Resolved(PathBuf),
    Unresolved(String),
}

#[derive(Debug, Clone)]
pub struct SourceMapSource {
    pub file_path: FilePath,
    pub source_type: SourceType,
    pub content: Option<String>,
}

#[derive(Debug)]
pub struct NoContentSourceMapSource {
    pub file_path: FilePath,
    pub source_type: SourceType,
}

pub fn process_references(settings: &Settings, source_map: &SourceMap) -> Vec<SourceMapSource> {
    source_map
        .sources
        .iter()
        .enumerate()
        .map(|(ref i, ref source_url)| {
            let mut abs_source_path = source_url.replace(&settings.dist_coverage_url, &settings.dist_coverage_path);
            if abs_source_path.contains(".pre-build-optimizer.js") {
                let rewritten = abs_source_path.replace(".pre-build-optimizer.js", "").to_owned();
                let content = util::fast_read(&rewritten).ok();
                let content_equal = source_map.sourcesContent.as_ref()
                    .and_then(|contents| contents.get(*i).map(|inner_content| inner_content == &content)).unwrap_or(false);
                if content_equal {
                    abs_source_path = rewritten
                }
            }


            let path = Path::new(&abs_source_path).canonicalize();
            let file_path = match path {
                Ok(resolved) => {
                    abs_source_path = resolved.to_string_lossy().to_string();
//
//                    let content = util::fast_read(&abs_source_path).ok();
//                    let content_equal = source_map.sourcesContent.as_ref()
//                        .and_then(|contents| contents.get(*i).map(|inner_content| inner_content == &content)).unwrap_or(false);
//                    if content_equal {
                        FilePath::Resolved(resolved)
//                    } else {
//                        FilePath::Conflict(resolved)
//                    }
                },
                _ => FilePath::Unresolved(abs_source_path.clone()),
            };

            let source_type = if abs_source_path.contains("node_modules") {
                SourceType::NodeModules
            } else if settings.patterns.iter().any(|ref p| abs_source_path.replace(&settings.patterns_base, "").starts_with(p.as_str())) {
                SourceType::User
            } else {
                SourceType::Generated
            };

            SourceMapSource {
                file_path,
                source_type,
                content: source_map
                    .sourcesContent
                    .as_ref()
                    .and_then(|contents| contents[*i].clone()),
            }
        }).collect()
}

impl SourceMapSource {
    pub fn convert_to_user_source(self) -> Option<String> {
        if let FilePath::Resolved(ref path_buf) = self.file_path {
            if let SourceType::User = self.source_type {
                if path_buf.to_string_lossy().ends_with(".js")
                    || path_buf.to_string_lossy().ends_with(".ts")
                {
                    Some(self.content.unwrap_or_else(|| fast_read(path_buf).unwrap()))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn meta(&self) -> NoContentSourceMapSource {
        NoContentSourceMapSource {
            file_path: self.file_path.clone(),
            source_type: self.source_type.clone(),
        }
    }
}
