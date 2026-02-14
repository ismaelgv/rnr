use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about, author)]
pub struct Cli {
    #[command(subcommand)]
    pub command: SubCommands,
}

#[derive(Args)]
#[command(flatten_help = true)]
pub struct RegexArgs {
    /// Expression to match (can be a regex).
    pub expression: String,
    /// Expression replacement (use single quotes for capture groups).
    pub replacement: String,

    #[command(flatten)]
    pub common: CommonArgs,
    #[command(flatten)]
    pub replace: ReplaceArgs,
    #[command(flatten)]
    pub path: PathArgs,
}

#[derive(Args)]
pub struct CommonArgs {
    /// Only show what would be done (default mode).
    #[arg(short = 'n', long, conflicts_with = "force")]
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
    /// Set the dump file prefix.
    #[arg(long, conflicts_with = "no_dump", default_value = "rnr-")]
    pub dump_prefix: String,
    /// Do not dump operations into a file.
    #[arg(long)]
    pub no_dump: bool,
}

#[derive(Args)]
pub struct PathArgs {
    /// Target paths.
    #[arg(value_name = "PATH(S)", required = true)]
    pub paths: Vec<String>,
    /// Rename matching directories.
    #[arg(short = 'D', long)]
    pub include_dirs: bool,

    /// Recursive mode.
    #[arg(short, long)]
    pub recursive: bool,
    /// Set max depth in recursive mode.
    #[arg(short = 'd', long, requires = "recursive", value_name = "LEVEL")]
    pub max_depth: Option<usize>,
    /// Include hidden files and directories.
    #[arg(short = 'x', long, requires = "recursive")]
    pub hidden: bool,
}

#[derive(Args)]
pub struct ReplaceArgs {
    /// Limit of replacements, all matches if set to 0.
    #[arg(short = 'l', long, value_name = "LIMIT")]
    pub replace_limit: Option<usize>,
    /// Apply a transformation to replacements including captured groups.
    #[arg(value_enum, short = 't', long)]
    pub replace_transform: Option<ReplaceTransform>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    /// Rename files and directories using a regular expression.
    #[command(arg_required_else_help = true)]
    Regex(RegexArgs),
    /// Read operations from a dump file.
    #[command(arg_required_else_help = true)]
    FromFile {
        #[command(flatten)]
        common: CommonArgs,

        #[arg(value_name = "DUMPFILE")]
        dumpfile: String,
        /// Undo the operations from the dump file.
        #[arg(short, long)]
        undo: bool,
    },
    /// Replace file name UTF-8 chars with ASCII chars representation.
    #[command(arg_required_else_help = true)]
    ToASCII {
        #[command(flatten)]
        common: CommonArgs,

        #[command(flatten)]
        path: PathArgs,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Color {
    Always,
    NoDiff,
    Never,
    Auto,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ReplaceTransform {
    Upper,
    Lower,
    Ascii,
}

#[cfg(test)]
mod test {
    use crate::cli::Cli;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
