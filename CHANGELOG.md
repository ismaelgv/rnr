# v0.1.4 (Unreleased)
### Added
* Bash, Fish and Zsh completions.
### Changed
* Now batch renaming stops if a file cannot be renamed. This will avoid some bad
  ordering problems and a possible file overwrite.

# v0.1.3 (2018-08-17)
### Added
* Exclude hidden files and directories by default. Create a new flag to include
  these hidden files.
* New renaming order solver which is more reliable handling conflicting renames. 
* New `solver` module.

# v0.1.2 (2018-08-16)
### Added
* Silent mode.
* Option to set color mode (always, auto, never).
* New tests.

### Changed
* New modules and heavy code reorganization: `output` and `fileutils`