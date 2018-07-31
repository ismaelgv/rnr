use config::{Config, RunMode};
use error::*;
use std::fs;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

pub type PathList = Vec<PathBuf>;

/// Return a list of files for the given configuration.
pub fn get_files(config: &Config) -> PathList {
    match &config.mode {
        RunMode::Recursive {
            path,
            max_depth,
            hidden,
        } => {
            // Detect if is a hidden file or directory, always include given path
            let is_hidden = |f: &DirEntry| -> bool {
                if !hidden && f.depth() > 0 {
                    f.file_name()
                        .to_str()
                        .map(|s| !s.starts_with('.'))
                        .unwrap_or(false)
                } else {
                    true
                }
            };
            // Get recursive list of files walking directories
            let walkdir = match max_depth {
                Some(max_depth) => WalkDir::new(path).max_depth(*max_depth),
                None => WalkDir::new(path),
            };
            walkdir
                .into_iter()
                .filter_entry(is_hidden)
                .filter_map(|e| e.ok())
                .map(|p| p.path().to_path_buf())
                .collect()
        }
        RunMode::FileList(file_list) => file_list.into_iter().map(|f| PathBuf::from(f)).collect(),
    }
}

/// Generate a non-existing name adding numbers to the end of the file. It also supports adding a
/// suffix to the original name.
pub fn get_unique_filename(file: &PathBuf, suffix: &str) -> PathBuf {
    let base_name = format!("{}{}", file.file_name().unwrap().to_string_lossy(), suffix);
    let mut unique_name = file.clone();
    unique_name.set_file_name(&base_name);

    let mut index = 0;
    while unique_name.exists() {
        index += 1;
        unique_name.set_file_name(format!("{}.{}", base_name, index));
    }

    unique_name
}

/// Create a backup of the file
pub fn create_backup(file: &PathBuf) -> Result<PathBuf> {
    let backup = get_unique_filename(file, ".bk");
    match fs::copy(file, &backup) {
        Ok(_) => Ok(backup),
        Err(_) => Err(Error {
            kind: ErrorKind::CreateBackup,
            value: Some(file.to_string_lossy().to_string()),
        }),
    }
}

/// Clean files that does not exists, broken links and directories
pub fn cleanup_files(files: &mut PathList) {
    files.retain(|file| {
        if !file.exists() {
            // Checks if non-existing path is actually a symlink
            fs::read_link(&file).is_ok()
        } else {
            !fs::metadata(&file).unwrap().is_dir()
        }
    });
}

#[cfg(test)]
mod test {
    extern crate tempfile;
    use super::*;
    use output::Printer;
    use regex::Regex;
    use std::fs;

