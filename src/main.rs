extern crate exitcode;
extern crate ficon;
extern crate serde;
extern crate termion;
extern crate toml;

use ficon::Ficon;
use ignore::Walk;
use std::path::Path;
use termion::{color, style};
use failure::ResultExt;
use exitfailure::ExitFailure;
use human_panic::setup_panic;

fn main() -> Result<(), ExitFailure> {
    setup_panic!();

    let ficon = Ficon::new();
    let mut ok = true;

    // skip first entry since it's the root dir and we only care about content inside
    for result in Walk::new(ficon.target_dir()).skip(1) {
        let entry = result
            .with_context(|_| format!("can't retrieve directory entry"))?;

        let path = entry.path();

        let is_passed = ficon.check(path);
        if !is_passed {
            ok = false;
        }

        print_check_result(path, entry.depth(), is_passed);
    }

    if !ok {
        std::process::exit(exitcode::DATAERR)
    }

    Ok(())
}

fn print_check_result(path: &Path, depth: usize, is_passed: bool) {
    let depth_space = "  ".repeat(depth);
    let file_name = path
        .file_name()
        .expect("filename doesn't exist")
        .to_str()
        .expect("filename can't be casted to string");

    if is_passed {
        println!(
            "{green}{path}{reset}",
            path = format!("{}✓ {}", depth_space, file_name),
            green = color::Fg(color::LightGreen),
            reset = style::Reset
        );
    } else {
        println!(
            "{bold}{red}{path}{reset}",
            path = format!("{}✘ {}", depth_space, file_name),
            red = color::Fg(color::LightRed),
            bold = style::Bold,
            reset = style::Reset
        );
    };
}
