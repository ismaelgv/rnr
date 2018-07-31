# RnR
[![Build Status](https://travis-ci.org/ChuckDaniels87/rnr.svg?branch=master)](https://travis-ci.org/ChuckDaniels87/rnr)
[![Build status](https://ci.appveyor.com/api/projects/status/97e28mxlakxbeqex?svg=true)](https://ci.appveyor.com/project/ChuckDaniels87/rnr)
[![Crates.io](https://img.shields.io/crates/v/rnr.svg)](https://crates.io/crates/rnr)
[![License](https://img.shields.io/crates/l/rnr.svg)](https://github.com/ChuckDaniels87/rnr/blob/master/LICENSE)

`rnr` is a command-line tool to batch rename files and directories.

# Install
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

### Arch Linux
A package is available in the AUR 
([`rnr`](https://aur.archlinux.org/packages/rnr/)) to install latest version of
*RnR* on Arch Linux.

# Options
```
USAGE:
    rnr [FLAGS] [OPTIONS] <EXPRESSION> <REPLACEMENT> <FILE(S)>...

FLAGS:
    -b, --backup          Generate file backups before renaming
    -n, --dry-run         Only show what would be done (default mode)
    -f, --force           Make actual changes to files
    -h, --help            Prints help information
    -x, --hidden          Include hidden files and directories
    -D, --include-dirs    Rename matching directories
    -s, --silent          Do not print any information
    -V, --version         Prints version information

OPTIONS:
        --color <color>        Set color output mode [default: auto]
                               [possible values: always, auto, never]
    -d, --max-depth <LEVEL>    Set max depth in recursive mode
    -r, --recursive <PATH>     Recursive mode

ARGS:
    <EXPRESSION>     Expression to match (can be a regex)
    <REPLACEMENT>    Expression replacement
    <FILE(S)>...     Target files
```

## Default behavior
* *Dry-run* by default.
* Only **UTF-8 valid** input arguments and filenames.
* Works on files and symlinks (ignores directories).
* Accepts multiple files as arguments.
* Accepts a **regex** to generate matches. These expressions have same
  limitations of `regex` crate. It supports *capture groups*.
* If max depth is not provided to recursive mode, it is assumed *infinite*.
* Does not generate backups.
* Output is *colored* (only ANSI terminals).
* Ignore hidden files and directories.

# Demo
[![Demo](https://cdn.rawgit.com/ChuckDaniels87/b0607fdaa44c6201cde398b6a9e23e4e/raw/59d43365d15c55d9c259edd29292609c06de21f7/rnr-demo.svg)](https://cdn.rawgit.com/ChuckDaniels87/b0607fdaa44c6201cde398b6a9e23e4e/raw/f29d84760f4225dce74bf81052180e12a287b892/rnr-demo.svg)
