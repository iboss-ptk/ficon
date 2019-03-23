extern crate termion;
extern crate exitcode;
extern crate regex;

use ignore::Walk;
use termion::{color, style};
use regex::Regex;


fn main() {
    let re_kebab = Regex::new(r"^[a-z][a-z\-]*[a-z]$").unwrap();
    let mut ok = true;

    for result in Walk::new(".") {
        let entry = result.unwrap();
        // check if it's kebab, if kebab then green else red
        let path_color = if re_kebab.is_match(entry
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
        ) {
            println!("{green}{path}{reset}",
                     path = format!("{}✓ {}",
                                    " ".repeat(entry.depth()),
                                    entry.path().file_name().unwrap().to_str().unwrap()
                     ),
                     green = color::Fg(color::LightGreen),
                     reset = style::Reset
            );
        } else {
            println!("{bold}{red}{path}{reset}",
                     path = format!("{}✘ {}",
                                    "  ".repeat(entry.depth()),
                                    entry.path().file_name().unwrap().to_str().unwrap()
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
