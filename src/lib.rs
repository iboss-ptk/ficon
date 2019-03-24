#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate structopt;

use glob::Pattern;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use exitfailure::ExitFailure;
use failure::{ResultExt, Context};

#[derive(StructOpt, Debug)]
#[structopt(name = "ficon")]
pub struct CliOption {
    /// Path to directory to check convention
    #[structopt(name = "PATH", default_value = ".", parse(from_os_str))]
    pub path: PathBuf,
}

#[derive(Deserialize)]
pub struct Config {
    default: SubConfig,
    for_patterns: Option<Vec<SubConfigByPattern>>,
}

#[derive(Deserialize)]
struct SubConfig {
    convention: String,
}

#[derive(Deserialize, Debug)]
struct SubConfigByPattern {
    pattern: String,
    convention: String,
}

pub struct Ficon {
    option: CliOption,
    config: Config,
}

impl Ficon {
    const DEFAULT_CONFIG_FILE: &'static str = "Ficon.toml";

    pub fn new() -> Result<Ficon, ExitFailure> {
        let option: CliOption = CliOption::from_args();

        let config_path = if option.path.is_dir() {
            Ok(format!("{}/{}", option.path.display(), Ficon::DEFAULT_CONFIG_FILE))
        } else {
            Err(Context::new(format!("\"{}\" is not a directory", option.path.display())))
        }?;

        let config = fs::read_to_string(&config_path)
            .with_context(|_| format!("can't read file from the path specified: {}", config_path.as_str()))?;

        let config: Config = toml::from_str(config.as_str())
            .with_context(|_| "Error while parsing configuration file")?;

        Ok(Ficon { option, config })
    }

    pub fn target_dir(&self) -> &Path {
        return self.option.path.as_ref();
    }

    pub fn check(&self, path: &Path) -> bool {
        let convention = self.config.convention_for(path);
        let reg_pattern = Regex::new(r"/(.*)/").unwrap();

        let convention = match convention.as_str() {
            "any" => Regex::new(r".*").unwrap(),
            "kebab" => Regex::new(r"^[a-z][a-z\-\d]*[a-z\d]$").unwrap(),
            "snake" => Regex::new(r"^[a-z][a-z_\d]*[a-z\d]$").unwrap(),
            "upper_snake" => Regex::new(r"^[A-Z][A-Z_\d]*$").unwrap(),
            "camel" => Regex::new(r"^[a-z][A-Za-z\d]*$").unwrap(),
            "pascal" => Regex::new(r"^[A-Z][A-Za-z\d]*$").unwrap(),
            convention => {
                if reg_pattern.is_match(convention) {
                    let pattern = reg_pattern.replace(convention, "$1").to_string();
                    Regex::new(pattern.as_str()).unwrap()
                } else {
                    panic!("can not parse convention: {}", convention);
                }
            }
        };

        let file_name = path
            .file_stem()
            .expect("file stem is missing")
            .to_str()
            .expect("can't cast file stem to string");

        // ignore multiple extension by default
        // TODO: make this configurable
        let file_name = file_name.split(".").next().unwrap_or("");

        convention.is_match(file_name)
    }
}

impl Config {
    fn convention_for(&self, path: &Path) -> String {
        let pattern_configs = &self.for_patterns;

        let empty_vec = vec![];
        let pattern_configs = pattern_configs.as_ref().map_or(&empty_vec, |e| e);

        let matched_formats: Vec<&SubConfigByPattern> = pattern_configs
            .iter()
            .filter(|conf| {
                let pattern = Pattern::new(conf.pattern.as_str()).expect("invalid glob pattern");

                pattern.matches_path(path)
            })
            .collect();

        return matched_formats
            .first()
            .map(|e| e.convention.clone())
            .unwrap_or(self.default.convention.clone());
    }
}
