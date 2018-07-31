# v0.1.6 (Unreleased)
### Added
* Option to include directories in the renaming process.
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
