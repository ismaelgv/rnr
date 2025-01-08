#[macro_use]
extern crate clap;

extern crate clap_complete;

use clap_complete::Shell;

#[path = "src/cli.rs"]
mod cli;

fn main() {
    let env_dir = std::env::var_os("OUT_DIR");
    let outdir = match env_dir {
        None => {
            println!("No OUT_DIR defined to store completion files.");
            std::process::exit(1);
        }
        Some(outdir) => outdir,
    };
}