    #[test]
    fn backup() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_files: Vec<PathBuf> = vec![
            [temp_path, "test_file_1.txt"].iter().collect(),
            [temp_path, "test_file_2.txt"].iter().collect(),
            [temp_path, "test_file_3.txt"].iter().collect(),
        ];

        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
            create_backup(&file).expect("Error generating backup file...");
        }

        let backup_files: Vec<PathBuf> = vec![
            [temp_path, "test_file_1.txt.bk"].iter().collect(),
            [temp_path, "test_file_2.txt.bk"].iter().collect(),
            [temp_path, "test_file_3.txt.bk"].iter().collect(),
        ];

        for file in &backup_files {
            println!("{}", file.display());
            assert!(file.exists());
        }
    }

    #[test]
    fn unique_name() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_files: Vec<PathBuf> = vec![
            [temp_path, "test_file_1"].iter().collect(),
            [temp_path, "test_file_1.1"].iter().collect(),
            [temp_path, "test_file_1.2"].iter().collect(),
        ];

        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        let new_file: PathBuf = [temp_path, "test_file_1.3"].iter().collect();
        assert_eq!(get_unique_filename(&mock_files[0], ""), new_file);
    }

    #[test]
    fn get_files_args() {
        let mock_files: Vec<String> = vec![
            "test_file_1.txt".to_string(),
            "test_file_2.txt".to_string(),
            "test_file_3.txt".to_string(),
        ];

        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: false,
            backup: false,
            mode: RunMode::FileList(mock_files),
            printer: Printer::colored(),
        };

        let files = get_files(&mock_config);
        assert!(files.contains(&PathBuf::from("test_file_1.txt")));
        assert!(files.contains(&PathBuf::from("test_file_2.txt")));
        assert!(files.contains(&PathBuf::from("test_file_3.txt")));
    }

    // Generate directory tree and files for recursive tests
    fn generate_recursive_tempdir() -> (tempfile::TempDir, String) {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_string_lossy().to_string().clone();
        // Generate a mock directories tree and files
        //
        // - temp_path
        //     |
        //     - test_file.txt
        //     |
        //     - .hidden_test_file.txt
        //     |
        //     - .hidden_mock_dir
        //     |   |
        //     |   - test_file.txt
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
        //
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mock_dirs: Vec<PathBuf> = vec![
            [&temp_path, ".hidden_mock_dir"].iter().collect(),
            [&temp_path, "mock_dir_1"].iter().collect(),
            [&temp_path, "mock_dir_1", "mock_dir_2"].iter().collect(),
            [&temp_path, "mock_dir_1", "mock_dir_2", "mock_dir_3"].iter().collect(),
        ];
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mock_files: Vec<PathBuf> = vec![
            [&temp_path, "test_file.txt"].iter().collect(),
            [&temp_path, ".hidden_test_file.txt"].iter().collect(),
            [&mock_dirs[0], &PathBuf::from("test_file.txt")].iter().collect(),
            [&mock_dirs[1], &PathBuf::from("test_file.txt")].iter().collect(),
            [&mock_dirs[2], &PathBuf::from("test_file.txt")].iter().collect(),
            [&mock_dirs[3], &PathBuf::from("test_file.txt")].iter().collect(),
        ];

        // Create directory tree and files in the filesystem
        for mock_dir in &mock_dirs {
            fs::create_dir(&mock_dir).expect("Error creating mock directory...");
        }
        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        // Return tempdir data
        (tempdir, temp_path)
    }

    #[test]
    fn get_files_recursive() {
        let (_tempdir, temp_path) = generate_recursive_tempdir();

        // Create config with recursive search WITHOUT max depth
        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: false,
            backup: false,
            mode: RunMode::Recursive {
                path: temp_path.to_string(),
                max_depth: None,
                hidden: false,
            },
            printer: Printer::colored(),
        };

        let files = get_files(&mock_config);
        // Must contain these files
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let listed_files: Vec<PathBuf> = vec![
            [&temp_path, "test_file.txt"].iter().collect(),
            [&temp_path, "mock_dir_1", "test_file.txt"].iter().collect(),
            [&temp_path, "mock_dir_1", "mock_dir_2", "test_file.txt"].iter().collect(),
            [&temp_path, "mock_dir_1", "mock_dir_2", "mock_dir_3", "test_file.txt"]
                .iter().collect(),
        ];
        for file in &listed_files {
            assert!(files.contains(file));
        }
        // Must NOT contain these files
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let non_listed_files: Vec<PathBuf> = vec![
            [&temp_path, ".hidden_test_file.txt"].iter().collect(),
            [&temp_path, ".hidden_mock_dir", "test_file.txt"].iter().collect(),
        ];
        for file in &non_listed_files {
            assert!(!files.contains(file));
        }
    }

    #[test]
    fn get_files_recursive_depth() {
        let (_tempdir, temp_path) = generate_recursive_tempdir();

        // Create config with recursive search WITH max depth
        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: false,
            backup: false,
            mode: RunMode::Recursive {
                path: temp_path.to_string(),
                max_depth: Some(2),
                hidden: false,
            },
            printer: Printer::colored(),
        };

        let files = get_files(&mock_config);
        // Must contain these files
        let listed_files: Vec<PathBuf> = vec![
            [&temp_path, "test_file.txt"].iter().collect(),
            [&temp_path, "mock_dir_1", "test_file.txt"].iter().collect(),
        ];
        for file in &listed_files {
            assert!(files.contains(file));
        }
        // Must NOT contain these files
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let non_listed_files: Vec<PathBuf> = vec![
            [&temp_path, "mock_dir_1", "mock_dir_2", "test_file.txt"].iter().collect(),
            [&temp_path, "mock_dir_1", "mock_dir_2", "mock_dir_3", "test_file.txt"]
                .iter().collect(),
            [&temp_path, ".hidden_test_file.txt"].iter().collect(),
            [&temp_path, ".hidden_mock_dir", "test_file.txt"].iter().collect(),
        ];
        for file in &non_listed_files {
            assert!(!files.contains(file));
        }
    }

    #[test]
    fn get_files_recursive_hidden() {
        let (_tempdir, temp_path) = generate_recursive_tempdir();

        // Create config with recursive search WITHOUT max depth
        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: false,
            backup: false,
            mode: RunMode::Recursive {
                path: temp_path.to_string(),
                max_depth: None,
                hidden: true,
            },
            printer: Printer::colored(),
        };

        let files = get_files(&mock_config);
        // Must contain these files
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let listed_files: Vec<PathBuf> = vec![
            [&temp_path, "test_file.txt"].iter().collect(),
            [&temp_path, "mock_dir_1", "test_file.txt"].iter().collect(),
            [&temp_path, "mock_dir_1", "mock_dir_2", "test_file.txt"].iter().collect(),
            [&temp_path, "mock_dir_1", "mock_dir_2", "mock_dir_3", "test_file.txt"]
                .iter().collect(),
            [&temp_path, ".hidden_test_file.txt"].iter().collect(),
            [&temp_path, ".hidden_mock_dir", "test_file.txt"].iter().collect(),
        ];
        for file in &listed_files {
            assert!(files.contains(file));
        }
    }

    #[test]
    fn cleanup() {
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
        //
        let mock_dirs: Vec<PathBuf> = vec![
            [temp_path, "mock_dir_1"].iter().collect(),
            [temp_path, "mock_dir_1", "mock_dir_2"].iter().collect(),
        ];
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mut mock_files: Vec<PathBuf> = vec![
            [temp_path, "test_file.txt"].iter().collect(),
            [&mock_dirs[0], &PathBuf::from("test_file.txt")].iter().collect(),
            [&mock_dirs[1], &PathBuf::from("test_file.txt")].iter().collect(),
        ];

        // Create directory tree and files in the filesystem
        for mock_dir in &mock_dirs {
            fs::create_dir(&mock_dir).expect("Error creating mock directory...");
        }
        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        // Add directories and false files to arguments
        mock_files.append(&mut mock_dirs.clone());
        mock_files.push([temp_path, "false_file.txt"].iter().collect());
        #[cfg_attr(rustfmt, rustfmt_skip)]
        mock_files.push([&mock_dirs[0], &PathBuf::from("false_file.txt")].iter().collect());
        #[cfg_attr(rustfmt, rustfmt_skip)]
        mock_files.push([&mock_dirs[1], &PathBuf::from("false_file.txt")].iter().collect());

        cleanup_files(&mut mock_files);

        // Must contain these the files
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let listed_files: Vec<PathBuf> = vec![
            [temp_path, "test_file.txt"].iter().collect(),
            [temp_path, "mock_dir_1", "test_file.txt"].iter().collect(),
            [temp_path, "mock_dir_1", "mock_dir_2", "test_file.txt"].iter().collect(),
        ];
        for file in &listed_files {
            assert!(mock_files.contains(file));
        }

        // Must NOT contain these files/directories
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let non_listed_files: Vec<PathBuf> = vec![
            [temp_path, "mock_dir_1"].iter().collect(),
            [temp_path, "mock_dir_1", "mock_dir_2"].iter().collect(),
            [temp_path, "false_file.txt"].iter().collect(),
            [temp_path, "mock_dir_1", "false_file.txt"].iter().collect(),
            [temp_path, "mock_dir_1", "mock_dir_2", "false_file.txt"].iter().collect(),
        ];
        for file in &non_listed_files {
            assert!(!mock_files.contains(file));
        }
    }
}
