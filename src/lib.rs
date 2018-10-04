#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate vlq;
extern crate xml;
extern crate globset;
extern crate lcov_parser;

pub mod debug;
pub mod settings;

mod lcov;
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
use std::collections::HashMap;
use std::collections::HashSet;
use std::env::args;
use std::io::Write;
use std::io;

pub fn process_source_map(settings: &Settings, data: PuppeteerData) -> Option<Vec<FileCoverage>> {
    if let Some(source_mapping_url) = data.get_source_mapping_url() {
        let source_path = data.url.replace(&settings.public_url_base, &settings.dist_path);

        let source_mapping_url = source_mapping_url.replace("//# sourceMappingURL=", "");
        let source_mapping_path = Path::new(&source_path)
            .parent()
            .unwrap()
            .join(source_mapping_url);

        println!("Processing source map {}", source_mapping_path.to_string_lossy());

        let source_mapping_path = Path::new(&source_mapping_path);
        if source_mapping_path.exists() {
            let source_map: SourceMap = util::deserialize_object(source_mapping_path).unwrap();

            let references = process_references(&settings, &source_map);

            let file_refs = references.iter().map(|s| s.file_path.clone()).collect();
            let line_refs = calculate_executable_line_mappings(&source_map, references);
            let mut file_coverage =
                calculate_line_coverage(data.ranges, line_refs, file_refs, data.text.as_str());

            if let Some(ref reify_against_lcov) = settings.reify_against_lcov {
                file_coverage = {
                    eprintln!("Reifying against LCOV file");

                    let mut file_hash_map: HashMap<_,_> = file_coverage.into_iter().map(|v| (v.path.clone(), v)).collect();

                    for line_data in lcov::LcovFilesLines::new(&util::fast_read(&reify_against_lcov).unwrap()) {

                        let our_coverage = file_hash_map.get_mut(&line_data.file_path);
                        our_coverage.map(|our_coverage| {
                            let new_lines : HashSet<_> = line_data.lines.into_iter().collect();

                            our_coverage.lines.retain(|v| new_lines.contains(&v.line_number));
                        });
                    }

                    file_hash_map.into_iter().map(|(_k,v)| v).collect()
                }
            }

            Some(file_coverage)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn run<P: AsRef<Path>, W: Write>(settings: Settings, json_path: Vec<P>, writer: Option<W>) {
    let values = load::load_items(json_path);
    let processed: Vec<_> = values
        .into_iter()
        .map(|value| process_source_map(&settings, value))
        .flat_map(|value| value.into_iter())
        .flat_map(|value| value.into_iter())
        .collect();

    let many_coverage = ManyCoverage { files: processed };

    if let Some(writer) = writer {
        many_coverage.write_xml(writer);
    } else {
        let stdout = io::stdout();
        let handle = stdout.lock();

        many_coverage.write_xml(handle);
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;
}
