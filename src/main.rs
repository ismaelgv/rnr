//! # RnR
//!
//! `rnr` is a command-line tool to batch rename files for ANSI terminals.
//!
extern crate ansi_term;
extern crate clap;
extern crate regex;
extern crate walkdir;

use ansi_term::Colour::*;
use renamer::Renamer;

mod app;
mod renamer;
mod fileutils;

fn main() {
    // Read arguments
    let config = app::Config::new();
    if !config.force {
        println!("{}", White.bold().paint("This is a DRY-RUN"));
    }

    // Configure renamer
    let mut renamer = Renamer::new(&config);

    // Process files
    renamer.process();
}
