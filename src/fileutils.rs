use config::{Config, RunMode};
use error::*;
use path_abs::PathAbs;
use std::fs;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

pub type PathList = Vec<PathBuf>;

/// Return a list of paths for the given configuration.
pub fn get_paths(config: &Config) -> PathList {
    match &config.mode {
        RunMode::Recursive {
            paths,
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
            // Get recursive list of paths walking directories
            let mut path_list = PathList::new();
            for path in paths {
                let walkdir = match max_depth {
                    Some(max_depth) => WalkDir::new(path).max_depth(*max_depth),
                    None => WalkDir::new(path),
                };
                let mut walk_list: PathList = walkdir
                    .into_iter()
                    .filter_entry(is_hidden)
                    .filter_map(|e| e.ok())
                    .map(|p| p.path().to_path_buf())
                    .collect();
                path_list.append(&mut walk_list);
            }

            path_list
        }
        RunMode::FileList(path_list) => path_list.into_iter().map(PathBuf::from).collect(),
    }
}

/// Generate a non-existing name adding numbers to the end of the file name. It also supports adding a
/// suffix to the original name.
pub fn get_unique_filename(path: &PathBuf, suffix: &str) -> PathBuf {
    let base_name = format!("{}{}", path.file_name().unwrap().to_string_lossy(), suffix);
    let mut unique_name = path.clone();
    unique_name.set_file_name(&base_name);

    let mut index = 0;
    while unique_name.exists() {
        index += 1;
        unique_name.set_file_name(format!("{}.{}", base_name, index));
    }

    unique_name
}

/// Create a backup of the file
pub fn create_backup(path: &PathBuf) -> Result<PathBuf> {
    let backup = get_unique_filename(path, ".bk");
    match fs::copy(path, &backup) {
        Ok(_) => Ok(backup),
        Err(_) => Err(Error {
            kind: ErrorKind::CreateBackup,
            value: Some(path.to_string_lossy().to_string()),
        }),
    }
}

/// Clean paths that does not exists, broken links and duplicated entries. It remove directories
/// too if dirs parameters is set to false.
pub fn cleanup_paths(paths: &mut PathList, keep_dirs: bool) {
    paths.retain(|path| {
        if !path.exists() {
            // Checks if non-existing path is actually a symlink
            path.read_link().is_ok()
        } else if path.is_dir() {
            keep_dirs && path.file_name().is_some()
        } else {
            true
        }
    });

    // Remove duplicated entries using absolute path
    paths.sort_unstable_by(|a, b| PathAbs::new(a).unwrap().cmp(&PathAbs::new(b).unwrap()));
    paths.dedup_by(|a, b| PathAbs::new(a).unwrap().eq(&PathAbs::new(b).unwrap()));
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

        let mock_files: PathList = vec![
            [temp_path, "test_file_1.txt"].iter().collect(),
            [temp_path, "test_file_2.txt"].iter().collect(),
            [temp_path, "test_file_3.txt"].iter().collect(),
        ];

        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
            create_backup(&file).expect("Error generating backup file...");
        }

        let backup_files: PathList = vec![
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

        let mock_files: PathList = vec![
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
    fn get_file_list() {
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
            dirs: false,
            mode: RunMode::FileList(mock_files),
            printer: Printer::colored(),
        };

        let files = get_paths(&mock_config);
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
        let mock_dirs: PathList = vec![
            [&temp_path, ".hidden_mock_dir"].iter().collect(),
            [&temp_path, "mock_dir_1"].iter().collect(),
            [&temp_path, "mock_dir_1", "mock_dir_2"].iter().collect(),
            [&temp_path, "mock_dir_1", "mock_dir_2", "mock_dir_3"].iter().collect(),
        ];
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mock_files: PathList = vec![
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
    fn get_paths_recursive() {
        let (_tempdir, temp_path) = generate_recursive_tempdir();

        // Create config with recursive search WITHOUT max depth
        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: false,
            backup: false,
            dirs: false,
            mode: RunMode::Recursive {
                paths: vec![temp_path.clone()],
                max_depth: None,
                hidden: false,
            },
            printer: Printer::colored(),
        };

        let files = get_paths(&mock_config);
        // Must contain these files
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let listed_files: PathList = vec![
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
        let non_listed_files: PathList = vec![
            [&temp_path, ".hidden_test_file.txt"].iter().collect(),
            [&temp_path, ".hidden_mock_dir", "test_file.txt"].iter().collect(),
        ];
        for file in &non_listed_files {
            assert!(!files.contains(file));
        }
    }

    #[test]
    fn get_paths_recursive_depth() {
        let (_tempdir, temp_path) = generate_recursive_tempdir();

        // Create config with recursive search WITH max depth
        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: false,
            backup: false,
            dirs: false,
            mode: RunMode::Recursive {
                paths: vec![temp_path.clone()],
                max_depth: Some(2),
                hidden: false,
            },
            printer: Printer::colored(),
        };

        let files = get_paths(&mock_config);
        // Must contain these files
        let listed_files: PathList = vec![
            [&temp_path, "test_file.txt"].iter().collect(),
            [&temp_path, "mock_dir_1", "test_file.txt"].iter().collect(),
        ];
        for file in &listed_files {
            assert!(files.contains(file));
        }
        // Must NOT contain these files
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let non_listed_files: PathList = vec![
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
    fn get_paths_recursive_hidden() {
        let (_tempdir, temp_path) = generate_recursive_tempdir();

        // Create config with recursive search WITHOUT max depth
        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: false,
            backup: false,
            dirs: false,
            mode: RunMode::Recursive {
                paths: vec![temp_path.clone()],
                max_depth: None,
                hidden: true,
            },
            printer: Printer::colored(),
        };

        let files = get_paths(&mock_config);
        // Must contain these files
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let listed_files: PathList = vec![
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
        let mock_dirs: PathList = vec![
            [temp_path, "mock_dir_1"].iter().collect(),
            [temp_path, "mock_dir_1", "mock_dir_2"].iter().collect(),
        ];
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mut mock_files: PathList = vec![
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

        // Add directories, false files and duplicated files to arguments
        // Directories
        mock_files.append(&mut mock_dirs.clone());
        // False files
        mock_files.push([temp_path, "false_file.txt"].iter().collect());
        #[cfg_attr(rustfmt, rustfmt_skip)]
        mock_files.push([&mock_dirs[0], &PathBuf::from("false_file.txt")].iter().collect());
        #[cfg_attr(rustfmt, rustfmt_skip)]
        mock_files.push([&mock_dirs[1], &PathBuf::from("false_file.txt")].iter().collect());
        // Duplicated files
        let duplicated_files = mock_files.clone();
        mock_files.extend_from_slice(&duplicated_files[..]);

        cleanup_paths(&mut mock_files, false);

        // Must contain these the files
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let listed_files: PathList = vec![
            [temp_path, "test_file.txt"].iter().collect(),
            [temp_path, "mock_dir_1", "test_file.txt"].iter().collect(),
            [temp_path, "mock_dir_1", "mock_dir_2", "test_file.txt"].iter().collect(),
        ];
        for file in &listed_files {
            assert!(mock_files.contains(file));
            // Only once
            assert_eq!(mock_files.iter().filter(|f| f == &file).count(), 1);
        }

        // Must NOT contain these files/directories
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let non_listed_files: PathList = vec![
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
