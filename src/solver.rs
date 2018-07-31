use error::*;
use fileutils::PathList;
use std::collections::HashMap;
use std::path::PathBuf;

pub type RenameMap = HashMap<PathBuf, PathBuf>;

/// Solve renaming order to avoid file overwrite. Solver will order existing targets to avoid
/// conflicts and adds remaining targets to the list.
pub fn solve_rename_order(rename_map: &RenameMap) -> Result<PathList> {
    // Return existing targets in the list of original filenames
    let existing_targets = match get_existing_targets(&rename_map) {
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
    match order_existing_targets(&rename_map, existing_targets) {
        Ok(mut targets) => rename_order.append(&mut targets),
        Err(err) => return Err(err),
    }

    Ok(rename_order)
}

/// Check if targets exists in the filesystem and return a list of them. If they exist, these
/// targets must be contained in the original file list for the renaming problem to be solvable.
fn get_existing_targets(rename_map: &RenameMap) -> Result<PathList> {
    let mut existing_targets: Vec<PathBuf> = Vec::new();
    let sources: Vec<PathBuf> = rename_map.values().cloned().collect();

    for (target, source) in rename_map {
        if target.exists() {
            if !sources.contains(&target) {
                return Err(Error {
                    kind: ErrorKind::ExistingFile,
                    value: Some(format!("{}->{}", source.display(), target.display())),
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
    mut existing_targets: PathList,
) -> Result<PathList> {
    let mut ordered_targets: Vec<PathBuf> = Vec::new();

    while !existing_targets.is_empty() {
        // Track selected index to extract value
        let mut selected_index: Option<usize> = None;
        // Create a vector with all sources from existing targets
        let sources: Vec<PathBuf> = existing_targets
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
    use std::fs;

    #[test]
    fn test_existing_targets() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_sources: Vec<PathBuf> = vec![
            [temp_path, "a.txt"].iter().collect(),
            [temp_path, "aa.txt"].iter().collect(),
            [temp_path, "aaa.txt"].iter().collect(),
            [temp_path, "aaaa.txt"].iter().collect(),
            [temp_path, "aaaaa.txt"].iter().collect(),
            [temp_path, "test.txt"].iter().collect(),
        ];
        // Create directory tree and files in the filesystem
        for file in &mock_sources {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        // Add one 'a' to the beginning of the filename
        let mock_targets: Vec<PathBuf> = vec![
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
        let mock_sources: Vec<PathBuf> = vec![
            PathBuf::from("a.txt"),
            PathBuf::from("aa.txt"),
            PathBuf::from("aaa.txt"),
            PathBuf::from("aaaa.txt"),
        ];
        // Add one 'a' to the beginning of the filename
        let mock_targets: Vec<PathBuf> = vec![
            PathBuf::from("aa.txt"),
            PathBuf::from("aaa.txt"),
            PathBuf::from("aaaa.txt"),
            PathBuf::from("aaaaa.txt"),
        ];
        let mock_rename_map: RenameMap = mock_targets
            .clone()
            .into_iter()
            .zip(mock_sources.into_iter())
            .collect();

        let mock_existing_targets: Vec<PathBuf> = vec![
            PathBuf::from("aa.txt"),
            PathBuf::from("aaa.txt"),
            PathBuf::from("aaaa.txt"),
        ];

        let ordered_targets = order_existing_targets(&mock_rename_map, mock_existing_targets)
            .expect("Failed to order existing_targets.");
        assert_eq!(ordered_targets[0], PathBuf::from("aaaa.txt"));
        assert_eq!(ordered_targets[1], PathBuf::from("aaa.txt"));
        assert_eq!(ordered_targets[2], PathBuf::from("aa.txt"));
    }

    #[test]
    fn test_solve_rename_order() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_sources: Vec<PathBuf> = vec![
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
        let mock_targets: Vec<PathBuf> = vec![
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
}
