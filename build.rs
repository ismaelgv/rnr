use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use std::io::Error;

include!("src/cli.rs");

const APP_NAME: &str = "rnr";

fn main() -> Result<(), Error> {
    let outdir =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?);

    let mut cmd = Cli::command();

    // Completion
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, APP_NAME, &outdir)?;
        println!("cargo:warning={shell:?} completion file is generated.");
    }

    // Man
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    let mut man_file = outdir.join(APP_NAME);
    man_file.set_extension("1");
    std::fs::write(man_file, buffer)?;
    println!("cargo:warning=man file is generated.");

    Ok(())
}
