use clap::{App, AppSettings, Arg, SubCommand};
use std::ffi::{OsStr, OsString};

/// From file subcommand name.
pub const FROM_FILE_SUBCOMMAND: &str = "from-file";

/// To ASCII subcommand name.
pub const TO_ASCII_SUBCOMMAND: &str = "to-ascii";

/// Create application using clap. It sets all options and command-line help.
pub fn create_app<'a>() -> App<'a, 'a> {
    // These commons args are shared by all commands.
    let common_args = [
        Arg::with_name("dry-run")
            .long("dry-run")
            .short("n")
            .help("Only show what would be done (default mode)")
            .conflicts_with("force"),
        Arg::with_name("force")
            .long("force")
            .short("f")
            .help("Make actual changes to files"),
        Arg::with_name("backup")
            .long("backup")
            .short("b")
            .help("Generate file backups before renaming"),
        Arg::with_name("silent")
            .long("silent")
            .short("s")
            .help("Do not print any information"),
        Arg::with_name("color")
            .long("color")
            .possible_values(&["always", "auto", "never"])
            .default_value("auto")
            .help("Set color output mode"),
        Arg::with_name("dump")
            .long("dump")
            .help("Force dumping operations into a file even in dry-run mode")
            .conflicts_with("no-dump"),
        Arg::with_name("no-dump")
            .long("no-dump")
            .help("Do not dump operations into a file")
            .conflicts_with("dump"),
    ];

    // Path related arguments.
    let path_args = [
        Arg::with_name("PATH(S)")
            .help("Target paths")
            .validator_os(is_valid_string)
            .multiple(true)
            .required(true),
        Arg::with_name("include-dirs")
            .long("include-dirs")
            .short("D")
            .group("TEST")
            .help("Rename matching directories"),
        Arg::with_name("recursive")
            .long("recursive")
            .short("r")
            .help("Recursive mode"),
        Arg::with_name("max-depth")
            .requires("recursive")
            .long("max-depth")
            .short("d")
            .takes_value(true)
            .value_name("LEVEL")
            .validator(is_integer)
            .help("Set max depth in recursive mode"),
        Arg::with_name("hidden")
            .requires("recursive")
            .long("hidden")
            .short("x")
            .help("Include hidden files and directories"),
    ];

    App::new(crate_name!())
        .setting(AppSettings::SubcommandsNegateReqs)
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("EXPRESSION")
                .help("Expression to match (can be a regex)")
                .required(true)
                .validator_os(is_valid_string)
                .index(1),
        )
        .arg(
            Arg::with_name("REPLACEMENT")
                .help("Expression replacement")
                .required(true)
                .validator_os(is_valid_string)
                .index(2),
        )
        .arg(
            Arg::with_name("replace-limit")
                .long("replace-limit")
                .short("l")
                .takes_value(true)
                .value_name("LIMIT")
                .default_value("1")
                .validator(is_integer)
                .help("Limit of replacements, all matches if set to 0"),
        )
        .args(&common_args)
        .args(&path_args)
        .subcommand(
            SubCommand::with_name(FROM_FILE_SUBCOMMAND)
                .args(&common_args)
                .arg(
                    Arg::with_name("DUMPFILE")
                        .takes_value(true)
                        .required(true)
                        .value_name("DUMPFILE")
                        .validator_os(is_valid_string)
                        .index(1),
                )
                .arg(
                    Arg::with_name("undo")
                        .long("undo")
                        .short("u")
                        .help("Undo the operations from the dump file"),
                )
                .about("Read operations from a dump file"),
        )
        .subcommand(
            SubCommand::with_name(TO_ASCII_SUBCOMMAND)
                .args(&common_args)
                .args(&path_args)
                .about("Replace file name UTF-8 chars with ASCII chars representation."),
        )
}

/// Check if the input provided is valid unsigned integer
fn is_integer(arg_value: String) -> Result<(), String> {
    match arg_value.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err("Value provided is not an integer".to_string()),
    }
}

/// Check if the input provided is valid UTF-8
fn is_valid_string(os_str: &OsStr) -> Result<(), OsString> {
    match os_str.to_str() {
        Some(_) => Ok(()),
        None => Err(OsString::from("Value provided is not a valid UTF-8 string")),
    }
}
