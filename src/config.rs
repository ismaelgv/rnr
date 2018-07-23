use app::create_app;
use output::Printer;
use regex::Regex;
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
    pub fn new() -> Result<Arc<Config>, String> {
        let config = match parse_arguments() {
            Ok(config) => config,
            Err(err) => return Err(err),
        };
        Ok(Arc::new(config))
    }
}

pub enum RunMode {
    FileList(Vec<String>),
    Recursive {
        path: String,
        max_depth: Option<usize>,
        hidden: bool,
    },
}

/// Parse arguments and do some checking.
fn parse_arguments() -> Result<Config, String> {
    let app = create_app();
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
            return Err(format!(
                "{}Bad expression provided\n\n{}",
                printer.colors.error.paint("Error: "),
                printer.colors.error.paint(err.to_string())
            ));
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
        RunMode::Recursive {
            path,
            max_depth,
            hidden: matches.is_present("hidden"),
        }
    } else {
        RunMode::FileList(
            matches
                .values_of("FILE(S)")
                .unwrap()
                .map(String::from)
                .collect(),
        )
    };

    Ok(Config {
        expression,
        replacement,
        force: matches.is_present("force"),
        backup: matches.is_present("backup"),
        mode,
        printer,
    })
}
