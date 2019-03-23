#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;
extern crate exitcode;
extern crate regex;
extern crate serde;
extern crate termion;
extern crate toml;

use ignore::Walk;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use termion::{color, style};
use std::ffi::OsStr;

#[derive(StructOpt, Debug)]
#[structopt(name = "ficon")]
struct Opt {
    /// Path to directory to check convention
    #[structopt(name = "PATH", default_value = ".", parse(from_os_str))]
    path: PathBuf,
}

#[derive(Deserialize)]
struct Config {
    default: SubConfig,
    extension: Option<Vec<SubConfigWithExtension>>,
}

#[derive(Deserialize)]
struct SubConfig {
    format: String,
}

#[derive(Deserialize, Debug)]
struct SubConfigWithExtension {
    extension: String,
    format: String,
}


impl Config {
    fn check(&self, path: &PathBuf) -> bool {
        let extensions = &self.extension;

        let vec =vec![];
        let extensions = extensions
            .as_ref()
            .map_or(&vec, |e| e);

        let matched: Vec<&SubConfigWithExtension> = extensions
            .iter()
            .filter(|e| {
                e.extension == path.extension().unwrap_or(OsStr::new("")).to_str().unwrap_or("")
            })
            .collect();

        let format = matched
            .first()
            .map(|e| e.format.clone())
            .unwrap_or(self.default.format.clone());

        let pattern = match format.as_str() {
            "kebab" => Regex::new(r"^[a-z][a-z\-]*[a-z]$").unwrap(),
            "snake" => Regex::new(r"^[a-z][a-z_]*[a-z]$").unwrap(),
            "upper_snake" => Regex::new(r"^[A-Z][A-Z_]*$").unwrap(),
            "camel" => Regex::new(r"^[a-z][A-Za-z]*$").unwrap(),
            "pascal" => Regex::new(r"^[A-Z][A-Za-z]*$").unwrap(),
            // TODO:
            // underscore_pre
            // underscore_post
            // underscore_surround
            _ => panic!("case not found {}", format),
        };

        pattern.is_match(path
            .file_stem()
            .expect("file stem is missing")
            .to_str().expect("can't cast file stem to string"))
    }
}

fn main() {
    let opt: Opt = Opt::from_args();

    let config_path = if opt.path.is_dir() {
        format!("{}/{}", opt.path.display(), "ficon.toml")
    } else {
        panic!("path specified is not a directory")
    };

    let config = fs::read_to_string(&config_path)
        .expect(format!("can't read file from the path specified: {}", config_path).as_str());

    let config: Config = toml::from_str(config.as_str()).unwrap();

    let mut ok = true;

    for result in Walk::new(opt.path).skip(1) {
        let entry = result.unwrap();
        let canonical_path = entry
            .path()
            .canonicalize()
            .expect("can't create canonical path");

        if config.check(&entry.clone().into_path()) {
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
