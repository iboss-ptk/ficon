extern crate exitcode;
extern crate ficon;
extern crate serde;
extern crate termion;
extern crate toml;

use ficon::Ficon;
use ignore::Walk;
use termion::{color, style};

fn main() {
    let ficon = Ficon::new();
    let mut ok = true;

    for result in Walk::new(ficon.target_dir()).skip(1) {
        let entry = result.unwrap();
        let canonical_path = entry
            .path()
            .canonicalize()
            .expect("can't create canonical path");

        if ficon.check(&entry.clone().into_path()) {
            println!(
                "{green}{path}{reset}",
                path = format!(
                    "{}✓ {}",
                    " ".repeat(entry.depth()),
                    canonical_path.file_name().unwrap().to_str().unwrap()
                ),
                green = color::Fg(color::LightGreen),
                reset = style::Reset
            );
        } else {
            println!(
                "{bold}{red}{path}{reset}",
                path = format!(
                    "{}✘ {}",
                    "  ".repeat(entry.depth()),
                    canonical_path.file_name().unwrap().to_str().unwrap()
                ),
                red = color::Fg(color::LightRed),
                bold = style::Bold,
                reset = style::Reset
            );
            ok = false;
        };
    }

    if !ok {
        std::process::exit(exitcode::DATAERR)
    }
}
