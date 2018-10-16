extern crate atlas_coverage_core;

use atlas_coverage_core as e2e_cc;

fn main() {
    let settings = e2e_cc::settings::from_root().unwrap();
    e2e_cc::debug::print_existing(settings);
}
