#![allow(unknown_lints)]
use ansi_term::Colour::*;
use clap::{App, Arg};
use regex::Regex;
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
    pub recursive: RecursiveMode,
    pub file_args: Option<Vec<String>>,
}

impl Config {
    pub fn new() -> Arc<Config> {
        let config = parse_arguments();
        Arc::new(config)
    }
}

pub struct RecursiveMode {
    pub active: bool,
    pub path: String,
    pub max_depth: Option<usize>,
}

/// Parse arguments and do some checking.
fn parse_arguments() -> Config {
    let app = config_app();
    let matches = app.get_matches();

    // Get and validate regex expression and replacement from arguments
    let expression = match Regex::new(matches.value_of("EXPRESSION").unwrap()) {
        Ok(expr) => expr,
        Err(err) => {
            eprintln!(
                "{}Bad expression provided\n\n{}",
                Red.paint("Error: "),
                Red.paint(err.to_string())
            );
            process::exit(1);
        }
    };
    let replacement = String::from(matches.value_of("REPLACEMENT").unwrap());

    // Detect normal or recursive mode and set properly set its parameters
    let mut file_args: Option<Vec<String>> = None;
    let mut recursive = RecursiveMode {
        active: false,
        path: "".to_string(),
        max_depth: None,
    };
    if matches.is_present("recursive") {
        recursive.active = true;
        recursive.path = matches.value_of("recursive").unwrap().to_string();
        if matches.is_present("max-depth") {
            let max_depth = matches
                .value_of("max-depth")
                .unwrap()
                .parse::<usize>()
                .unwrap();
            recursive.max_depth = Some(max_depth);
        }
    } else {
        file_args = Some(
            matches
                .values_of("FILE(S)")
                .unwrap()
                .map(String::from)
                .collect(),
        )
    }

    Config {
        expression,
        replacement,
        force: matches.is_present("force"),
        backup: matches.is_present("backup"),
        recursive,
        file_args,
    }
}

/// Configure application using clap. It sets all options and command-line help.
fn config_app<'a>() -> App<'a, 'a> {
    App::new("rnr")
        .version("0.1")
        .author("Ismael González <ismgonval@gmail.com>")
        .about("\nrnr is simple file renamer written in Rust.")
        .arg(
            Arg::with_name("EXPRESSION")
                .help("Expression to match (can be a regex)")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("REPLACEMENT")
                .help("Expression replacement")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("FILE(S)")
                .help("Target files")
                .required_unless("recursive")
                .conflicts_with("recursive")
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
                .help("Recursive mode"),
        )
        .arg(
            Arg::with_name("max-depth")
                .requires("recursive")
                .long("max-depth")
                .short("d")
                .value_name("LEVEL")
                .validator(is_integer)
                .help("Set max depth in recursive mode")
                .takes_value(true),
        )
}

#[allow(clippy)]
fn is_integer(arg_value: String) -> Result<(), String> {
    match arg_value.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err("Value provided is not an integer".to_string()),
    }
}

#[cfg(test)]
mod test {}
