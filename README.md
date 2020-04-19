# RnR
[![Build Status](https://travis-ci.org/ChuckDaniels87/rnr.svg?branch=master)](https://travis-ci.org/ChuckDaniels87/rnr)
[![Build status](https://ci.appveyor.com/api/projects/status/97e28mxlakxbeqex/branch/master?svg=true)](https://ci.appveyor.com/project/ChuckDaniels87/rnr/branch/master)
[![Crates.io](https://img.shields.io/crates/v/rnr.svg)](https://crates.io/crates/rnr)
[![License](https://img.shields.io/crates/l/rnr.svg)](https://github.com/ChuckDaniels87/rnr/blob/master/LICENSE)

*RnR* is a command-line tool to securely rename multiple files and directories that
supports regular expressions.

# Features
* Batch rename files and directories.
* Automated checks to avoid unwanted file collisions, removals or overwrites.
* Use regexp, including capture groups.
* Include directories recursively.
* Create backup files.
* Create and read operations from dump file.
* Undo operations from dump file.
* Exclude/include hidden files.
* Linux, Mac and Windows support, including terminal coloring.
* Extensive unit testing.

# Install

## Binaries

### GitHub Releases
You can download binaries from [latest release
page](https://github.com/ChuckDaniels87/rnr/releases), choose the compressed
file corresponding to your platform. These compressed files contain the
executable and other additional content such as completion files (*Bash*, *Zsh*,
*fish* and *PowerShell*).

### Arch Linux
A package is available in the AUR 
([`rnr`](https://aur.archlinux.org/packages/rnr/)) to install latest version of
*RnR* on Arch Linux.

## From Source
*RnR* is written in Rust. You can build it from source using Cargo.

### From git repository
```sh
git clone https://github.com/ChuckDaniels87/rnr .
cargo install
```
### From Crates.io
```sh
cargo install rnr
```
# Options
```
USAGE:
    rnr [FLAGS] [OPTIONS] <EXPRESSION> <REPLACEMENT> <PATH(S)>...
    rnr [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -b, --backup          Generate file backups before renaming
    -n, --dry-run         Only show what would be done (default mode)
        --dump            Force dumping operations into a file even in dry-run mode
    -f, --force           Make actual changes to files
    -h, --help            Prints help information
    -x, --hidden          Include hidden files and directories
    -D, --include-dirs    Rename matching directories
        --no-dump         Do not dump operations into a file
    -r, --recursive       Recursive mode
    -s, --silent          Do not print any information
    -V, --version         Prints version information

OPTIONS:
        --color <color>        Set color output mode [default: auto]  [possible values: always, auto, never]
    -d, --max-depth <LEVEL>    Set max depth in recursive mode

ARGS:
    <EXPRESSION>     Expression to match (can be a regex)
    <REPLACEMENT>    Expression replacement
    <PATH(S)>...     Target paths

SUBCOMMANDS:
    from-file    Read operations from a dump file
    help         Prints this message or the help of the given subcommand(s)
```

## Default behavior
* Checks all operations to avoid overwriting existing files.
* *Dry-run* by default.
* Only **UTF-8 valid** input arguments and filenames.
* Works on files and symlinks (ignores directories).
* Accepts multiple files as arguments.
* Accepts a **regex** to generate matches. These expressions have same
  limitations of `regex` crate. You can check regex syntax
  [here](https://docs.rs/regex/#syntax). It supports numbered and named *capture
  groups*.
* If max depth is not provided to recursive mode, it is assumed *infinite*.
* Does not generate backups.
* Output is *colored* (only ANSI terminals).
* Ignore hidden files and directories.
* Dump all operations into a file in force mode. This dump file can be used to
  undo these operations from `from-file` subcommand.

# Demo
[![Demo](https://cdn.rawgit.com/ChuckDaniels87/b0607fdaa44c6201cde398b6a9e23e4e/raw/59d43365d15c55d9c259edd29292609c06de21f7/rnr-demo.svg)](https://cdn.rawgit.com/ChuckDaniels87/b0607fdaa44c6201cde398b6a9e23e4e/raw/f29d84760f4225dce74bf81052180e12a287b892/rnr-demo.svg)
