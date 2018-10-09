use app::create_app;
use atty;
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
    pub dirs: bool,
    pub dump: bool,
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
    Simple(Vec<String>),
    Recursive {
        paths: Vec<String>,
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
    } else {
        match matches.value_of("color").unwrap_or("auto") {
            "always" => Printer::colored(),
            "never" => Printer::no_colored(),
            "auto" | _ => detect_output_color(),
        }
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
    let input_paths: Vec<String> = matches
        .values_of("PATH(S)")
        .unwrap()
        .map(String::from)
        .collect();

    let mode = if matches.is_present("recursive") {
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
            paths: input_paths,
            max_depth,
            hidden: matches.is_present("hidden"),
        }
    } else {
        RunMode::Simple( input_paths )
    };

    // Set dump defaults: write in force mode and do not in dry-run unless it is explicitly asked
    let dump = if matches.is_present("force") {
        !matches.is_present("no-dump")
    } else {
        matches.is_present("dump")
    };

    Ok(Config {
        expression,
        replacement,
        force: matches.is_present("force"),
        backup: matches.is_present("backup"),
        dirs: matches.is_present("include-dirs"),
        dump,
        mode,
        printer,
    })
}

/// Detect if output must be colored and returns a properly configured printer.
fn detect_output_color() -> Printer {
    if atty::is(atty::Stream::Stdout) {
        #[cfg(not(windows))]
        {
            Printer::colored()
        }
        // Enable color support for Windows 10
        #[cfg(windows)]
        {
            use ansi_term;
            match ansi_term::enable_ansi_support() {
                Ok(_) => Printer::colored(),
                Err(_) => Printer::no_colored(),
            }
        }
    } else {
        Printer::no_colored()
    }
}
