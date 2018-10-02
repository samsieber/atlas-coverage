#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate vlq;
extern crate xml;
extern crate globset;

pub mod debug;
pub mod settings;

mod lines;
mod load;
mod model;
mod range;
mod source_map;
mod vlq_decode;
mod util;

use std::path::Path;

use lines::calculate_executable_line_mappings;
use lines::calculate_line_coverage;
use lines::FileCoverage;
use lines::ManyCoverage;
use model::{PuppeteerData, SourceMap};
use settings::Settings;
use source_map::*;
use std::env::args;

pub fn process_source_map(settings: &Settings, data: PuppeteerData) -> Option<Vec<FileCoverage>> {
    if let Some(source_mapping_url) = data.get_source_mapping_url() {
        let source_path = data.url.replace(&settings.public_url_base, &settings.dist_path);

        let source_mapping_url = source_mapping_url.replace("//# sourceMappingURL=", "");
        let source_mapping_path = Path::new(&source_path)
            .parent()
            .unwrap()
            .join(source_mapping_url);

        let source_mapping_path = Path::new(&source_mapping_path);
        if source_mapping_path.exists() {
            let source_map: SourceMap = util::deserialize_object(source_mapping_path).unwrap();

            let references = process_references(&settings, &source_map);

            let file_refs = references.iter().map(|s| s.file_path.clone()).collect();
            let line_refs = calculate_executable_line_mappings(&source_map, references);
            let file_coverage =
                calculate_line_coverage(data.ranges, line_refs, file_refs, data.text.as_str());

            Some(file_coverage)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn run(settings: Settings, json_path: Vec<impl AsRef<Path>>) {
    let values = load::load_items(json_path);
    let processed: Vec<_> = values
        .into_iter()
        .map(|value| process_source_map(&settings, value))
        .flat_map(|value| value.into_iter())
        .flat_map(|value| value.into_iter())
        .collect();

    let many_coverage = ManyCoverage { files: processed };
    many_coverage.write_xml_to_stdout();
}

#[cfg(test)]
mod test {
    use std::path::Path;
}
