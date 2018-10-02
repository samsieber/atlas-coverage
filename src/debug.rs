use settings::Settings;
use model::PuppeteerData;
use util::deserialize_object;
use model::SourceMap;
use source_map::SourceMapSource;
use vlq_decode;
use std::path::Path;
use util;
use lines::calculate_executable_line_mappings;
use source_map::process_references;
use lines::calculate_line_coverage;
use load;
use serde_json::Deserializer;
use std::env::current_dir;
use std::env::args;
use std::fs;
use lines::FileCoverage;

pub fn print_existing(settings: Settings, json_path: String) {
    let args: Vec<String> = args().collect();
    let reads = fs::read_dir(current_dir().expect("Not in a valid directory").join(&args[1])).expect("Cannot read directory");
    let paths = reads.into_iter().map(|v| v.expect("Cannot read file entry").path()).collect::<Vec<_>>();

    let values = load::load_items(paths);

    for v in values {
        print_if_has_existing_source_map(&settings, v);
    }
}

fn print_if_has_existing_source_map(settings: &Settings, data: PuppeteerData) {
    if let Some(source_mapping_url) = data.get_source_mapping_url() {
        let source_path = data.url.replace(&settings.public_url_base, &settings.dist_path);

        let source_mapping_url = source_mapping_url.replace("//# sourceMappingURL=", "");
        let source_mapping_path = Path::new(&source_path)
            .parent()
            .unwrap()
            .join(source_mapping_url.clone());

        println!("{} - {}", source_mapping_url, data.url);

        let source_mapping_path = Path::new(&source_mapping_path);
        if source_mapping_path.exists() {
            let source_map: SourceMap = util::deserialize_object(source_mapping_path).unwrap();

            let references = process_references(&settings, &source_map);

            let meta_refs = references.iter().map(|r| r.meta()).collect::<Vec<_>>();
//            let meta_refs = references.clone();

//            println!("{:#?}", &references);

//            debug_source_map(settings, &source_map, references)

            let file_refs = references.iter().map(|s| s.file_path.clone()).collect();
            let line_refs = calculate_executable_line_mappings(&source_map, references);
            let file_coverage : Vec<FileCoverage> =
                calculate_line_coverage(data.ranges, line_refs, file_refs, data.text.as_str());

            file_coverage.iter().for_each(|fc| {
                let content = util::fast_read(&fc.path);
                match content {
                    Ok(content) => {
                        let lines = content.lines().count();
                        let cov_lines = fc.lines.iter().last().map(|v| v.line_number);
                        match cov_lines {
                            Some(last_line) => {
                                println!("Within bounds? {: <5} || {: <5} - {: <5}", last_line <= lines, lines, last_line);
                            },
                            None => {
                                println!("Empty file");
                            }
                        }
                    },
                    Err(err) => {
                        println!("Within bounds? {: <5} - {} doesn't exist or is empty!", false, &fc.path)
                    },
                }
            })
//            println!("{}:\n------------------------------------------------------------------------------------------------\n{:#?}\n-=-=-=-=-=-=-=-=-\n{:#?}", source_mapping_path.to_string_lossy(), meta_refs, file_coverage);
        }
    }


}

// Currently unused, but useful code:

pub fn debug_source_map(
    _settings: &Settings,
    source_map: &SourceMap,
    references: Vec<SourceMapSource>,
    minified: String,
) {
    let sources = references
        .into_iter()
        .map(SourceMapSource::convert_to_user_source)
        .collect::<Vec<_>>();
    let mut prev: usize = 9999;
    let mut announce = None;
    for v in vlq_decode::MappingData::new(source_map.mappings.as_bytes()) {
        let minified_slice: String = minified
            .lines()
            .skip(v.gen_line as usize)
            .next()
            .unwrap()
            .chars()
            .skip(v.gen_column as usize)
            .take(20)
            .collect();
        let mut source_slice: Option<String> = None;
        let mut name: Option<String> = None;
        if let Some(ref source_ref) = v.source.as_ref() {
            if prev != source_ref.source_idx as usize {
                prev = source_ref.source_idx as usize;
                announce = Some(prev);
            }
            source_slice = sources
                .get(source_ref.source_idx as usize)
                .unwrap()
                .as_ref()
                .map(|ref source_content| {
                    source_content
                        .lines()
                        .skip(source_ref.source_line as usize)
                        .next()
                        .unwrap()
                        .chars()
                        .skip(source_ref.source_col as usize)
                        .take(20)
                        .collect()
                });
            if let Some(name_idx) = source_ref.name_idx {
                name = source_map
                    .names
                    .get(name_idx as usize)
                    .map(|v| v.to_owned());
            }
        }
        if let Some(announce_idx) = announce {
            println!("--------------------------------------------------------------------------------------------");
            println!("FILE {}", source_map.sources[announce_idx]);
            println!("--------------------------------------------------------------------------------------------");
            announce = None
        }
        if let Some(ref source_slice) = source_slice {
            let source = v.source.as_ref().unwrap();
            println!(
                "{: <1}.{: <5} > {: <3}.{: <5} | {: <20} | {: <20} | {: <20}",
                v.gen_line,
                v.gen_column,
                source.source_line,
                source.source_col,
                minified_slice,
                source_slice,
                name.unwrap_or("".to_owned())
            );
        } else {
            println!(
                "{: <1}.{: <5}             | {: <20} |",
                v.gen_line, v.gen_column, minified_slice
            );
        }
    }
}