extern crate exitcode;
extern crate ficon;
extern crate serde;
extern crate termion;
extern crate toml;

use exitfailure::ExitFailure;
use failure::ResultExt;
use ficon::{filename_of, Ficon};
use human_panic::setup_panic;
use ignore::Walk;
use std::{io, path::Path};
use termion::{color, style};

fn main() -> Result<(), ExitFailure> {
    setup_panic!();

    let mut ficon = Ficon::new()?;
    let mut all_files_passed = true;
    let stdout = io::stdout();
    let mut locked_stdout = stdout.lock();

    // skip first entry since it's the root dir and we only care about content inside
    for result in Walk::new(ficon.target_dir()).skip(1) {
        let entry = result.with_context(|_| "can't retrieve directory entry")?;
        let path = entry.path();

        let file_passed = ficon.check(path)?;
        print_check_result(&mut locked_stdout, path, entry.depth(), file_passed)?;

        all_files_passed = all_files_passed && file_passed;
    }

    if !all_files_passed {
        std::process::exit(exitcode::DATAERR)
    }

    Ok(())
}

fn print_check_result(
    mut out: impl io::Write,
    path: &Path,
    depth: usize,
    is_passed: bool,
) -> Result<(), io::Error> {
    let depth_space = "  ".repeat(depth);
    let file_name = filename_of(path);

    if is_passed {
        writeln!(
            out,
            "{green}{path}{reset}",
            path = format!("{}✓ {}", depth_space, file_name),
            green = color::Fg(color::LightGreen),
            reset = style::Reset
        )?;
    } else {
        writeln!(
            out,
            "{bold}{red}{path}{reset}",
            path = format!("{}✘ {}", depth_space, file_name),
            red = color::Fg(color::LightRed),
            bold = style::Bold,
            reset = style::Reset
        )?;
    };
    Ok(())
}
