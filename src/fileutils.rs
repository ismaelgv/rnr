use app::Config;
use app::RunMode;
use error::*;
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

/// Return a list of files for the given configuration.
pub fn get_files(config: &Config) -> Vec<String> {
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
                .filter_map(|x| match x.path().to_str() {
                    Some(s) => Some(s.to_string()),
                    None => {
                        let warn = &config.printer.colors.warn;
                        config.printer.eprint(&format!(
                            "{}File '{}' contains invalid characters",
                            warn.paint("Warn: "),
                            warn.paint(x.path().to_string_lossy())
                        ));
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
pub fn get_unique_filename(file: &str, suffix: &str) -> String {
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
pub fn create_backup(file: &str) -> Result<String> {
    let backup = get_unique_filename(file, ".bk");
    match fs::copy(file, &backup) {
        Ok(_) => Ok(backup),
        Err(_) => Err(Error {
            kind: ErrorKind::CreateBackup,
            value: Some(file.to_string()),
        }),
    }
}

/// Clean files that does not exists, broken links and directories
pub fn cleanup_files(files: &mut Vec<String>) {
    files.retain(|file| {
        if !Path::new(&file).exists() {
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
    use std::path::Path;

    #[test]
    fn backup() {
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
            create_backup(&file).expect("Error generating backup file...");
        }

        assert!(Path::new(&format!("{}/test_file_1.txt.bk", temp_path)).exists());
        assert!(Path::new(&format!("{}/test_file_2.txt.bk", temp_path)).exists());
        assert!(Path::new(&format!("{}/test_file_3.txt.bk", temp_path)).exists());
    }

    #[test]
    fn unique_name() {
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
    fn get_files_args() {
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
            printer: Printer::colored(),
        };

        let files = get_files(&mock_config);
        assert!(files.contains(&format!("{}/test_file_1.txt", temp_path)));
        assert!(files.contains(&format!("{}/test_file_2.txt", temp_path)));
        assert!(files.contains(&format!("{}/test_file_3.txt", temp_path)));
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
        let mock_dirs: Vec<String> = vec![
            format!("{}/.hidden_mock_dir", temp_path),
            format!("{}/mock_dir_1", temp_path),
            format!("{}/mock_dir_1/mock_dir_2", temp_path),
            format!("{}/mock_dir_1/mock_dir_2/mock_dir_3", temp_path),
        ];
        let mock_files: Vec<String> = vec![
            format!("{}/test_file.txt", temp_path),
            format!("{}/.hidden_test_file.txt", temp_path),
            format!("{}/test_file.txt", mock_dirs[0]),
            format!("{}/test_file.txt", mock_dirs[1]),
            format!("{}/test_file.txt", mock_dirs[2]),
            format!("{}/test_file.txt", mock_dirs[3]),
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
        // Must NOT contain these files
        assert!(!files.contains(&format!("{}/.hidden_test_file.txt", temp_path)));
        assert!(!files.contains(&format!("{}/.hidden_mock_dir/test_file.txt", temp_path)));
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
        assert!(!files.contains(&format!("{}/.hidden_test_file.txt", temp_path)));
        assert!(!files.contains(&format!("{}/.hidden_mock_dir/test_file.txt", temp_path)));
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
        assert!(files.contains(&format!("{}/.hidden_test_file.txt", temp_path)));
        assert!(files.contains(&format!("{}/.hidden_mock_dir/test_file.txt", temp_path)));
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
        let mock_dirs: Vec<String> = vec![
            format!("{}/mock_dir_1", temp_path),
            format!("{}/mock_dir_1/mock_dir_2", temp_path),
        ];
        let mut mock_files: Vec<String> = vec![
            format!("{}/test_file.txt", temp_path),
            format!("{}/test_file.txt", mock_dirs[0]),
            format!("{}/test_file.txt", mock_dirs[1]),
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
        mock_files.push(format!("{}/false_file.txt", temp_path));
        mock_files.push(format!("{}/false_file.txt", mock_dirs[0]));
        mock_files.push(format!("{}/false_file.txt", mock_dirs[1]));

        cleanup_files(&mut mock_files);

        // Must contain these the files
        assert!(mock_files.contains(&format!("{}/test_file.txt", temp_path)));
        assert!(mock_files.contains(&format!("{}/mock_dir_1/test_file.txt", temp_path)));
        assert!(mock_files.contains(&format!(
            "{}/mock_dir_1/mock_dir_2/test_file.txt",
            temp_path
        )));
        // Must NOT contain these files/directories
        assert!(!mock_files.contains(&format!("{}/mock_dir_1", temp_path)));
        assert!(!mock_files.contains(&format!("{}/mock_dir_1/mock_dir_2", temp_path)));
        assert!(!mock_files.contains(&format!("{}/false_file.txt", temp_path)));
        assert!(!mock_files.contains(&format!("{}/mock_dir_1/false_file.txt", temp_path)));
        assert!(!mock_files.contains(&format!(
            "{}/mock_dir_1/mock_dir_2/false_file.txt",
            temp_path
        )));
    }
}
