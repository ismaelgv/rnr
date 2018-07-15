use ansi_term::Colour::*;
use args::Config;
use args::RunMode;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;
use walkdir::WalkDir;

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

/// Return a list of files for the given configuration.
fn get_files(config: &Config) -> Vec<String> {
    match &config.mode {
        RunMode::Recursive { path, max_depth } => {
            // Get recursive list of files walking directories
            let walkdir = match max_depth {
                Some(max_depth) => WalkDir::new(path).max_depth(*max_depth),
                None => WalkDir::new(path),
            };
            walkdir
                .into_iter()
                .filter_map(|e| e.ok())
                .filter_map(|x| match x.path().to_str() {
                    Some(s) => Some(s.to_string()),
                    None => {
                        eprintln!(
                            "{}File '{}' contains invalid characters",
                            Yellow.paint("Warn: "),
                            Yellow.paint(x.path().to_string_lossy())
                        );
                        None
                    }
                })
                .collect()
        }
        RunMode::FileList(file_list) => file_list.clone(),
    }
}

/// Generate a non-existing name adding numbers to the end of the file. It also supports adding a
/// suffix to the original name.
fn get_unique_filename(file: &str, suffix: &str) -> String {
    let base_name = format!("{}{}", file, suffix);
    let mut unique_name = base_name.clone();
    let mut index = 0;

    while Path::new(&unique_name).exists() {
        index += 1;
        unique_name = format!("{}.{}", base_name, index);
    }

    unique_name.to_string()
}

/// Create a backup of the file
fn create_backup(file: &str) {
    let backup = get_unique_filename(file, ".bk");
    println!(
        "{} Creating a backup - {}",
        Blue.paint("Info: "),
        Blue.paint(format!("{} -> {}", file, backup))
    );

    if fs::copy(file, backup).is_err() {
        eprintln!("{}File backup failed.", Red.paint("Error: "));
        process::exit(1);
    }
}

#[cfg(test)]
mod test {
    extern crate tempfile;
    use super::*;
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

    #[test]
    fn backup_test() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_files: Vec<String> = vec![
            format!("{}/test_file_1.txt", temp_path),
            format!("{}/test_file_2.txt", temp_path),
            format!("{}/test_file_3.txt", temp_path),
        ];

        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
            create_backup(&file);
        }

        assert!(Path::new(&format!("{}/test_file_1.txt.bk", temp_path)).exists());
        assert!(Path::new(&format!("{}/test_file_2.txt.bk", temp_path)).exists());
        assert!(Path::new(&format!("{}/test_file_3.txt.bk", temp_path)).exists());
    }

    #[test]
    fn unique_name_test() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_files: Vec<String> = vec![
            format!("{}/test_file_1", temp_path),
            format!("{}/test_file_1.1", temp_path),
            format!("{}/test_file_1.2", temp_path),
        ];

        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        assert_eq!(
            get_unique_filename(&mock_files[0], ""),
            format!("{}/test_file_1.3", temp_path)
        );
    }

    #[test]
    fn get_files_args_test() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_files: Vec<String> = vec![
            format!("{}/test_file_1.txt", temp_path),
            format!("{}/test_file_2.txt", temp_path),
            format!("{}/test_file_3.txt", temp_path),
        ];

        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: false,
            backup: false,
            mode: RunMode::FileList(mock_files),
        };

        let files = get_files(&mock_config);
        assert!(files.contains(&format!("{}/test_file_1.txt", temp_path)));
        assert!(files.contains(&format!("{}/test_file_2.txt", temp_path)));
        assert!(files.contains(&format!("{}/test_file_3.txt", temp_path)));
    }

    #[test]
    fn get_files_recursive_test() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        // Generate a mock directories tree and files
        //
        // - temp_path
        //     |
        //     - test_file.txt
        //     |
        //     - mock_dir_1
        //         |
        //         - test_file.txt
        //         |
        //         - mock_dir_2
        //             |
        //             - test_file.txt
        //             |
        //             - mock_dir_3
        //                 |
        //                 - test_file.txt
        //
        let mock_dirs: Vec<String> = vec![
            format!("{}/mock_dir_1", temp_path),
            format!("{}/mock_dir_1/mock_dir_2", temp_path),
            format!("{}/mock_dir_1/mock_dir_2/mock_dir_3", temp_path),
        ];
        let mock_files: Vec<String> = vec![
            format!("{}/test_file.txt", temp_path),
            format!("{}/test_file.txt", mock_dirs[0]),
            format!("{}/test_file.txt", mock_dirs[1]),
            format!("{}/test_file.txt", mock_dirs[2]),
        ];

        // Create directory tree and files in the filesystem
        for mock_dir in &mock_dirs {
            fs::create_dir(&mock_dir).expect("Error creating mock directory...");
        }
        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        // Create config with recursive search WITH max depth
        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: false,
            backup: false,
            mode: RunMode::Recursive {
                path: temp_path.to_string(),
                max_depth: Some(2),
            },
        };

        let files = get_files(&mock_config);
        // Must contain these files
        assert!(files.contains(&format!("{}/test_file.txt", temp_path)));
        assert!(files.contains(&format!("{}/mock_dir_1/test_file.txt", temp_path)));
        // Must NOT contain these files
        assert!(!files.contains(&format!(
            "{}/mock_dir_1/mock_dir_2/test_file.txt",
            temp_path
        )));
        assert!(!files.contains(&format!(
            "{}/mock_dir_1/mock_dir_2/mock_dir_3/test_file.txt",
            temp_path
        )));

        // Create config with recursive search WITHOUT max depth
        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: false,
            backup: false,
            mode: RunMode::Recursive {
                path: temp_path.to_string(),
                max_depth: None,
            },
        };

        let files = get_files(&mock_config);
        // Must contain all the files
        assert!(files.contains(&format!("{}/test_file.txt", temp_path)));
        assert!(files.contains(&format!("{}/mock_dir_1/test_file.txt", temp_path)));
        assert!(files.contains(&format!(
            "{}/mock_dir_1/mock_dir_2/test_file.txt",
            temp_path
        )));
        assert!(files.contains(&format!(
            "{}/mock_dir_1/mock_dir_2/mock_dir_3/test_file.txt",
            temp_path
        )));
    }
}
