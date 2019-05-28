#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate structopt;

use failure::{Context, Error, ResultExt};
use glob::Pattern;
use regex::Regex;
use std::{
    fs,
    path::{Path, PathBuf},
};
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

    pub fn new() -> Result<Ficon, Error> {
        let option: CliOption = CliOption::from_args();

        let config_path = if option.path.is_dir() {
            Ok(format!(
                "{}/{}",
                option.path.display(),
                Ficon::DEFAULT_CONFIG_FILE
            ))
        } else {
            Err(Context::new(format!(
                "\"{}\" is not a directory",
                option.path.display()
            )))
        }?;

        let config = fs::read_to_string(&config_path)
            .with_context(|_| format!("Config file is missing: {}", &config_path))?;

        let config: Config =
            toml::from_str(&config).with_context(|_| "Error while parsing configuration file")?;

        Ok(Ficon { option, config })
    }

    pub fn target_dir(&self) -> &Path {
        return self.option.path.as_ref();
    }

    pub fn check(&self, path: &Path) -> Result<bool, Error> {
        let convention_str = self.config.convention_for(path);
        let reg_pattern = Regex::new(r"/(.*)/").unwrap();

        let convention_regex = match convention_str.as_str() {
            "any" => Ficon::convention_from_regex(r".*"),
            "kebab" => Ficon::convention_from_regex(r"^[a-z][a-z\-\d]*[a-z\d]$"),
            "snake" => Ficon::convention_from_regex(r"^[a-z][a-z_\d]*[a-z\d]$"),
            "upper_snake" => Ficon::convention_from_regex(r"^[A-Z][A-Z_\d]*$"),
            "camel" => Ficon::convention_from_regex(r"^[a-z][A-Za-z\d]*$"),
            "pascal" => Ficon::convention_from_regex(r"^[A-Z][A-Za-z\d]*$"),
            convention => {
                if reg_pattern.is_match(convention_str.as_str()) {
                    let convention = reg_pattern.replace(convention, "$1").to_string();
                    Regex::new(convention.as_str())
                        .with_context(|_| format!("{} is not a valid regexp", convention))
                } else {
                    Err(Context::new(format!(
                        "convention is not predefined or defined as regexp: {}",
                        convention
                    )))
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

        let convention = convention_regex.with_context(|_| "fail to parse convention")?;

        Ok(convention.is_match(file_name))
    }

    fn convention_from_regex(pattern: &str) -> Result<Regex, Context<String>> {
        Regex::new(pattern).with_context(|_| format!("Invalid convention definition: {}", pattern))
    }
}

impl Config {
    fn convention_for(&self, path: &Path) -> String {
        match self.for_patterns.as_ref() {
            Some(pattern_configs) => pattern_configs
                .iter()
                .filter(|conf| {
                    Pattern::new(&conf.pattern)
                        .expect("invalid glob pattern")
                        .matches_path(path)
                })
                .next()
                .map(|e| e.convention.clone())
                .unwrap_or(self.default.convention.clone()),
            None => self.default.convention.clone(),
        }
    }
}
