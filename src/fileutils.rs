use config::RunMode;
use error::*;
use path_abs::PathAbs;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

pub type PathList = Vec<PathBuf>;

/// Return a list of paths for the given run mode.
pub fn get_paths(mode: &RunMode) -> PathList {
    match mode {
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
        RunMode::Simple(path_list) => path_list.iter().map(PathBuf::from).collect(),
        // Return an empty PathList otherwise
        _ => PathList::new(),
    }
}

/// Generate a non-existing name adding numbers to the end of the file name. It also supports adding a
/// suffix to the original name.
pub fn get_unique_filename(path: &Path, suffix: &str) -> PathBuf {
    let base_name = format!("{}{}", path.file_name().unwrap().to_string_lossy(), suffix);
    let mut unique_name = path.to_path_buf();
    unique_name.set_file_name(&base_name);

    let mut index = 0;
    while unique_name.symlink_metadata().is_ok() {
        index += 1;
        unique_name.set_file_name(format!("{}.{}", base_name, index));
    }

    unique_name
}

/// Create a backup of the file
pub fn create_backup(path: &Path) -> Result<PathBuf> {
    let backup = get_unique_filename(path, ".bk");
    match fs::copy(path, &backup) {
        Ok(_) => Ok(backup),
        Err(_) => Err(Error {
            kind: ErrorKind::CreateBackup,
            value: Some(path.to_string_lossy().to_string()),
        }),
    }
}

/// Clean paths that does not exists and duplicated entries. It remove directories too if dirs
/// parameters is set to false.
pub fn cleanup_paths(paths: &mut PathList, keep_dirs: bool) {
    paths.retain(|path| {
        // Path must exists before performing other checks
        if path.symlink_metadata().is_err() {
            return false;
        }

        if path.is_dir() {
            keep_dirs && path.file_name().is_some()
        } else {
            true
        }
    });

    // Deduplicate paths generating their absolute path and inserting them in a Hashmap. Replace
    // the PathList original content with the deduplicated data.
    let abs_path_map: HashMap<PathAbs, PathBuf> = paths
        .drain(..)
        .map(|p| (PathAbs::new(&p).unwrap(), p))
        .collect();
    paths.append(&mut abs_path_map.values().cloned().collect());
}

/// Wrapper to create symlink files without considering the OS explicitly
#[allow(dead_code)]
pub fn create_symlink(source: &Path, symlink_file: &Path) -> Result<()> {
    #[cfg(windows)]
    match ::std::os::windows::fs::symlink_file(source, symlink_file) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error {
            kind: ErrorKind::CreateSymlink,
            value: Some(symlink_file.to_string_lossy().to_string()),
        }),
    }
    #[cfg(unix)]
    match ::std::os::unix::fs::symlink(source, symlink_file) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error {
            kind: ErrorKind::CreateSymlink,
            value: Some(symlink_file.to_string_lossy().to_string()),
        }),
    }
}

/// Check if the paths references the same file. This is useful in case insensitive systems.
pub fn is_same_file(source: &Path, target: &Path) -> bool {
    // Only perform a more exhaustive check for platform that support case insensitive and case
    // preserving file systems by default.
    #[cfg(any(windows, target_os = "macos"))]
    {
        let source_metadata = fs::File::open(&source).unwrap().metadata().unwrap();
        let target_metadata = fs::File::open(&target).unwrap().metadata().unwrap();
        let low_source = source.to_string_lossy().to_string().to_lowercase();
        let low_target = target.to_string_lossy().to_string().to_lowercase();

        return low_source == low_target
            && source_metadata.file_type() == target_metadata.file_type()
            && source_metadata.len() == target_metadata.len()
            && source_metadata.created().unwrap() == target_metadata.created().unwrap()
            && source_metadata.modified().unwrap() == target_metadata.modified().unwrap();
    }

    source == target
}

