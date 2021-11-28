use app::{create_app, AppCommand};
use atty;
use clap::ArgMatches;
use output::Printer;
use regex::Regex;
use std::sync::Arc;

/// This module is defined Config struct to carry application configuration. This struct is created
/// from the parsed arguments from command-line input using `clap`. Only UTF-8 valid arguments are
/// considered.
pub struct Config {
    pub force: bool,
    pub backup: bool,
    pub dirs: bool,
    pub dump: bool,
    pub run_mode: RunMode,
    pub replace_mode: ReplaceMode,
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
    FromFile {
        path: String,
        undo: bool,
    },
}

pub enum ReplaceMode {
    RegExp {
        expression: Regex,
        replacement: String,
        limit: usize,
    },
    ToASCII,
}

struct ArgumentParser<'a> {
    matches: &'a ArgMatches<'a>,
    printer: &'a Printer,
    command: &'a AppCommand,
}

impl ArgumentParser<'_> {
    fn parse_run_mode(&self) -> Result<RunMode, String> {
        if let AppCommand::FromFile = self.command {
            return Ok(RunMode::FromFile {
                path: String::from(self.matches.value_of("DUMPFILE").unwrap_or_default()),
                undo: self.matches.is_present("undo"),
            });
        }

        // Detect run mode and set parameters accordingly
        let input_paths: Vec<String> = self
            .matches
            .values_of("PATH(S)")
            .unwrap_or_default()
            .map(String::from)
            .collect();

        if self.matches.is_present("recursive") {
            let max_depth = if self.matches.is_present("max-depth") {
                Some(
                    self.matches
                        .value_of("max-depth")
                        .unwrap_or_default()
                        .parse::<usize>()
                        .unwrap_or_default(),
                )
            } else {
                None
            };

            Ok(RunMode::Recursive {
                paths: input_paths,
                max_depth,
                hidden: self.matches.is_present("hidden"),
            })
        } else {
            Ok(RunMode::Simple(input_paths))
        }
    }

    fn parse_replace_mode(&self) -> Result<ReplaceMode, String> {
        if let AppCommand::ToASCII = self.command {
            return Ok(ReplaceMode::ToASCII);
        }

        // Get and validate regex expression and replacement from arguments
        let expression = match Regex::new(self.matches.value_of("EXPRESSION").unwrap_or_default()) {
            Ok(expr) => expr,
            Err(err) => {
                return Err(format!(
                    "{}Bad expression provided\n\n{}",
                    self.printer.colors.error.paint("Error: "),
                    self.printer.colors.error.paint(err.to_string())
                ));
            }
        };
        let replacement = String::from(self.matches.value_of("REPLACEMENT").unwrap_or_default());

        let limit = self
            .matches
            .value_of("replace-limit")
            .unwrap_or_default()
            .parse::<usize>()
            .unwrap_or_default();

        Ok(ReplaceMode::RegExp {
            expression,
            replacement,
            limit,
        })
    }
}

/// Parse arguments and do some checking.
fn parse_arguments() -> Result<Config, String> {
    let app = create_app();
    let matches = app.get_matches();
    let (command, matches) = match matches.subcommand() {
        (name, Some(submatches)) => (AppCommand::from_str(name)?, submatches),
        (_, None) => (AppCommand::Root, &matches), // Always defaults to root if no submatches found.
    };

    // Set dump defaults: write in force mode and do not in dry-run unless it is explicitly asked
    let dump = if matches.is_present("force") {
        !matches.is_present("no-dump")
    } else {
        matches.is_present("dump")
    };

    let printer = if matches.is_present("silent") {
        Printer::silent()
    } else {
        match matches.value_of("color").unwrap_or("auto") {
            "always" => Printer::color(),
            "never" => Printer::no_color(),
            "auto" | _ => detect_output_color(),
        }
    };

    let argument_parser = ArgumentParser {
        printer: &printer,
        matches,
        command: &command,
    };

    let run_mode = argument_parser.parse_run_mode()?;
    let replace_mode = argument_parser.parse_replace_mode()?;

    Ok(Config {
        force: matches.is_present("force"),
        backup: matches.is_present("backup"),
        dirs: matches.is_present("include-dirs"),
        dump,
        run_mode,
        replace_mode,
        printer,
    })
}

/// Detect if output must be colored and returns a properly configured printer.
fn detect_output_color() -> Printer {
    if atty::is(atty::Stream::Stdout) {
        #[cfg(not(windows))]
        {
            Printer::color()
        }
        // Enable color support for Windows 10
        #[cfg(windows)]
        {
            use ansi_term;
            match ansi_term::enable_ansi_support() {
                Ok(_) => Printer::color(),
                Err(_) => Printer::no_color(),
            }
        }
    } else {
        Printer::no_color()
    }
}
