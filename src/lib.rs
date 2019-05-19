#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate structopt;

use exitfailure::ExitFailure;
use failure::{Context, ResultExt};
use glob::Pattern;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

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

    pub fn new() -> Result<Self, ExitFailure> {
        let option: CliOption = CliOption::from_args();

        let config_path = if option.path.is_dir() {
            Ok(format!(
                "{}/{}",
                option.path.display(),
                Self::DEFAULT_CONFIG_FILE
            ))
        } else {
            Err(Context::new(format!(
                "\"{}\" is not a directory",
                option.path.display()
            )))
        }?;

        let config_str = fs::read_to_string(&config_path)
            .with_context(|_| format!("Config file is missing: {}", &config_path))?;

        let config: Config = toml::from_str(&config_str)
            .with_context(|_| "Error while parsing configuration file")?;

        Ok(Self { option, config })
    }

    pub fn target_dir(&self) -> &Path {
        return &self.option.path;
    }

    pub fn check(&self, path: &Path) -> Result<bool, ExitFailure> {
        let file_name = path
            .file_stem()
            .expect("file stem is missing")
            .to_str()
            .expect("can't cast file stem to string");

        // ignore multiple extension by default
        // TODO: make this configurable
        let file_name = file_name.split(".").next().unwrap_or("");

        let reg_pattern = Regex::new(r"/(.*)/").unwrap();

        let convention = match self.config.convention_for(path) {
            "any" => Self::convention_from_regex(r".*"),
            "kebab" => Self::convention_from_regex(r"^[a-z][a-z\-\d]*[a-z\d]$"),
            "snake" => Self::convention_from_regex(r"^[a-z][a-z_\d]*[a-z\d]$"),
            "upper_snake" => Self::convention_from_regex(r"^[A-Z][A-Z_\d]*$"),
            "camel" => Self::convention_from_regex(r"^[a-z][A-Za-z\d]*$"),
            "pascal" => Self::convention_from_regex(r"^[A-Z][A-Za-z\d]*$"),
            convention_str => {
                if reg_pattern.is_match(convention_str) {
                    let convention = reg_pattern.replace(convention_str, "$1");
                    Regex::new(&convention)
                        .with_context(|_| format!("{} is not a valid regexp", convention))
                } else {
                    Err(Context::new(format!(
                        "convention is not predefined or defined as regexp: {}",
                        convention_str
                    )))
                }
            }
        }?;

        Ok(convention.is_match(file_name))
    }

    fn convention_from_regex(pattern: &str) -> Result<Regex, Context<String>> {
        Regex::new(pattern).with_context(|_| format!("Invalid convention definition: {}", pattern))
    }
}

impl Config {
    fn convention_for(&self, path: &Path) -> &str {
        self.for_patterns
            .as_ref()
            .map_or(self.default.convention.as_str(), |configs| {
                configs
                    .iter()
                    .filter(|conf| {
                        let pattern = Pattern::new(&conf.pattern).expect("invalid glob pattern");

                        pattern.matches_path(path)
                    })
                    .collect::<Vec<_>>()
                    .first()
                    .map(|e| e.convention.as_str())
                    .unwrap_or(self.default.convention.as_str())
            })
    }
}
