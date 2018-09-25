extern crate e2e_cc;

use std::env::args;

fn main() {
    let args: Vec<String> = args().collect();
    let settings = e2e_cc::settings::from_root();

    println!("{:#?}", &settings);

    e2e_cc::debug::print_existing(settings, args[1].clone())
}
