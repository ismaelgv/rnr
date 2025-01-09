use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about, author)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<SubCommands>,

    /// Expression to match (can be a regex).
    pub expression: String,
    /// Expression replacement (use single quotes for capture groups).
    pub replacement: String,

    // NOTE: COMMON
    /// Only show what would be done (default mode).
    #[arg(short = 'n', long = "dry-run", conflicts_with = "force")]
    pub dry_run: bool,
    /// Make actual changes to files.
    #[arg(short, long)]
    pub force: bool,
    /// Generate file backups before renaming.
    #[arg(short, long)]
    pub backup: bool,

    /// Do not print any information.
    #[arg(short, long)]
    pub silent: bool,
    /// Set color output mode.
    #[arg(value_enum, long, default_value_t = Color::Auto)]
    pub color: Color,

    /// Force dumping operations into a file even in dry-run mode.
    #[arg(long, conflicts_with = "no_dump")]
    pub dump: bool,
    /// Do not dump operations into a file.
    #[arg(long = "no-dump")]
    pub no_dump: bool,

    // NOTE: PATH ARGS
    /// Target paths.
    #[arg(value_name = "PATH(S)")]
    pub paths: Vec<String>,
    /// Rename matching directories.
    #[arg(short = 'D', long = "include-dirs")]
    pub include_dirs: bool,

    /// Recursive mode.
    #[arg(short, long)]
    pub recursive: bool,
    /// Set max depth in recursive mode.
    #[arg(
        short = 'd',
        long = "max-depth",
        requires = "recursive",
        value_name = "LEVEL"
    )]
    pub max_depth: Option<usize>,
    /// Include hidden files and directories.
    #[arg(short = 'x', long, requires = "recursive")]
    pub hidden: bool,

    // NOTE: REPLACE ARGS
    /// Limit of replacements, all matches if set to 0.
    #[arg(short = 'l', long = "replace-limit", value_name = "LIMIT")]
    pub replace_limit: Option<usize>,
    /// Apply a transformation to replacements including captured groups.
    #[arg(value_enum, short = 't', long = "replace-transform")]
    pub replace_transform: Option<ReplaceTransform>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    /// Read operations from a dump file.
    FromFile {
        #[arg(value_name = "DUMPFILE")]
        dumpfile: String,
        /// Undo the operations from the dump file.
        #[arg(short, long)]
        undo: Option<bool>,
    },
    /// Replace file name UTF-8 chars with ASCII chars representation.
    ToASCII,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Color {
    Always,
    Never,
    Auto,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ReplaceTransform {
    Upper,
    Lower,
    ASCII,
}
