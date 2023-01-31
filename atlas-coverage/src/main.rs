extern crate structopt;
extern crate atlas_coverage_core;

use atlas_coverage_core as e2e_cc;
use std::path::PathBuf;
use structopt::StructOpt;
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::BufWriter;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "atlas-coverage")]
struct Opt {
    /// Where to write the xml
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,

    /// Path to configuration json. Uses the CWD if omitted
    #[structopt(short = "-c", long = "config", parse(from_os_str))]
    config: Option<PathBuf>,

    /// Input directory with .json files to parse
    #[structopt(name = "input", parse(from_os_str))]
    input: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>>{
    let opt = Opt::from_args();

    let settings = if let Some(path) = opt.config {
        e2e_cc::settings::from_path(path)
    } else {
        e2e_cc::settings::from_root()
    }.expect("Cannot read settings");

    let writer = {
        let output_file = opt.output;

        fs::create_dir_all(output_file.parent().unwrap())?;

        let unbuffered = OpenOptions::new().create(true).write(true).truncate(true).open(output_file).expect("Cannot open output file");

        BufWriter::new(unbuffered)
    };

    let inputs : Vec<_> = {
        let input_directory_items = fs::read_dir(opt.input)?;

        input_directory_items.into_iter()
            .flat_map(|potential_input|{
                let input_path = potential_input.expect("Encountered intermittent io error").path();

                if input_path.to_string_lossy().ends_with(".json") {
                    Some(input_path)
                } else {
                    eprintln!("Skipping non-json file: {}", input_path.to_string_lossy());
                    None
                }
            }).collect()
    };

    Ok(e2e_cc::run(settings, inputs, Some(writer)))
}