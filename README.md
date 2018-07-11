# Description
`rnr` is a command-line tool to batch rename files for ANSI terminals.

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

## Defaults
* Dry-run by default.
* If max depth is not provided to recursive mode, it is assumed infite.
* No backups.
* Output is always colored. [TODO: no color mode / silent mode]
