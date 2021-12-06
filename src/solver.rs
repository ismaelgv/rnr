use error::*;
use fileutils::{is_same_file, PathList};
use path_abs::PathAbs;
use std::collections::HashMap;
use std::path::PathBuf;

pub type RenameMap = HashMap<PathBuf, PathBuf>;

// This struct stores required information about a single renaming operation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Operation {
    pub source: PathBuf,
    pub target: PathBuf,
}

pub type Operations = Vec<Operation>;

/// Solve renaming order to avoid file overwrite. Solver will order the operations considering
/// existing targets to avoid conflicts.
pub fn solve_rename_order(rename_map: &RenameMap) -> Result<Operations> {
    // Get a list of path levels
    let mut level_list: Vec<usize> = rename_map
        .values()
        .map(|p| p.components().count())
        .collect();
    level_list.sort_unstable();
    level_list.dedup();
    level_list.reverse();

    // Sort from deeper to higher path level
    let mut rename_order = PathList::new();
    for level in level_list {
        // Get all targets of this level
        let level_targets: Vec<PathBuf> = rename_map
            .keys()
            .filter_map(|p| {
                if p.components().count() == level {
                    Some(p.clone())
                } else {
                    None
                }
            })
            .collect();
        // Return existing targets in the list of original filenames
        let mut existing_targets = get_existing_targets(&level_targets, rename_map)?;

        // Store first all non conflicting entries
        rename_order.append(
            &mut level_targets
                .iter()
                .filter_map(|p| {
                    if !existing_targets.contains(p) {
                        Some(p.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        );
        // Order and append the rest of entries
        match sort_existing_targets(rename_map, &mut existing_targets) {
            Ok(mut targets) => rename_order.append(&mut targets),
            Err(err) => return Err(err),
        }
    }

    // Construct a vector with the ordered operations
    let mut operations = Operations::new();
    for target in rename_order {
        operations.push(Operation {
            source: rename_map[&target].clone(),
            target,
        });
    }

    Ok(operations)
}

/// Revert the given operations. Returns operations in reverse order and with source/target
/// fields interchanged.
pub fn revert_operations(operations: &[Operation]) -> Result<Operations> {
    let mut reverse_operations = operations.to_owned();
    reverse_operations.reverse();
    let inverse_operations = reverse_operations
        .into_iter()
        .map(|Operation { source, target }| Operation {
            source: target,
            target: source,
        })
        .collect();
    Ok(inverse_operations)
}

/// Check if targets exist in the filesystem and return a list of them. If they exist, these
/// targets must be contained in the original file list for the renaming problem to be solvable.
fn get_existing_targets(targets: &[PathBuf], rename_map: &RenameMap) -> Result<PathList> {
    let mut existing_targets: PathList = Vec::new();

    for target in targets {
        if target.symlink_metadata().is_err() {
            continue;
        }

        if !rename_map.values().any(|x| x == target) {
            let source = rename_map.get(target).cloned().unwrap();

            // The source and the target may be the same file in some conditions like case
            // insensitive but case-preserving file systems.
            if is_same_file(&source, target) {
                continue;
            }

            return Err(Error {
                kind: ErrorKind::ExistingPath,
                value: Some(format!("{} -> {}", source.display(), target.display())),
            });
        }

        existing_targets.push(target.clone());
    }
    Ok(existing_targets)
}

/// Process the container with existing targets until it is empty. The algorithm extracts
/// recursively all targets that are not present in a container with the sources exclusively related
/// to current existing targets.
fn sort_existing_targets(
    rename_map: &RenameMap,
    existing_targets: &mut PathList,
) -> Result<PathList> {
    let mut ordered_targets: PathList = Vec::new();

    while !existing_targets.is_empty() {
        // Track selected index to extract value
        let mut selected_index: Option<usize> = None;

        // Create a vector with all sources from existing targets using absolute paths
        #[allow(clippy::needless_collect)] // Benchmark shows no diff, code is clearer this way.
        let sources: PathList = existing_targets
            .iter()
            .map(|x| rename_map.get(x).cloned().unwrap())
            .map(|p| PathAbs::new(p).unwrap().to_path_buf())
            .collect();

        // Select targets without conflicts in sources
        for (index, target) in existing_targets.iter().enumerate() {
            let absolute_target = PathAbs::new(target).unwrap().to_path_buf();
            if !sources.contains(&absolute_target) {
                selected_index = Some(index);
                break;
            }
        }

        // Store result in ordered targets container or fail to stop the loop
        match selected_index {
            Some(index) => ordered_targets.push(existing_targets.swap_remove(index)),
            // This will avoid infinite while loop if order is not solved
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

#[cfg(test)]
mod test {
    extern crate tempfile;
    use super::*;
    use fileutils::create_symlink;
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
        let existing_targets = get_existing_targets(&mock_targets, &mock_rename_map)
            .expect("Error getting existing targets.");

        assert!(existing_targets.contains(&mock_targets[0]));
        assert!(existing_targets.contains(&mock_targets[1]));
        assert!(existing_targets.contains(&mock_targets[2]));
        assert!(existing_targets.contains(&mock_targets[3]));
        assert!(!existing_targets.contains(&mock_targets[4]));
        assert!(!existing_targets.contains(&mock_targets[5]));
    }

    #[test]
    fn test_existing_targets_symlinks() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_sources: PathList = vec![
            [temp_path, "a.txt"].iter().collect(),
            [temp_path, "aa.txt"].iter().collect(),
            [temp_path, "aaa.txt"].iter().collect(),
        ];
        // Create files in the filesystem
        fs::File::create(&mock_sources[0]).expect("Error creating mock file...");
        // Create symlinks
        create_symlink(&mock_sources[0], &mock_sources[1]).expect("Error creating symlink.");
        create_symlink(&PathBuf::from("broken_link"), &mock_sources[2])
            .expect("Error creating broken symlink.");

        // Add one 'a' to the beginning of the filename
        let mock_targets: PathList = vec![
            [temp_path, "aa.txt"].iter().collect(),
            [temp_path, "aaa.txt"].iter().collect(),
            [temp_path, "aaaa.txt"].iter().collect(),
        ];
        let mock_rename_map: RenameMap = mock_targets
            .clone()
            .into_iter()
            .zip(mock_sources.into_iter())
            .collect();
        let existing_targets = get_existing_targets(&mock_targets, &mock_rename_map)
            .expect("Error getting existing targets.");

        assert!(existing_targets.contains(&mock_targets[0]));
        assert!(existing_targets.contains(&mock_targets[1]));
        assert!(!existing_targets.contains(&mock_targets[2]));
    }

    #[test]
    fn test_sort_existing_targets() {
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

        let ordered_targets = sort_existing_targets(&mock_rename_map, &mut mock_existing_targets)
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

        let operations =
            solve_rename_order(&mock_rename_map).expect("Failed to solve rename order.");

        assert_eq!(operations[0].target, mock_targets[4]);
        assert_eq!(operations[1].target, mock_targets[3]);
        assert_eq!(operations[2].target, mock_targets[2]);
        assert_eq!(operations[3].target, mock_targets[1]);
        assert_eq!(operations[4].target, mock_targets[0]);
    }
}
