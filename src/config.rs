use clap::Parser;
use cli::Cli;
use output::Printer;
use regex::Regex;
use std::{
    io::{self, IsTerminal},
    sync::Arc,
};

use crate::{
    cli::{ReplaceTransform, SubCommands},
    renamer::TextTransformation,
};

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
        transform: TextTransformation,
    },
    ToASCII,
}

struct ArgumentParser<'a> {
    cli: &'a Cli,
    printer: &'a Printer,
}

impl ArgumentParser<'_> {
    fn parse_run_mode(&self) -> Result<RunMode, String> {
        if let Some(SubCommands::FromFile { dumpfile, undo }) = &self.cli.command {
            return Ok(RunMode::FromFile {
                path: dumpfile.clone(),
                undo: undo.unwrap_or(false),
            });
        }

        if self.cli.recursive {
            Ok(RunMode::Recursive {
                paths: self.cli.paths.clone(),
                max_depth: self.cli.max_depth,
                hidden: self.cli.hidden,
            })
        } else {
            Ok(RunMode::Simple(self.cli.paths.clone()))
        }
    }

    fn parse_replace_mode(&self) -> Result<ReplaceMode, String> {
        if let Some(SubCommands::ToASCII) = self.cli.command {
            return Ok(ReplaceMode::ToASCII);
        }

        // Get and validate regex expression and replacement from arguments
        let expression = match Regex::new(&self.cli.expression) {
            Ok(expr) => expr,
            Err(err) => {
                return Err(format!(
                    "{}Bad expression provided\n\n{}",
                    self.printer.colors.error.paint("Error: "),
                    self.printer.colors.error.paint(err.to_string())
                ));
            }
        };

        Ok(ReplaceMode::RegExp {
            expression,
            replacement: self.cli.replacement.clone(),
            limit: self.cli.replace_limit.unwrap_or(1),
            transform: self.cli.replace_transform.into(),
        })
    }
}

/// Parse arguments and do some checking.
fn parse_arguments() -> Result<Config, String> {
    let cli = Cli::parse();

    // Set dump defaults: write in force mode and do not in dry-run unless it is explicitly asked
    let dump = if cli.force { !cli.no_dump } else { cli.dump };

    let printer = if cli.silent {
        Printer::silent()
    } else {
        match cli.color {
            crate::cli::Color::Always => Printer::color(),
            crate::cli::Color::Never => Printer::no_color(),
            crate::cli::Color::Auto => detect_output_color(),
        }
    };

    let argument_parser = ArgumentParser {
        cli: &cli,
        printer: &printer,
    };

    let run_mode = argument_parser.parse_run_mode()?;
    let replace_mode = argument_parser.parse_replace_mode()?;

    Ok(Config {
        force: cli.force,
        backup: cli.backup,
        dirs: cli.include_dirs,
        dump,
        run_mode,
        replace_mode,
        printer,
    })
}

/// Detect if output must be colored and returns a properly configured printer.
fn detect_output_color() -> Printer {
    let stdout = io::stdout();
    if stdout.is_terminal() {
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

impl From<Option<ReplaceTransform>> for TextTransformation {
    fn from(value: Option<ReplaceTransform>) -> Self {
        match value {
            Some(transform) => match transform {
                ReplaceTransform::Upper => TextTransformation::Upper,
                ReplaceTransform::Lower => TextTransformation::Lower,
                ReplaceTransform::ASCII => TextTransformation::ASCII,
            },
            None => TextTransformation::None,
        }
    }
}

#[cfg(test)]
mod test {}
