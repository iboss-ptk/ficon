#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate structopt;
#[macro_use]
extern crate failure;

use failure::{Context, Error, ResultExt};
use glob::Pattern;
use regex::Regex;
use std::convert::{TryFrom, TryInto};
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
    validated_config: ValidatedConfig,
}

struct ValidatedSubConfig {
    pattern: Pattern,
    convention: String,
}

struct ValidatedConfig {
    default_convention: String,
    for_patterns: Vec<ValidatedSubConfig>,
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
            .with_context(|_| format!("Config file is missing: '{}'", &config_path))?;

        let config: Config = toml::from_str(&config).with_context(|_| {
            format!(
                "Error while parsing configuration file at '{}'",
                config_path
            )
        })?;

        Ok(Ficon {
            option,
            validated_config: <_ as TryInto<ValidatedConfig>>::try_into(config).with_context(
                |_| format!("Validation of configuration at '{}' failed", config_path),
            )?,
        })
    }

    pub fn target_dir(&self) -> &Path {
        return self.option.path.as_ref();
    }

    pub fn check(&self, path: &Path) -> Result<bool, Error> {
        let convention_str = self.validated_config.convention_for(path);
        let convention_regex = Ficon::regex_for_convention(convention_str);

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

    fn regex_for_convention(convention_str: &str) -> Result<Regex, Error> {
        let reg_pattern = Regex::new(r"/(.*)/").unwrap();
        let convention_regex = match convention_str {
            "any" => Ficon::convention_from_regex(r".*"),
            "kebab" => Ficon::convention_from_regex(r"^[a-z][a-z\-\d]*[a-z\d]$"),
            "snake" => Ficon::convention_from_regex(r"^[a-z][a-z_\d]*[a-z\d]$"),
            "upper_snake" => Ficon::convention_from_regex(r"^[A-Z][A-Z_\d]*$"),
            "camel" => Ficon::convention_from_regex(r"^[a-z][A-Za-z\d]*$"),
            "pascal" => Ficon::convention_from_regex(r"^[A-Z][A-Za-z\d]*$"),
            convention => {
                if reg_pattern.is_match(convention_str) {
                    let convention = reg_pattern.replace(convention, "$1").to_string();
                    Regex::new(convention.as_str())
                        .with_context(|_| format!("{} is not a valid regexp", convention))
                        .map_err(Into::into)
                } else {
                    bail!(
                        "convention is not predefined or defined as regexp: {}",
                        convention
                    )
                }
            }
        };
        convention_regex
    }

    fn convention_from_regex(pattern: &str) -> Result<Regex, Error> {
        Regex::new(pattern)
            .with_context(|_| format!("Invalid convention definition: {}", pattern))
            .map_err(Into::into)
    }
}

impl TryFrom<Config> for ValidatedConfig {
    type Error = Error;

    fn try_from(value: Config) -> Result<ValidatedConfig, Error> {
        Ok(ValidatedConfig {
            default_convention: value.default.convention,
            for_patterns: match value.for_patterns {
                Some(mut pattern_configs) => pattern_configs
                    .drain(..)
                    .map(|conf| {
                        Pattern::new(&conf.pattern)
                            .with_context(|_| format!("Failed to parse pattern '{}'", conf.pattern))
                            .map(|pattern| ValidatedSubConfig {
                                convention: conf.convention,
                                pattern,
                            })
                    })
                    .collect::<Result<_, _>>()?,
                None => Vec::default(),
            },
        })
    }
}

impl ValidatedConfig {
    fn convention_for(&self, path: &Path) -> &str {
        self.for_patterns
            .iter()
            .filter(|p| p.pattern.matches_path(path))
            .next()
            .map(|p| &p.convention)
            .unwrap_or_else(|| &self.default_convention)
    }
}
