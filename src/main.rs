#[macro_use]
extern crate structopt;
extern crate termion;
extern crate exitcode;
extern crate regex;

use ignore::Walk;
use termion::{color, style};
use regex::Regex;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "ficon")]
struct Opt {
    /// Path to directory to check convention
    #[structopt(name = "PATH", default_value = ".", parse(from_os_str))]
    path: PathBuf,
}

fn main() {
    let opt: Opt = Opt::from_args();
    let re_kebab = Regex::new(r"^[a-z][a-z\-]*[a-z]$").unwrap();
    let mut ok = true;

    for result in Walk::new(opt.path).skip(1) {
        let entry = result.unwrap();
        let canonical_path = entry.path()
            .canonicalize()
            .expect("can't create canonical path");

        let path_color = if re_kebab.is_match(canonical_path
            .file_stem()
            .expect("file stem")
            .to_str()
            .expect("file stem to str")
        ) {
            println!("{green}{path}{reset}",
                     path = format!("{}✓ {}",
                                    " ".repeat(entry.depth()),
                                    canonical_path.file_name().unwrap().to_str().unwrap()
                     ),
                     green = color::Fg(color::LightGreen),
                     reset = style::Reset
            );
        } else {
            println!("{bold}{red}{path}{reset}",
                     path = format!("{}✘ {}",
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