#[cfg(test)]
mod test {
    extern crate tempfile;
    use super::*;
    use std::fs;
    use std::io::prelude::*;

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
            create_backup(file).expect("Error generating backup file...");
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

        let symlink = PathBuf::from(format!("{}/test_file_1.3", temp_path));
        create_symlink(&mock_files[0], &symlink).expect("Error creating symlink.");

        let broken_symlink = PathBuf::from(format!("{}/test_file_1.4", temp_path));
        create_symlink(&PathBuf::from("broken_link"), &broken_symlink)
            .expect("Error creating broken symlink.");

        let new_file: PathBuf = [temp_path, "test_file_1.5"].iter().collect();
        assert_eq!(get_unique_filename(&mock_files[0], ""), new_file);
    }

    #[test]
    fn get_file_list() {
        let mock_files: Vec<String> = vec![
            "test_file_1.txt".to_string(),
            "test_file_2.txt".to_string(),
            "test_file_3.txt".to_string(),
        ];

        let mode = RunMode::Simple(mock_files);
        let files = get_paths(&mode);
        assert!(files.contains(&PathBuf::from("test_file_1.txt")));
        assert!(files.contains(&PathBuf::from("test_file_2.txt")));
        assert!(files.contains(&PathBuf::from("test_file_3.txt")));
    }

    #[test]
    fn test_create_symlinks() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_string_lossy().to_string().clone();

        let file = PathBuf::from(format!("{}/test_file", temp_path));
        fs::File::create(&file).expect("Error creating mock file...");

        let symlink = PathBuf::from(format!("{}/test_link", temp_path));
        create_symlink(&file, &symlink).expect("Error creating symlink.");

        let broken_symlink = PathBuf::from(format!("{}/test_broken_link", temp_path));
        create_symlink(&PathBuf::from("broken_link"), &broken_symlink)
            .expect("Error creating broken symlink.");

        assert!(file.symlink_metadata().is_ok());
        assert!(symlink.symlink_metadata().is_ok());
        assert!(broken_symlink.symlink_metadata().is_ok());
    }

    #[test]
    fn test_same_file() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_files: PathList = vec![
            [temp_path, "test_FILE"].iter().collect(),
            [temp_path, "test_File"].iter().collect(),
            [temp_path, "test_file"].iter().collect(),
        ];

        let other_file = PathBuf::from(format!("{}/other_file", temp_path));

        for file in &mock_files {
            fs::File::create(&file)
                .expect("Error creating mock file...")
                .write_all(b"Hello, world!")
                .expect("Error writting in the mock file...");
        }

        fs::File::create(&other_file)
            .expect("Error creating mock file...")
            .write_all(b"Hello, world!")
            .expect("Error writting in the mock file...");

        #[cfg(any(windows, target_os = "macos"))]
        {
            assert!(is_same_file(&mock_files[0], &mock_files[0]));
            assert!(is_same_file(&mock_files[0], &mock_files[1]));
            assert!(is_same_file(&mock_files[0], &mock_files[2]));
            assert!(is_same_file(&mock_files[1], &mock_files[2]));
            assert!(!is_same_file(&mock_files[0], &other_file));
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        {
            assert!(is_same_file(&mock_files[0], &mock_files[0]));
            assert!(!is_same_file(&mock_files[0], &mock_files[1]));
            assert!(!is_same_file(&mock_files[0], &mock_files[2]));
            assert!(!is_same_file(&mock_files[1], &mock_files[2]));
            assert!(!is_same_file(&mock_files[0], &other_file));
        }
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
        #[rustfmt::skip]
        let mock_dirs: PathList = vec![
            [&temp_path, ".hidden_mock_dir"].iter().collect(),
            [&temp_path, "mock_dir_1"].iter().collect(),
            [&temp_path, "mock_dir_1", "mock_dir_2"].iter().collect(),
            [&temp_path, "mock_dir_1", "mock_dir_2", "mock_dir_3"].iter().collect(),
        ];
        #[rustfmt::skip]
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

        // Create mode with recursive search WITHOUT max depth
        let mode = RunMode::Recursive {
            paths: vec![temp_path.clone()],
            max_depth: None,
            hidden: false,
        };
        let files = get_paths(&mode);
        // Must contain these files
        #[rustfmt::skip]
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
        #[rustfmt::skip]
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

        // Create mode with recursive search WITH max depth
        let mode = RunMode::Recursive {
            paths: vec![temp_path.clone()],
            max_depth: Some(2),
            hidden: false,
        };
        let files = get_paths(&mode);
        // Must contain these files
        let listed_files: PathList = vec![
            [&temp_path, "test_file.txt"].iter().collect(),
            [&temp_path, "mock_dir_1", "test_file.txt"].iter().collect(),
        ];
        for file in &listed_files {
            assert!(files.contains(file));
        }
        // Must NOT contain these files
        #[rustfmt::skip]
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

        // Create mode with recursive search WITHOUT max depth
        let mode = RunMode::Recursive {
            paths: vec![temp_path.clone()],
            max_depth: None,
            hidden: true,
        };
        let files = get_paths(&mode);
        // Must contain these files
        #[rustfmt::skip]
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
        //     - test_link -> test_file.txt
        //     |
        //     - test_broken_link -> broken_link
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
        #[rustfmt::skip]
        let mock_files: PathList = vec![
            [temp_path, "test_file.txt"].iter().collect(),
            [&mock_dirs[0], &PathBuf::from("test_file.txt")].iter().collect(),
            [&mock_dirs[1], &PathBuf::from("test_file.txt")].iter().collect(),
        ];

        // Create directory tree, files and symlinks in the filesystem
        for mock_dir in &mock_dirs {
            fs::create_dir(&mock_dir).expect("Error creating mock directory...");
        }
        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }
        let symlink: PathBuf = [temp_path, "test_link"].iter().collect();
        let broken_symlink: PathBuf = [temp_path, "test_broken_link"].iter().collect();
        create_symlink(&mock_files[0], &symlink).expect("Error creating symlink.");
        create_symlink(&PathBuf::from("broken_link"), &broken_symlink)
            .expect("Error creating broken symlink.");

        // Create mock_paths from files, symlink, directories, false files and duplicated files
        // Existing files
        let mut mock_paths = PathList::new();
        mock_paths.append(&mut mock_files.clone());
        // Symlinks
        mock_paths.push(symlink.clone());
        mock_paths.push(broken_symlink.clone());
        // Directories
        mock_paths.append(&mut mock_dirs.clone());
        // False files
        #[rustfmt::skip]
        let false_files: PathList = vec![
            [temp_path, "false_file.txt"].iter().collect(),
            [&mock_dirs[0], &PathBuf::from("false_file.txt")].iter().collect(),
            [&mock_dirs[1], &PathBuf::from("false_file.txt")].iter().collect(),
        ];
        mock_paths.append(&mut mock_files.clone());
        // Quadruplicate existing files
        mock_paths.append(&mut mock_files.clone());
        mock_paths.append(&mut mock_files.clone());
        mock_paths.append(&mut mock_files.clone());

        cleanup_paths(&mut mock_paths, false);

        // Must contain these the files
        let mut listed_files = PathList::new();
        listed_files.append(&mut mock_files.clone());
        listed_files.push(symlink.clone());
        listed_files.push(broken_symlink.clone());

        for file in &listed_files {
            assert!(mock_paths.contains(file));
            // Only once
            assert_eq!(mock_paths.iter().filter(|f| f == &file).count(), 1);
        }

        // Must NOT contain these files/directories
        #[rustfmt::skip]
        let mut non_listed_files = PathList::new();
        non_listed_files.append(&mut mock_dirs.clone());
        non_listed_files.append(&mut false_files.clone());
        for file in &non_listed_files {
            assert!(!mock_paths.contains(file));
        }
    }
}
