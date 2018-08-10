use error::*;
use fileutils::PathList;
use path_abs::PathAbs;
use std::collections::HashMap;
use std::path::PathBuf;

pub type RenameMap = HashMap<PathBuf, PathBuf>;

/// Solve renaming order to avoid file overwrite. Solver will order existing targets to avoid
/// conflicts and adds remaining targets to the list.
pub fn solve_rename_order(rename_map: &RenameMap) -> Result<PathList> {
    // Return existing targets in the list of original filenames
    let mut existing_targets = match get_existing_targets(&rename_map) {
        Ok(existing_targets) => existing_targets,
        Err(err) => return Err(err),
    };

    // Store first all non conflicting entries
    let mut rename_order: PathList = rename_map
        .iter()
        .filter_map(|(target, _)| {
            if !existing_targets.contains(&target) {
                Some(target.clone())
            } else {
                None
            }
        })
        .collect();

    // Order and store the rest of entries
    match order_existing_targets(&rename_map, &mut existing_targets) {
        Ok(mut targets) => rename_order.append(&mut targets),
        Err(err) => return Err(err),
    }

    // Move children before parent directories if they are renamed
    reorder_children_first(&rename_map, &mut rename_order);

    Ok(rename_order)
}

/// Check if targets exists in the filesystem and return a list of them. If they exist, these
/// targets must be contained in the original file list for the renaming problem to be solvable.
fn get_existing_targets(rename_map: &RenameMap) -> Result<PathList> {
    let mut existing_targets: PathList = Vec::new();
    let sources: PathList = rename_map.values().cloned().collect();

    for (target, source) in rename_map {
        if target.exists() {
            if !sources.contains(&target) {
                return Err(Error {
                    kind: ErrorKind::ExistingPath,
                    value: Some(format!("{} -> {}", source.display(), target.display())),
                });
            } else {
                existing_targets.push(target.clone());
            }
        }
    }
    Ok(existing_targets)
}

