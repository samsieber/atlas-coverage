use model::SourceMap;
use range::RangeStack;
use source_map::FilePath;
use source_map::SourceMapSource;
use std::collections::BTreeMap;
use std::io;
use std::io::Write;
use vlq_decode;
use xml::writer::XmlEvent;
use xml::EventWriter;

type LineRefs = Vec<Option<BTreeMap<usize, Vec<Coord>>>>;

#[derive(Debug)]
pub struct Coord {
    line: usize,
    col: usize,
}

#[derive(Debug)]
pub struct FileCoverage {
    pub path: String,
    pub lines: Vec<LineCoverage>,
}

#[derive(Debug)]
pub struct LineCoverage {
    pub line_number: usize,
    pub covered: bool,
}

pub fn calculate_executable_line_mappings(
    source_map: &SourceMap,
    references: Vec<SourceMapSource>,
) -> LineRefs {
    //    println!("{:#?}", vlq_decode::MappingData::new(source_map.mappings.as_bytes()).collect::<Vec<_>>());
    //    println!("{:#?}", &references);
    //    source_map.sources.iter().enumerate().for_each(|(i,v)| {
    //        if v.contains("sf-capc-feedback") {
    //            println!("{} - {}", v, source_map.sourcesContent.as_ref().unwrap().get(i).as_ref().unwrap().as_ref().unwrap())
    //        }
    //    });

    let mut sources: LineRefs = references
        .into_iter()
        .map(SourceMapSource::convert_to_user_source)
        .map(|v| v.map(|_| BTreeMap::new()))
        .collect::<Vec<_>>();

    for mut v in vlq_decode::MappingData::new(source_map.mappings.as_bytes()) {
        if let Some(ref source_ref) = v.source {
            if let Some(ref mut source_map) =
                sources.get_mut(source_ref.source_idx as usize).unwrap()
            {
                let coords = source_map
                    .entry(source_ref.source_line as usize)
                    .or_insert(vec![]);
                (*coords).push(Coord {
                    col: v.gen_column as usize,
                    line: v.gen_line as usize,
                });
            }
        }
    }

    sources
}

pub fn calculate_line_coverage(
    ranges: RangeStack,
    line_refs: LineRefs,
    sources: Vec<FilePath>,
    minified: &str,
) -> Vec<FileCoverage> {
    let idxs = {
        let mut sum = 0;
        minified
            .lines()
            .into_iter()
            .map(|line| {
                let ret = sum;
                sum += 1;
                sum = sum + line.len();
                ret
            }).collect::<Vec<usize>>()
    };

    sources
        .into_iter()
        .zip(line_refs)
        .flat_map(|(source, maybe_exec_lines)| {
            maybe_exec_lines.map(|exec_lines| FileCoverage {
                lines: exec_lines
                    .into_iter()
                    .map(|(line_number, coords)| LineCoverage {
                        line_number: line_number + 1,
                        covered: coords
                            .iter()
                            .map(|v| idxs[v.line] + v.col)
                            .any(|byte_idx| ranges.contains_value(byte_idx)),
                    }).collect(),
                path: match source {
                    FilePath::Resolved(path_buf) => path_buf.to_string_lossy().to_string(),
                    FilePath::Unresolved(_) => panic!("The file does not exist!"),
                },
            })
        }).collect::<Vec<_>>()
}

pub struct ManyCoverage {
    pub files: Vec<FileCoverage>,
}

impl ManyCoverage {
    pub fn write_xml<W: Write>(&self, writer: W) {
        let mut xml_writer = EventWriter::new(writer);
        xml_writer
            .write(XmlEvent::start_element("coverage").attr("version", "1"))
            .unwrap();
        for file in &self.files {
            xml_writer
                .write(XmlEvent::start_element("file").attr("path", &file.path))
                .unwrap();
            for line in &file.lines {
                xml_writer
                    .write(
                        XmlEvent::start_element("lineToCover")
                            .attr("lineNumber", &format!("{}", line.line_number))
                            .attr("covered", &format!("{}", line.covered)),
                    ).unwrap();
                xml_writer.write(XmlEvent::end_element()).unwrap();
            }
            xml_writer.write(XmlEvent::end_element()).unwrap();
        }
        xml_writer.write(XmlEvent::end_element()).unwrap();
    }

    pub fn write_xml_to_stdout(&self) {
        let stdout = io::stdout();
        let handle = stdout.lock();

        self.write_xml(handle);
    }
}
