use ansi_term::Colour::*;
use args::Config;
use fileutils::{create_backup, get_files};
use std::fs;
use std::path::Path;
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
        self.cleanup();

        for file in &self.files {
            let target = self.replace_match(file);
            if target != *file {
                self.rename(file, &target);
            }
        }
    }

    /// Clean files that does not exists, broken links and directories
    fn cleanup(&mut self) {
        self.files.retain(|file| {
            if !Path::new(&file).exists() {
                // Checks if non-existing path is actually a symlink
                match fs::read_link(&file) {
                    Ok(_) => true,
                    Err(_) => {
                        eprintln!(
                            "{}File '{}' is not accessible",
                            Yellow.paint("Warn: "),
                            Yellow.paint(file.as_str())
                        );
                        false
                    }
                }
            } else {
                !fs::metadata(&file).unwrap().is_dir()
            }
        });
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
        if Path::new(&target).exists() {
            eprintln!(
                "{}File already exists - {}",
                Red.paint("Error: "),
                Red.paint(format!("{} -> {}", file, target))
            );
        } else if self.config.force {
            if self.config.backup {
                create_backup(file);
            }

            if fs::rename(&file, &target).is_err() {
                eprintln!(
                    "{}File {} renaming failed.",
                    Red.paint("Error: "),
                    Red.paint(file)
                );
            } else {
                println!("{} -> {}", Blue.paint(file), Green.paint(target),);
            }
        } else {
            println!("{} -> {}", Blue.paint(file), Green.paint(target),);
        }
    }
}

#[cfg(test)]
mod test {
    extern crate tempfile;
    use super::*;
    use args::RunMode;
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
