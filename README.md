# RnR
[![Build Status](https://travis-ci.org/ChuckDaniels87/rnr.svg?branch=master)](https://travis-ci.org/ChuckDaniels87/rnr) [![Crates.io](https://img.shields.io/crates/v/rnr.svg)](https://crates.io/crates/rnr)
[![License](https://img.shields.io/crates/l/rnr.svg)](https://github.com/ChuckDaniels87/rnr/blob/master/LICENSE)

`rnr` is a command-line tool to batch rename files for ANSI terminals.

# Install
*RnR* is written in Rust. At this moment, you will need Cargo to
build/install this application.

### From source
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
    rnr [FLAGS] [OPTIONS] <EXPRESSION> <REPLACEMENT> <FILE(S)>...

FLAGS:
    -b, --backup     Generate file backups before renaming
    -n, --dry-run    Only show what would be done (default mode)
    -f, --force      Make actual changes to files
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
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
  limitations of `regex` crate.
* If max depth is not provided to recursive mode, it is assumed *infite*.
* Does not generate backups.
* Output is *always colored*. [TODO: *no color mode / silent mode*]

# Screenshots

![screenshot_1](https://user-images.githubusercontent.com/8478202/42589754-5ac244ec-8542-11e8-9b1a-8c0d8d0419bf.png)
![screenshot_2](https://user-images.githubusercontent.com/8478202/42589674-110570f4-8542-11e8-9b10-7ff21b1cd4ce.png)
