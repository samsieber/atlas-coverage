extern crate e2e_cc;

fn main() {
    let settings = e2e_cc::settings::from_root();
    e2e_cc::debug::print_existing(settings)
}
