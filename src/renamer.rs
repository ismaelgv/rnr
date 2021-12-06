use any_ascii::any_ascii;
use config::{Config, ReplaceMode, RunMode};
use dumpfile;
use error::*;
use fileutils::{cleanup_paths, create_backup, get_paths};
use solver;
use solver::{Operation, Operations, RenameMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Renamer {
    config: Arc<Config>,
}

impl Renamer {
    pub fn new(config: &Arc<Config>) -> Result<Renamer> {
        Ok(Renamer {
            config: config.clone(),
        })
    }

    /// Process path batch
    pub fn process(&self) -> Result<Operations> {
        let operations = match self.config.run_mode {
            RunMode::Simple(_) | RunMode::Recursive { .. } => {
                // Get paths
                let mut input_paths = get_paths(&self.config.run_mode);

                // Remove directories and on existing paths from the list
                cleanup_paths(&mut input_paths, self.config.dirs);

                // Relate original names with their targets
                let rename_map = self.get_rename_map(&input_paths)?;

                // Solve renaming operation ordering to avoid conflicts
                solver::solve_rename_order(&rename_map)?
            }
            RunMode::FromFile { ref path, undo } => {
                // Read operations from file
                let operations = dumpfile::read_from_file(&PathBuf::from(path))?;
                if undo {
                    solver::revert_operations(&operations)?
                } else {
                    operations
                }
            }
        };

        // Dump operations into a file if required
        if self.config.dump {
            dumpfile::dump_to_file(&operations)?;
        }

        Ok(operations)
    }

    /// Rename an operation batch
    pub fn batch_rename(&self, operations: Operations) -> Result<()> {
        for operation in operations {
            self.rename(&operation)?;
        }
        Ok(())
    }

    /// Replace file name matches in the given path using stored config.
    fn replace_match(&self, path: &Path) -> PathBuf {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let parent = path.parent();

        let target_name = match &self.config.replace_mode {
            ReplaceMode::RegExp {
                expression,
                replacement,
                limit,
            } => expression
                .replacen(file_name, *limit, &replacement[..])
                .to_string(),
            ReplaceMode::ToASCII => any_ascii(file_name),
        };

        match parent {
            None => PathBuf::from(target_name),
            Some(path) => path.join(Path::new(&target_name)),
        }
    }

    /// Get hash map containing all replacements to be done
    fn get_rename_map(&self, paths: &[PathBuf]) -> Result<RenameMap> {
        let printer = &self.config.printer;
        let colors = &printer.colors;

        let mut rename_map = RenameMap::new();
        let mut error_string = String::new();

        for path in paths {
            let target = self.replace_match(path);
            // Discard paths with no changes
            if target != *path {
                if let Some(old_path) = rename_map.insert(target.clone(), path.clone()) {
                    // Targets cannot be duplicated by any reason
                    error_string.push_str(
                        &colors
                            .error
                            .paint(format!(
                                "\n{0}->{2}\n{1}->{2}\n",
                                old_path.display(),
                                path.display(),
                                target.display()
                            ))
                            .to_string(),
                    );
                }
            }
        }
        if error_string.is_empty() {
            Ok(rename_map)
        } else {
            Err(Error {
                kind: ErrorKind::SameFilename,
                value: Some(error_string),
            })
        }
    }

    /// Rename path in the filesystem or simply print renaming information. Checks if target
    /// filename exists before renaming.
    fn rename(&self, operation: &Operation) -> Result<()> {
        let printer = &self.config.printer;
        let colors = &printer.colors;

        if self.config.force {
            // Create a backup before actual renaming
            if self.config.backup {
                match create_backup(&operation.source) {
                    Ok(backup) => printer.print(&format!(
                        "{} Backup created - {}",
                        colors.info.paint("Info: "),
                        colors.source.paint(format!(
                            "{} -> {}",
                            operation.source.display(),
                            backup.display()
                        ))
                    )),
                    Err(err) => {
                        return Err(err);
                    }
                }
            }

            // Rename paths in the filesystem
            if let Err(err) = fs::rename(&operation.source, &operation.target) {
                return Err(Error {
                    kind: ErrorKind::Rename,
                    value: Some(format!(
                        "{} -> {}\n{}",
                        operation.source.display(),
                        operation.target.display(),
                        err
                    )),
                });
            } else {
                printer.print_operation(&operation.source, &operation.target);
            }
        } else {
            // Just print info in dry-run mode
            printer.print_operation(&operation.source, &operation.target);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    extern crate tempfile;
    use super::*;
    use config::RunMode;
    use output::Printer;
    use regex::Regex;
    use std::fs;
    use std::path::Path;
    use std::process;
    use std::sync::Arc;

    #[test]
    fn renamer() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        // Generate a mock directory tree and files
        //
        // - temp_path
        //     |
        //     - test_file_1.txt
        //     |
        //     - test_file_2.txt
        //     |
        //     - mock_dir
        //         |
        //         - test_file_1.txt
        //         |
        //         - test_file_2.txt
        //
        let mock_dir = format!("{}/mock_dir", temp_path);
        let mock_files: Vec<String> = vec![
            format!("{}/test_file_1.txt", temp_path),
            format!("{}/test_file_2.txt", temp_path),
            format!("{}/test_file_1.txt", mock_dir),
            format!("{}/test_file_2.txt", mock_dir),
        ];

        // Create directory tree and files in the filesystem
        fs::create_dir(&mock_dir).expect("Error creating mock directory...");
        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        // Create config
        let mock_config = Arc::new(Config {
            force: true,
            backup: true,
            dirs: false,
            dump: false,
            run_mode: RunMode::Simple(mock_files),
            replace_mode: ReplaceMode::RegExp {
                expression: Regex::new("test").unwrap(),
                replacement: "passed".to_string(),
                limit: 1,
            },
            printer: Printer::color(),
        });

        // Run renamer
        let renamer = match Renamer::new(&mock_config) {
            Ok(renamer) => renamer,
            Err(err) => {
                mock_config.printer.print_error(&err);
                process::exit(1);
            }
        };
        let operations = match renamer.process() {
            Ok(operations) => operations,
            Err(err) => {
                mock_config.printer.print_error(&err);
                process::exit(1);
            }
        };
        if let Err(err) = renamer.batch_rename(operations) {
            mock_config.printer.print_error(&err);
            process::exit(1);
        }

        // Check renamed files
        assert!(Path::new(&format!("{}/passed_file_1.txt", temp_path)).exists());
        assert!(Path::new(&format!("{}/passed_file_2.txt", temp_path)).exists());
        assert!(Path::new(&format!("{}/passed_file_1.txt", mock_dir)).exists());
        assert!(Path::new(&format!("{}/passed_file_2.txt", mock_dir)).exists());

        // Check backup files
        assert!(Path::new(&format!("{}/test_file_1.txt.bk", temp_path)).exists());
        assert!(Path::new(&format!("{}/test_file_2.txt.bk", temp_path)).exists());
        assert!(Path::new(&format!("{}/test_file_1.txt.bk", mock_dir)).exists());
        assert!(Path::new(&format!("{}/test_file_2.txt.bk", mock_dir)).exists());
    }

    #[test]
    fn replace_limit() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_files: Vec<String> = vec![format!("{}/replace_all_aaaaa.txt", temp_path)];
        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        let mock_config = Arc::new(Config {
            force: true,
            backup: false,
            dirs: false,
            dump: false,
            run_mode: RunMode::Simple(mock_files),
            replace_mode: ReplaceMode::RegExp {
                expression: Regex::new("a").unwrap(),
                replacement: "b".to_string(),
                limit: 0,
            },
            printer: Printer::color(),
        });

        let renamer = match Renamer::new(&mock_config) {
            Ok(renamer) => renamer,
            Err(err) => {
                mock_config.printer.print_error(&err);
                process::exit(1);
            }
        };
        let operations = match renamer.process() {
            Ok(operations) => operations,
            Err(err) => {
                mock_config.printer.print_error(&err);
                process::exit(1);
            }
        };
        if let Err(err) = renamer.batch_rename(operations) {
            mock_config.printer.print_error(&err);
            process::exit(1);
        }

        // Check renamed files
        assert!(Path::new(&format!("{}/replbce_bll_bbbbb.txt", temp_path)).exists());
    }

    #[test]
    fn to_ascii() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_files: Vec<String> = vec![
            format!("{}/ǹön-âścîı-lower.txt", temp_path),
            format!("{}/ǸÖN-ÂŚCÎI-UPPER.txt", temp_path),
        ];
        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        let mock_config = Arc::new(Config {
            force: true,
            backup: false,
            dirs: false,
            dump: false,
            run_mode: RunMode::Simple(mock_files),
            replace_mode: ReplaceMode::ToASCII,
            printer: Printer::color(),
        });

        let renamer = match Renamer::new(&mock_config) {
            Ok(renamer) => renamer,
            Err(err) => {
                mock_config.printer.print_error(&err);
                process::exit(1);
            }
        };
        let operations = match renamer.process() {
            Ok(operations) => operations,
            Err(err) => {
                mock_config.printer.print_error(&err);
                process::exit(1);
            }
        };
        if let Err(err) = renamer.batch_rename(operations) {
            mock_config.printer.print_error(&err);
            process::exit(1);
        }

        // Check renamed files
        assert!(Path::new(&format!("{}/non-ascii-lower.txt", temp_path)).exists());
        assert!(Path::new(&format!("{}/NON-ASCII-UPPER.txt", temp_path)).exists());
    }
}
