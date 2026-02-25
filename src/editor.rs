use crate::error::*;
use crate::solver::{Operation, Operations};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;

/// Result of the editor interaction: rename operations and paths to delete.
#[derive(Debug)]
pub struct EditorResult {
    pub operations: Operations,
    pub deletions: Vec<PathBuf>,
}

/// Open the given paths in a text editor and return the resulting rename/delete operations.
///
/// When `allow_delete` is `false` the temp file lists bare paths one per line and the line
/// count must match after editing (an error is raised otherwise).
///
/// When `allow_delete` is `true` each line is prefixed with a 1-based index and a tab
/// (`INDEX\tPATH`).  Removing a line deletes the corresponding file; changing the path after
/// the tab renames it.
pub fn open_editor(paths: &[PathBuf], editor: &str, allow_delete: bool) -> Result<EditorResult> {
    // Build the content that will be shown in the editor
    let content: String = if allow_delete {
        paths
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{}\t{}", i + 1, p.display()))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Write content to a named temporary file so the editor can open it by path
    let mut temp_file = Builder::new()
        .prefix("rnr-editor-")
        .suffix(".txt")
        .tempfile()
        .map_err(|e| Error {
            kind: ErrorKind::CreateFile,
            value: Some(format!("temporary file: {}", e)),
        })?;

    temp_file
        .write_all(content.as_bytes())
        .map_err(|e| Error {
            kind: ErrorKind::CreateFile,
            value: Some(format!("write temporary file: {}", e)),
        })?;
    // Flush so the editor sees the content
    temp_file.flush().map_err(|e| Error {
        kind: ErrorKind::CreateFile,
        value: Some(format!("flush temporary file: {}", e)),
    })?;

    let temp_path = temp_file.path().to_path_buf();

    // Launch editor and wait for it to finish
    let status = Command::new(editor)
        .arg(&temp_path)
        .status()
        .map_err(|e| Error {
            kind: ErrorKind::EditorCommand,
            value: Some(format!("'{}': {}", editor, e)),
        })?;

    if !status.success() {
        return Err(Error {
            kind: ErrorKind::EditorCommand,
            value: Some(format!("'{}' exited with status {}", editor, status)),
        });
    }

    // Read the edited content back
    let edited_content = std::fs::read_to_string(&temp_path).map_err(|e| Error {
        kind: ErrorKind::ReadFile,
        value: Some(format!("{}: {}", temp_path.display(), e)),
    })?;

    let edited_lines: Vec<&str> = edited_content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if allow_delete {
        parse_with_delete(paths, &edited_lines)
    } else {
        parse_without_delete(paths, &edited_lines)
    }
}

/// Parse editor output when deletion is NOT allowed.
/// Each line must correspond positionally to the original path list.
fn parse_without_delete(paths: &[PathBuf], edited_lines: &[&str]) -> Result<EditorResult> {
    if paths.len() != edited_lines.len() {
        return Err(Error {
            kind: ErrorKind::EditorLineCount,
            value: Some(format!(
                "expected {} lines but got {}. Use --delete to enable deletion.",
                paths.len(),
                edited_lines.len()
            )),
        });
    }

    let mut operations = Operations::new();
    for (source, new_line) in paths.iter().zip(edited_lines.iter()) {
        let target = PathBuf::from(new_line.trim());
        if *source != target {
            operations.push(Operation {
                source: source.clone(),
                target,
            });
        }
    }

    Ok(EditorResult {
        operations,
        deletions: vec![],
    })
}

/// Parse editor output when deletion IS allowed.
/// Lines have the format `INDEX\tPATH`.  Missing indices mean deletion.
fn parse_with_delete(paths: &[PathBuf], edited_lines: &[&str]) -> Result<EditorResult> {
    use std::collections::HashMap;

    let mut index_to_new_path: HashMap<usize, PathBuf> = HashMap::new();

    for line in edited_lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match line.find('\t') {
            None => {
                return Err(Error {
                    kind: ErrorKind::EditorLineCount,
                    value: Some(format!(
                        "line '{}' is missing the index prefix (expected 'INDEX<TAB>PATH')",
                        line
                    )),
                });
            }
            Some(tab_pos) => {
                let index_str = line[..tab_pos].trim();
                let path_str = line[tab_pos + 1..].trim();
                let index: usize = index_str.parse().map_err(|_| Error {
                    kind: ErrorKind::EditorLineCount,
                    value: Some(format!("invalid index '{}' in line '{}'", index_str, line)),
                })?;
                if index < 1 || index > paths.len() {
                    return Err(Error {
                        kind: ErrorKind::EditorLineCount,
                        value: Some(format!(
                            "index {} is out of range (1–{})",
                            index,
                            paths.len()
                        )),
                    });
                }
                if index_to_new_path
                    .insert(index, PathBuf::from(path_str))
                    .is_some()
                {
                    return Err(Error {
                        kind: ErrorKind::EditorLineCount,
                        value: Some(format!("duplicate index {} in editor output", index)),
                    });
                }
            }
        }
    }

    let mut operations = Operations::new();
    let mut deletions = Vec::new();

    for (i, source) in paths.iter().enumerate() {
        let index = i + 1;
        match index_to_new_path.get(&index) {
            Some(target) => {
                if source != target {
                    operations.push(Operation {
                        source: source.clone(),
                        target: target.clone(),
                    });
                }
            }
            None => {
                deletions.push(source.clone());
            }
        }
    }

    Ok(EditorResult {
        operations,
        deletions,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    fn paths(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(|n| PathBuf::from(n)).collect()
    }

    // --- parse_without_delete ---

    #[test]
    fn no_delete_unchanged() {
        let p = paths(&["/tmp/a.txt", "/tmp/b.txt"]);
        let lines = vec!["/tmp/a.txt", "/tmp/b.txt"];
        let result = parse_without_delete(&p, &lines).unwrap();
        assert!(result.operations.is_empty());
        assert!(result.deletions.is_empty());
    }

    #[test]
    fn no_delete_rename() {
        let p = paths(&["/tmp/a.txt", "/tmp/b.txt"]);
        let lines = vec!["/tmp/a_new.txt", "/tmp/b.txt"];
        let result = parse_without_delete(&p, &lines).unwrap();
        assert_eq!(result.operations.len(), 1);
        assert_eq!(result.operations[0].source, PathBuf::from("/tmp/a.txt"));
        assert_eq!(result.operations[0].target, PathBuf::from("/tmp/a_new.txt"));
        assert!(result.deletions.is_empty());
    }

    #[test]
    fn no_delete_wrong_line_count_error() {
        let p = paths(&["/tmp/a.txt", "/tmp/b.txt"]);
        let lines = vec!["/tmp/a.txt"]; // one line missing
        let err = parse_without_delete(&p, &lines).unwrap_err();
        assert_eq!(err.kind, ErrorKind::EditorLineCount);
    }

    // --- parse_with_delete ---

    #[test]
    fn with_delete_unchanged() {
        let p = paths(&["/tmp/a.txt", "/tmp/b.txt"]);
        let lines = vec!["1\t/tmp/a.txt", "2\t/tmp/b.txt"];
        let result = parse_with_delete(&p, &lines).unwrap();
        assert!(result.operations.is_empty());
        assert!(result.deletions.is_empty());
    }

    #[test]
    fn with_delete_rename() {
        let p = paths(&["/tmp/a.txt", "/tmp/b.txt"]);
        let lines = vec!["1\t/tmp/a_new.txt", "2\t/tmp/b.txt"];
        let result = parse_with_delete(&p, &lines).unwrap();
        assert_eq!(result.operations.len(), 1);
        assert_eq!(result.operations[0].source, PathBuf::from("/tmp/a.txt"));
        assert_eq!(result.operations[0].target, PathBuf::from("/tmp/a_new.txt"));
        assert!(result.deletions.is_empty());
    }

    #[test]
    fn with_delete_delete_file() {
        let p = paths(&["/tmp/a.txt", "/tmp/b.txt"]);
        let lines = vec!["1\t/tmp/a.txt"]; // b.txt line removed
        let result = parse_with_delete(&p, &lines).unwrap();
        assert!(result.operations.is_empty());
        assert_eq!(result.deletions, vec![PathBuf::from("/tmp/b.txt")]);
    }

    #[test]
    fn with_delete_invalid_index_error() {
        let p = paths(&["/tmp/a.txt"]);
        let lines = vec!["99\t/tmp/a.txt"]; // index out of range
        let err = parse_with_delete(&p, &lines).unwrap_err();
        assert_eq!(err.kind, ErrorKind::EditorLineCount);
    }

    #[test]
    fn with_delete_missing_tab_error() {
        let p = paths(&["/tmp/a.txt"]);
        let lines = vec!["/tmp/a.txt"]; // no tab separator
        let err = parse_with_delete(&p, &lines).unwrap_err();
        assert_eq!(err.kind, ErrorKind::EditorLineCount);
    }

    #[test]
    fn with_delete_duplicate_index_error() {
        let p = paths(&["/tmp/a.txt", "/tmp/b.txt"]);
        let lines = vec!["1\t/tmp/a.txt", "1\t/tmp/b_new.txt"]; // duplicate index 1
        let err = parse_with_delete(&p, &lines).unwrap_err();
        assert_eq!(err.kind, ErrorKind::EditorLineCount);
    }
}
