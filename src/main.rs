//! # RnR
//! *RnR* is a command-line tool to rename multiple files and directories that supports regex
//! expressions.
//!
extern crate ansi_term;
extern crate any_ascii;
extern crate atty;
extern crate chrono;
extern crate difference;
extern crate path_abs;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate walkdir;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;

use renamer::Renamer;

mod app;
mod config;
mod dumpfile;
mod error;
mod fileutils;
mod output;
mod renamer;
mod solver;

fn main() {
    // Read arguments
    let config = match config::Config::new() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    if !config.force {
        let info = &config.printer.colors.info;
        config
            .printer
            .print(&format!("{}", info.paint("This is a DRY-RUN")));
    }

    // Configure renamer
    let renamer = match Renamer::new(&config) {
        Ok(renamer) => renamer,
        Err(err) => {
            config.printer.print_error(&err);
            std::process::exit(1);
        }
    };

    // Generate operations
    let operations = match renamer.process() {
        Ok(operations) => operations,
        Err(err) => {
            config.printer.print_error(&err);
            std::process::exit(1);
        }
    };

    // Batch rename operations
    if let Err(err) = renamer.batch_rename(operations) {
        config.printer.print_error(&err);
        std::process::exit(1);
    }
}
