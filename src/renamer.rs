use app::Config;
use fileutils::{cleanup_files, create_backup, get_files};
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;

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
        cleanup_files(&mut self.files);

        for file in &self.files {
            let target = self.replace_match(file);
            if target != *file {
                self.rename(file, &target);
            }
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

    /// Rename file in the filesystem or simply print renaming information. Checks if target
    /// filename exists before renaming.
    fn rename(&self, file: &str, target: &str) {
        let printer = &self.config.printer;
        let colors = &printer.colors;

        if Path::new(&target).exists() {
            printer.print(&format!(
                "{}File already exists - {}",
                colors.error.paint("Error: "),
                colors.error.paint(format!("{} -> {}", file, target))
            ));
        } else if self.config.force {
            if self.config.backup {
                match create_backup(file) {
                    Ok(backup) => printer.print(&format!(
                        "{} Backup created - {}",
                        colors.info.paint("Info: "),
                        colors.source.paint(format!("{} -> {}", file, backup))
                    )),
                    Err(_) => {
                        printer.eprint(&format!(
                            "{}File backup failed - {}",
                            colors.error.paint("Error: "),
                            colors.error.paint(file)
                        ));
                        process::exit(1);
                    }
                }
            }

            if fs::rename(&file, &target).is_err() {
                printer.eprint(&format!(
                    "{}File {} renaming failed.",
                    colors.error.paint("Error: "),
                    colors.error.paint(file)
                ));
            } else {
                printer.print(&format!(
                    "{} -> {}",
                    colors.source.paint(file),
                    colors.target.paint(target)
                ));
            }
        } else {
            printer.print(&format!(
                "{} -> {}",
                colors.source.paint(file),
                colors.target.paint(target)
            ));
        }
    }
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
    fn renamer_test() {
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