/// Process the container with existing targets until it is empty. The algorithm extracts
/// recursively all targets that are not present in a container with the sources exclusively related
/// to current existing targets.
fn order_existing_targets(
    rename_map: &RenameMap,
    existing_targets: &mut PathList,
) -> Result<PathList> {
    let mut ordered_targets: PathList = Vec::new();

    while !existing_targets.is_empty() {
        // Track selected index to extract value
        let mut selected_index: Option<usize> = None;
        // Create a vector with all sources from existing targets using absolute paths
        let sources: PathList = existing_targets
            .iter()
            .map(|x| rename_map.get(x).cloned().unwrap())
            .map(|p| PathAbs::new(p).unwrap().to_path_buf())
            .collect();
        // Select targets without conflicts in sources
        for (index, target) in existing_targets.iter().enumerate() {
            if sources.contains(&PathAbs::new(target).unwrap().to_path_buf()) {
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
            None => {
                return Err(Error {
                    kind: ErrorKind::SolveOrder,
                    value: None,
                })
            }
        }
    }

    Ok(ordered_targets)
}

/// Move children in the remaname order list before its parents if are renamed.
fn reorder_children_first(rename_map: &RenameMap, rename_order: &mut PathList) {
    let mut i = 0;
    let order_length = rename_order.len();
    while i < order_length {
        // Only consider directories, work with absolute paths to avoid bad match problems
        let source = PathAbs::new(&rename_map[&rename_order[i]]).unwrap();
        if !source.is_dir() {
            i += 1;
            continue;
        }

        let mut children_indices: Vec<usize> = Vec::new();
        for j in i + 1..rename_order.len() {
            let child_source = PathAbs::new(&rename_map[&rename_order[j]]).unwrap();
            if child_source.starts_with(&source) {
                children_indices.push(j);
            }
        }
        // Increase outer index counter when there is any change
        if children_indices.is_empty() {
            i += 1;
        } else {
            // Reorder elements in the vector
            let mut new_index = i;
            for old_index in children_indices {
                let element = rename_order.remove(old_index);
                rename_order.insert(new_index, element);
                new_index += 1;
            }
        }
    }
}

#[cfg(test)]
mod test {
    extern crate tempfile;
    use super::*;
    use std::fs;

    #[test]
    fn test_existing_targets() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_sources: PathList = vec![
            [temp_path, "a.txt"].iter().collect(),
            [temp_path, "aa.txt"].iter().collect(),
            [temp_path, "aaa.txt"].iter().collect(),
            [temp_path, "aaaa.txt"].iter().collect(),
            [temp_path, "aaaaa.txt"].iter().collect(),
            [temp_path, "test.txt"].iter().collect(),
        ];
        // Create files in the filesystem
        for file in &mock_sources {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        // Add one 'a' to the beginning of the filename
        let mock_targets: PathList = vec![
            [temp_path, "aa.txt"].iter().collect(),
            [temp_path, "aaa.txt"].iter().collect(),
            [temp_path, "aaaa.txt"].iter().collect(),
            [temp_path, "aaaaa.txt"].iter().collect(),
            [temp_path, "aaaaaa.txt"].iter().collect(),
            [temp_path, "atest.txt"].iter().collect(),
        ];
        let mock_rename_map: RenameMap = mock_targets
            .clone()
            .into_iter()
            .zip(mock_sources.into_iter())
            .collect();
        let existing_targets =
            get_existing_targets(&mock_rename_map).expect("Error getting existing targets.");

        assert!(existing_targets.contains(&mock_targets[0]));
        assert!(existing_targets.contains(&mock_targets[1]));
        assert!(existing_targets.contains(&mock_targets[2]));
        assert!(existing_targets.contains(&mock_targets[3]));
        assert!(!existing_targets.contains(&mock_targets[4]));
        assert!(!existing_targets.contains(&mock_targets[5]));
    }

    #[test]
    fn test_order_existing_targets() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_sources: PathList = vec![
            [temp_path, "a.txt"].iter().collect(),
            [temp_path, "aa.txt"].iter().collect(),
            [temp_path, "aaa.txt"].iter().collect(),
            [temp_path, "aaaa.txt"].iter().collect(),
        ];
        // Create files in the filesystem
        for file in &mock_sources {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        // Add one 'a' to the beginning of the filename
        let mock_targets: PathList = vec![
            [temp_path, "aa.txt"].iter().collect(),
            [temp_path, "aaa.txt"].iter().collect(),
            [temp_path, "aaaa.txt"].iter().collect(),
            [temp_path, "aaaaa.txt"].iter().collect(),
        ];
        let mock_rename_map: RenameMap = mock_targets
            .clone()
            .into_iter()
            .zip(mock_sources.into_iter())
            .collect();

        let mut mock_existing_targets: PathList = vec![
            [temp_path, "aa.txt"].iter().collect(),
            [temp_path, "aaa.txt"].iter().collect(),
            [temp_path, "aaaa.txt"].iter().collect(),
        ];

        let ordered_targets = order_existing_targets(&mock_rename_map, &mut mock_existing_targets)
            .expect("Failed to order existing_targets.");
        assert_eq!(
            ordered_targets[0],
            [temp_path, "aaaa.txt"].iter().collect::<PathBuf>()
        );
        assert_eq!(
            ordered_targets[1],
            [temp_path, "aaa.txt"].iter().collect::<PathBuf>()
        );
        assert_eq!(
            ordered_targets[2],
            [temp_path, "aa.txt"].iter().collect::<PathBuf>()
        );
    }

    #[test]
    fn test_solve_rename_order() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_sources: PathList = vec![
            [temp_path, "a.txt"].iter().collect(),
            [temp_path, "aa.txt"].iter().collect(),
            [temp_path, "aaa.txt"].iter().collect(),
            [temp_path, "aaaa.txt"].iter().collect(),
            [temp_path, "aaaaa.txt"].iter().collect(),
        ];
        // Create directory tree and files in the filesystem
        for file in &mock_sources {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        // Add one 'a' to the beginning of the filename
        let mock_targets: PathList = vec![
            [temp_path, "aa.txt"].iter().collect(),
            [temp_path, "aaa.txt"].iter().collect(),
            [temp_path, "aaaa.txt"].iter().collect(),
            [temp_path, "aaaaa.txt"].iter().collect(),
            [temp_path, "aaaaaa.txt"].iter().collect(),
        ];
        let mock_rename_map: RenameMap = mock_targets
            .clone()
            .into_iter()
            .zip(mock_sources.into_iter())
            .collect();

        let rename_order =
            solve_rename_order(&mock_rename_map).expect("Failed to solve rename order.");

        assert_eq!(rename_order[0], mock_targets[4]);
        assert_eq!(rename_order[1], mock_targets[3]);
        assert_eq!(rename_order[2], mock_targets[2]);
        assert_eq!(rename_order[3], mock_targets[1]);
        assert_eq!(rename_order[4], mock_targets[0]);
    }

    #[test]
    fn test_reorder_children_first() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mock_dirs: PathList = vec![
            [temp_path, "mock_dir_1"].iter().collect(),
            [temp_path, "mock_dir_1", "mock_dir_2"].iter().collect(),
        ];
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mock_files: PathList = vec![
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
        let mock_sources: PathList = vec![
            mock_dirs[0].clone(),
            mock_dirs[1].clone(),
            mock_files[0].clone(),
            mock_files[1].clone(),
            mock_files[2].clone(),
        ];
        // Add one 'a' to the beginning of the filename
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mock_targets: PathList = vec![
            [temp_path, "amock_dir_1"].iter().collect(),
            [temp_path, "mock_dir_1", "amock_dir_2"].iter().collect(),
            [temp_path, "atest_file.txt"].iter().collect(),
            [&mock_dirs[0], &PathBuf::from("atest_file.txt")].iter().collect(),
            [&mock_dirs[1], &PathBuf::from("atest_file.txt")].iter().collect(),
        ];
        let mut rename_order = mock_targets.clone();

        let mock_rename_map: RenameMap = mock_targets
            .clone()
            .into_iter()
            .zip(mock_sources.into_iter())
            .collect();

        reorder_children_first(&mock_rename_map, &mut rename_order);

        assert_eq!(rename_order[0], mock_targets[4]); // mock_dir_1/mock_dir_2/test_file.txt
        assert_eq!(rename_order[1], mock_targets[1]); // mock_dir_1/mock_dir_2/
        assert_eq!(rename_order[2], mock_targets[3]); // mock_dir_1/test_file.txt
        assert_eq!(rename_order[3], mock_targets[0]); // mock_dir_1/
        assert_eq!(rename_order[4], mock_targets[2]); // test_file.txt
    }
}
