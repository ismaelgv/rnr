use app::Config;
use fileutils::{cleanup_files, create_backup, get_files};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;

type RenameMap = HashMap<String, String>;

pub struct Renamer {
    files: Vec<String>,
    config: Arc<Config>,
}

impl Renamer {
    pub fn new(config: &Arc<Config>) -> Renamer {
        let input_files = get_files(&config);
        Renamer {
            files: input_files,
            config: config.clone(),
        }
    }

    /// Process file batch
    pub fn process(&mut self) {
        let printer = &self.config.printer;
        let colors = &printer.colors;

        // Remove directories and on existing files from the list
        cleanup_files(&mut self.files);

        // Relate original names with their targets
        let rename_map = match self.get_rename_map() {
            Ok(rename_map) => rename_map,
            Err(_) => process::exit(1),
        };

        // Return existing targets in the list of original filenames
        let existing_targets = match existing_targets(&rename_map) {
            Ok(existing_targets) => existing_targets,
            Err(err) => {
                printer.eprint(&format!(
                    "{}Conflict with existing file: {}",
                    colors.error.paint("Error: "),
                    colors.error.paint(err),
                ));
                process::exit(1);
            }
        };

        // Order targets to avoid renaming conflicts
        let ordered_targets = match solve_rename_order(&rename_map, existing_targets) {
            Ok(ordered_targets) => ordered_targets,
            Err(err) => {
                printer.eprint(&format!("{}{}", colors.error.paint("Error: "), err));
                process::exit(1);
            }
        };

        // Execute actual renaming
        for target in &ordered_targets {
            let source = &rename_map[target];
            self.rename(&source, target);
        }
    }

    /// Replace expression match in the given file using stored config.
    fn replace_match(&self, file: &str) -> String {
        let expression = &self.config.expression;
        let replacement = &self.config.replacement;

        let file_name = Path::new(&file).file_name().unwrap().to_str().unwrap();
        let file_path = Path::new(&file).parent().unwrap().to_str().unwrap();

        let target_name = expression.replace(file_name, &replacement[..]);
        match file_path {
            "" => String::from(target_name),
            _ => format!("{}/{}", file_path, target_name),
        }
    }

    /// Get hash map containing all replacements to be done
    fn get_rename_map(&self) -> Result<RenameMap, ()> {
        let printer = &self.config.printer;
        let colors = &printer.colors;

        let mut rename_map = RenameMap::new();
        let mut is_error = false;

        for file in &self.files {
            let target = self.replace_match(&file);
            // Discard files with no changes
            if target != *file {
                if let Some(old_file) = rename_map.insert(target.clone(), file.clone()) {
                    // Targets cannot be duplicated by any reason
                    printer.eprint(&format!(
                        "{}Two files will have the same name\n{}",
                        colors.error.paint("Error: "),
                        colors
                            .error
                            .paint(format!("{0}->{2}\n{1}->{2}", old_file, file, target))
                    ));
                    is_error = true;
                }
            }
        }

        if !is_error {
            Ok(rename_map)
        } else {
            Err(())
        }
    }

    /// Rename file in the filesystem or simply print renaming information. Checks if target
    /// filename exists before renaming.
    fn rename(&self, source: &str, target: &str) {
        let printer = &self.config.printer;
        let colors = &printer.colors;

        if self.config.force {
            // Create a backup before actual renaming
            if self.config.backup {
                match create_backup(source) {
                    Ok(backup) => printer.print(&format!(
                        "{} Backup created - {}",
                        colors.info.paint("Info: "),
                        colors.source.paint(format!("{} -> {}", source, backup))
                    )),
                    Err(_) => {
                        printer.eprint(&format!(
                            "{}File backup failed - {}",
                            colors.error.paint("Error: "),
                            colors.error.paint(source)
                        ));
                        process::exit(1);
                    }
                }
            }

            // Rename files in the filesystem
            if fs::rename(&source, &target).is_err() {
                printer.eprint(&format!(
                    "{}File {} renaming failed.",
                    colors.error.paint("Error: "),
                    colors.error.paint(source)
                ));
            } else {
                printer.print(&format!(
                    "{} -> {}",
                    colors.source.paint(source),
                    colors.target.paint(target)
                ));
            }
        } else {
            // Just print info in dry-run mode
            printer.print(&format!(
                "{} -> {}",
                colors.source.paint(source),
                colors.target.paint(target)
            ));
        }
    }
}

/// Check if targets exists in the filesystem. If they exist, these targets must be contained in
/// the original file list for the renaming problem to be solvable.
fn existing_targets(rename_map: &RenameMap) -> Result<Vec<String>, String> {
    let mut existing_targets: Vec<String> = Vec::new();
    let sources: Vec<String> = rename_map.values().cloned().collect();

    for (target, source) in rename_map {
        if Path::new(&target).exists() {
            if !sources.contains(&target) {
                return Err(format!("{}->{}", source, target));
            } else {
                existing_targets.push(target.clone());
            }
        }
    }
    Ok(existing_targets)
}

/// Solve renaming order to avoid file overwrite. Solver will order existing targets to avoid
/// conflicts and adds remaining targets to the list.
fn solve_rename_order(
    rename_map: &RenameMap,
    mut existing_targets: Vec<String>,
) -> Result<Vec<String>, String> {
    // Store first all non conflicting entries
    let mut ordered_targets: Vec<String> = rename_map
        .iter()
        .filter_map(|(target, _)| {
            if !existing_targets.contains(&target) {
                Some(target.clone())
            } else {
                None
            }
        })
        .collect();

    // Process the container with existing targets until it is empty. The algorithm extracts
    // recursively all targets that are not present in a container with sources exclusively related
    // to current existing targets.
    while !existing_targets.is_empty() {
        // Track selected index to extract value
        let mut selected_index: Option<usize> = None;
        // Create a vector with all sources from existing targets
        let sources: Vec<String> = existing_targets
            .iter()
            .map(|x| rename_map.get(x).cloned().unwrap())
            .collect();
        // Select targets without conflicts in sources
        for (index, target) in existing_targets.iter().enumerate() {
            if sources.contains(&target) {
                continue;
            } else {
                selected_index = Some(index);
                break;
            }
        }

        // Store result in ordered targets container or fail to stop the loop
        match selected_index {
            Some(index) => ordered_targets.push(existing_targets.swap_remove(index)),
            // This will avoid infite while loop if order is not solved
            None => return Err("Cannot solve sorting problem.".to_string()),
        }
    }

    // Store ordered conflicting entries
    ordered_targets.append(&mut existing_targets);

    Ok(ordered_targets)
}

#[cfg(test)]
mod test {
    extern crate tempfile;
    use super::*;
    use app::RunMode;
    use output::Printer;
    use regex::Regex;
    use std::fs;
    use std::path::Path;
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
        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: true,
            backup: true,
            mode: RunMode::FileList(mock_files),
            printer: Printer::colored(),
        };

        // Run renamer
        let mut renamer = Renamer::new(&Arc::new(mock_config));
        renamer.process();

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
}
