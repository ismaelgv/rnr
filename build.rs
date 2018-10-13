#[macro_use]
extern crate clap;

use clap::Shell;
use std::fs;

#[path = "src/app.rs"]
mod app;

fn main() {
    let env_dir = std::env::var_os("OUT_DIR");
    let outdir = match env_dir {
        None => {
            println!("No OUT_DIR defined to store completion files.");
            std::process::exit(1);
        }
        Some(outdir) => outdir,
    };
    fs::create_dir_all(&outdir).unwrap();

    let mut app = app::create_app();
    app.gen_completions("rnr", Shell::Bash, &outdir);
    app.gen_completions("rnr", Shell::Zsh, &outdir);
    app.gen_completions("rnr", Shell::Fish, &outdir);
    app.gen_completions("rnr", Shell::PowerShell, &outdir);
}
