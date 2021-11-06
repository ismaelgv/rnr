# v0.3.1 (2021-11-06)
### Changed
* Update references from `ChuckDaniels87/rnr` to `ismaelgv/rnr`.
* Migrate full CI to GitHub Actions.

# v0.3.0 (2020-11-21)
### Added
* Support for case-insensitive case-preserving file systems for MacOS and
  Windows.
* Check if the file is actually the same in case of rename conflict.

### Changed
* Include symlinks in existing file checks.
* Include symlinks when generating an unique file name.

# v0.2.4 (2020-08-01)
### Added
* Add diff to operation output.
* Add replace limit option.

### Changed
* Internal refactor of printer.

# v0.2.3 (2020-07-18)
### Changed
* Update README with examples.
* Refactor several parts.
* Update dependencies

# v0.2.2 (2018-10-13)
#### Added
* Dump operations into a file. This functionality can be activated and
  deactivated from command-line. It is activated in force mode by default.
* New subcommand to read operations from a dump file. This subcommand overrides
  requirements from default behavior.
* New undo operation based on the content of the dump file.
* New dependencies: `chrono`, `serde`, `serde_derive` and `serde_json`.

# v0.2.1 (2018-08-23)
### Added
* More info displayed on error messages.
* Symlink test.
### Fixed
* Notable execution speed regression when recursive mode changes were
  introduced.
### Changed
* Heavy rewrite of solver. Now, the execution speed when directories are
  included is several order of magnitude faster. This is more noticeable when a
  large number of directories are processed.

# v0.2.0 (2018-08-10)
### Added
* Recursive mode accept more than one input path.
* New dependency: `path_abs`
### Changed
* Recursive mode now takes the last positional arguments instead of the next
  one.

# v0.1.6 (2018-08-01)
### Added
* Option to include directories in the renaming process.
* Binary files in GitHub Releases.
### Changed
* Heavy internal refactor to use PathBuf instead of String for files.

# v0.1.5 (2018-07-30)
### Added
* Detect output type in color=auto mode.
* Windows support. (Color is only supported in ANSI terminals)
### Changed
* Change source color and default info color.

# v0.1.4 (2018-07-23)
### Added
* Bash, Fish and Zsh completions.
### Changed
* Now batch renaming stops if a file cannot be renamed. This will avoid some bad
  ordering problems and a possible file overwrite.

# v0.1.3 (2018-07-17)
### Added
* Exclude hidden files and directories by default. Create a new flag to include
  these hidden files.
* New renaming order solver which is more reliable handling conflicting renames. 
* New `solver` module.

# v0.1.2 (2018-07-16)
### Added
* Silent mode.
* Option to set color mode (always, auto, never).
* New tests.

### Changed
* New modules and heavy code reorganization: `output` and `fileutils`
