extern crate e2e_cc;

use std::env::current_dir;
use std::env::args;
use std::fs;

fn main() {
    let args: Vec<String> = args().collect();

    let settings = e2e_cc::settings::from_root();

    let reads = fs::read_dir(current_dir().expect("Not in a valid directory").join(&args[1])).expect("Cannot read directory");

    let paths = reads.into_iter().map(|v| v.expect("Cannot read file entry").path()).collect::<Vec<_>>();

    e2e_cc::run(settings, paths);
}
