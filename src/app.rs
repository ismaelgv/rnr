#![allow(unknown_lints)]
use clap::{App, Arg};
use output::Printer;
use regex::Regex;
use std::ffi::{OsStr, OsString};
use std::process;
use std::sync::Arc;

/// This module is defined Config struct to carry application configuration. This struct is created
/// from the parsed arguments from command-line input using `clap`. Only UTF-8 valid arguments are
/// considered.
pub struct Config {
    pub expression: Regex,
    pub replacement: String,
    pub force: bool,
    pub backup: bool,
    pub mode: RunMode,
    pub printer: Printer,
}

impl Config {
    pub fn new() -> Arc<Config> {
        let config = parse_arguments();
        Arc::new(config)
    }
}

pub enum RunMode {
    FileList(Vec<String>),
    Recursive {
        path: String,
        max_depth: Option<usize>,
    },
}

/// Parse arguments and do some checking.
fn parse_arguments() -> Config {
    let app = config_app();
    let matches = app.get_matches();

    // Set output mode
    let printer = if matches.is_present("silent") {
        Printer::silent()
    } else if matches.value_of("color").unwrap_or("auto") == "never" {
        Printer::no_colored()
    } else {
        Printer::colored()
    };

    // Get and validate regex expression and replacement from arguments
    let expression = match Regex::new(matches.value_of("EXPRESSION").unwrap()) {
        Ok(expr) => expr,
        Err(err) => {
            printer.eprint(&format!(
                "{}Bad expression provided\n\n{}",
                printer.colors.error.paint("Error: "),
                printer.colors.error.paint(err.to_string())
            ));
            process::exit(1);
        }
    };
    let replacement = String::from(matches.value_of("REPLACEMENT").unwrap());

    // Detect normal or recursive mode and set properly set its parameters
    let mode = if matches.is_present("recursive") {
        let path = matches.value_of("recursive").unwrap().to_string();
        let max_depth = if matches.is_present("max-depth") {
            Some(
                matches
                    .value_of("max-depth")
                    .unwrap()
                    .parse::<usize>()
                    .unwrap(),
            )
        } else {
            None
        };
        RunMode::Recursive { path, max_depth }
    } else {
        RunMode::FileList(
            matches
                .values_of("FILE(S)")
                .unwrap()
                .map(String::from)
                .collect(),
        )
    };

    Config {
        expression,
        replacement,
        force: matches.is_present("force"),
        backup: matches.is_present("backup"),
        mode,
        printer,
    }
}

/// Configure application using clap. It sets all options and command-line help.
fn config_app<'a>() -> App<'a, 'a> {
    App::new("rnr")
        .version("0.1.2")
        .author("Ismael González <ismgonval@gmail.com>")
        .about("\nrnr is simple file renamer written in Rust.")
        .arg(
            Arg::with_name("EXPRESSION")
                .help("Expression to match (can be a regex)")
                .required(true)
                .validator_os(is_valid_string)
                .index(1),
        )
        .arg(
            Arg::with_name("REPLACEMENT")
                .help("Expression replacement")
                .required(true)
                .validator_os(is_valid_string)
                .index(2),
        )
        .arg(
            Arg::with_name("FILE(S)")
                .help("Target files")
                .required_unless("recursive")
                .conflicts_with("recursive")
                .validator_os(is_valid_string)
                .multiple(true),
        )
        .arg(
            Arg::with_name("dry-run")
                .long("dry-run")
                .short("n")
                .help("Only show what would be done (default mode)")
                .conflicts_with("force"),
        )
        .arg(
            Arg::with_name("force")
                .long("force")
                .short("f")
                .help("Make actual changes to files"),
        )
        .arg(
            Arg::with_name("backup")
                .long("backup")
                .short("b")
                .help("Generate file backups before renaming"),
        )
        .arg(
            Arg::with_name("recursive")
                .long("recursive")
                .short("r")
                .value_name("PATH")
                .validator_os(is_valid_string)
                .help("Recursive mode"),
        )
        .arg(
            Arg::with_name("max-depth")
                .requires("recursive")
                .long("max-depth")
                .short("d")
                .takes_value(true)
                .value_name("LEVEL")
                .validator(is_integer)
                .help("Set max depth in recursive mode"),
        )
        .arg(
            Arg::with_name("silent")
                .long("silent")
                .short("s")
                .help("Do not print any information"),
        )
        .arg(
            Arg::with_name("color")
                .long("color")
                .possible_values(&["always", "auto", "never"])
                .default_value("auto")
                .help("Set color output mode"),
        )
}

#[allow(clippy)]
/// Check if the input provided is valid unsigned integer
fn is_integer(arg_value: String) -> Result<(), String> {
    match arg_value.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err("Value provided is not an integer".to_string()),
    }
}

/// Check if the input provided is valid UTF-8
fn is_valid_string(os_str: &OsStr) -> Result<(), OsString> {
    match os_str.to_str() {
        Some(_) => Ok(()),
        None => Err(OsString::from("Value provided is not a valid UTF-8 string")),
    }
}

#[cfg(test)]
mod test {}
