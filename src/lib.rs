#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate structopt;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref REGEX_PATTERN: Regex = Regex::new(r"/(.*)/").unwrap();
}

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
    convention_regex: Option<Regex>,
}

struct ValidatedConfig {
    default_convention: Regex,
    patterns: Vec<ValidatedSubConfig>,
}

pub fn filename_of(path: &Path) -> &str {
    path.file_stem()
        .expect("file stem is missing")
        .to_str()
        .expect("can't cast file stem to string")
}

impl Ficon {
    const DEFAULT_CONFIG_FILE: &'static str = "Ficon.toml";

    pub fn new() -> Result<Ficon, Error> {
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
        &self.option.path
    }

    pub fn check(&mut self, path: &Path) -> Result<bool, Error> {
        let convention_regex = self.validated_config.convention_for(path)?;
        let file_name = filename_of(path)
            // ignore multiple extension by default
            // TODO: make this configurable
            .split('.')
            .next()
            .unwrap_or("");

        Ok(convention_regex.is_match(file_name))
    }
}

impl TryFrom<Config> for ValidatedConfig {
    type Error = Error;

    fn try_from(value: Config) -> Result<ValidatedConfig, Error> {
        Ok(ValidatedConfig {
            default_convention: Self::new_regex_for_convention(&value.default.convention)?,
            patterns: match value.for_patterns {
                Some(mut pattern_configs) => pattern_configs
                    .drain(..)
                    .map(|conf| {
                        Pattern::new(&conf.pattern)
                            .with_context(|_| format!("Failed to parse pattern '{}'", conf.pattern))
                            .map(|pattern| ValidatedSubConfig {
                                convention: conf.convention,
                                convention_regex: None,
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
    fn new_regex(pattern: &str) -> Result<Regex, Error> {
        Regex::new(pattern)
            .with_context(|_| format!("Invalid convention definition: {}", pattern))
            .map_err(Into::into)
    }

    fn new_regex_for_convention(convention: &str) -> Result<Regex, Error> {
        match convention {
            "any" => Self::new_regex(r".*"),
            "kebab" => Self::new_regex(r"^[a-z][a-z\-\d]*[a-z\d]$"),
            "snake" => Self::new_regex(r"^[a-z][a-z_\d]*[a-z\d]$"),
            "upper_snake" => Self::new_regex(r"^[A-Z][A-Z_\d]*$"),
            "camel" => Self::new_regex(r"^[a-z][A-Za-z\d]*$"),
            "pascal" => Self::new_regex(r"^[A-Z][A-Za-z\d]*$"),
            convention => {
                if REGEX_PATTERN.is_match(convention) {
                    let convention = REGEX_PATTERN.replace(convention, "$1");
                    Regex::new(&convention)
                        .with_context(|_| format!("{} is not a valid regexp", convention))
                        .map_err(Into::into)
                } else {
                    bail!(
                        "convention is not predefined or defined as regexp: {}",
                        convention
                    )
                }
            }
        }
    }

    fn convention_for(&mut self, path: &Path) -> Result<&Regex, Error> {
        match self
            .patterns
            .iter_mut()
            .find(|p| p.pattern.matches_path(path))
        {
            Some(pattern) => {
                let convention = &pattern.convention;
                Ok(get_or_insert_with_error(
                    &mut pattern.convention_regex,
                    || Self::new_regex_for_convention(convention),
                )?)
            }
            None => Ok(&self.default_convention),
        }
    }
}

/// Like Option::get_or_insert_with(), but supports errors
fn get_or_insert_with_error<T, E>(
    input: &mut Option<T>,
    f: impl FnOnce() -> Result<T, E>,
) -> Result<&T, E> {
    Ok(match input {
        Some(ref v) => v,
        None => {
            *input = Some(f()?);
            match input {
                Some(ref v) => v,
                None => unreachable!(),
            }
        }
    })
}
