use crate::cli::{Cli, RegexArgs};
use crate::output::Printer;
use anyhow::{Result, bail};
use clap::Parser;
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
    pub dump_prefix: String,
    pub run_mode: RunMode,
    pub replace_mode: ReplaceMode,
    pub printer: Printer,
}

impl Config {
    pub fn new() -> Result<Arc<Config>> {
        let config = match parse_arguments() {
            Ok(config) => config,
            Err(err) => bail!("{}", err),
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
    None,
}

struct ArgumentParser<'a> {
    cli: &'a Cli,
    printer: &'a Printer,
}

impl ArgumentParser<'_> {
    fn parse_run_mode(&self) -> Result<RunMode> {
        let path = match &self.cli.command {
            SubCommands::FromFile { dumpfile, undo, .. } => {
                return Ok(RunMode::FromFile {
                    path: dumpfile.clone(),
                    undo: *undo,
                });
            }
            SubCommands::Regex(RegexArgs { path, .. }) => path,
            SubCommands::ToASCII { path, .. } => path,
        };

        if path.recursive {
            Ok(RunMode::Recursive {
                paths: path.paths.clone(),
                max_depth: path.max_depth,
                hidden: path.hidden,
            })
        } else {
            Ok(RunMode::Simple(path.paths.clone()))
        }
    }

    fn parse_replace_mode(&self) -> Result<ReplaceMode> {
        let regex = match &self.cli.command {
            SubCommands::ToASCII { .. } => return Ok(ReplaceMode::ToASCII),
            SubCommands::FromFile { .. } => return Ok(ReplaceMode::None),
            SubCommands::Regex(regex) => regex,
        };

        // Get and validate regex expression and replacement from arguments
        let expression = match Regex::new(&regex.expression) {
            Ok(expr) => expr,
            Err(err) => {
                bail!(
                    "{}Bad expression provided\n\n{}",
                    self.printer.colors.error.paint("Error: "),
                    self.printer.colors.error.paint(err.to_string())
                );
            }
        };

        Ok(ReplaceMode::RegExp {
            expression,
            replacement: regex.replacement.clone(),
            limit: regex.replace.replace_limit.unwrap_or(1),
            transform: regex.replace.replace_transform.into(),
        })
    }
}

/// Parse arguments and do some checking.
fn parse_arguments() -> Result<Config> {
    let cli = Cli::parse();

    let (common, path) = match &cli.command {
        SubCommands::Regex(RegexArgs { common, path, .. }) => (common, Some(path)),
        SubCommands::ToASCII { common, path } => (common, Some(path)),
        SubCommands::FromFile { common, .. } => (common, None),
    };

    // Set dump defaults: write in force mode and do not in dry-run unless it is explicitly asked
    let dump = if common.force {
        !common.no_dump
    } else {
        common.dump
    };

    let printer = if common.silent {
        Printer::silent()
    } else {
        match common.color {
            crate::cli::Color::Always => Printer::color(true),
            crate::cli::Color::NoDiff => Printer::color(false),
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
        force: common.force,
        backup: common.backup,
        dirs: path.is_some_and(|p| p.include_dirs),
        dump,
        dump_prefix: common.dump_prefix.clone(),
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
            Printer::color(true)
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
                ReplaceTransform::Ascii => TextTransformation::Ascii,
            },
            None => TextTransformation::None,
        }
    }
}

#[cfg(test)]
mod test {}
