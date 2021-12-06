use chrono;
use error::*;
use serde_json;
use solver::{Operation, Operations};
use std::fs::File;
use std::path::Path;

/// Dump operations intto file in JSON format
pub fn dump_to_file(operations: &[Operation]) -> Result<()> {
    let now = chrono::Local::now();
    let dump = DumpFormat {
        date: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        operations: operations.to_vec(),
    };

    // Create filename with the following syntax: "rnr-<DATE>.json"
    let filename = "rnr-".to_string() + &now.format("%Y-%m-%d_%H%M%S").to_string() + ".json";

    // Dump info to a file
    let file = match File::create(&filename) {
        Ok(file) => file,
        Err(_) => {
            return Err(Error {
                kind: ErrorKind::CreateFile,
                value: Some(filename),
            })
        }
    };
    match serde_json::to_writer_pretty(file, &dump) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error {
            kind: ErrorKind::JsonParse,
            value: Some(filename),
        }),
    }
}

/// Read operations from a dump file and generate a Operations vector
pub fn read_from_file(filepath: &Path) -> Result<Operations> {
    let file = match File::open(&filepath) {
        Ok(file) => file,
        Err(_) => {
            return Err(Error {
                kind: ErrorKind::ReadFile,
                value: Some(filepath.to_string_lossy().to_string()),
            })
        }
    };
    let dump: DumpFormat = match serde_json::from_reader(file) {
        Ok(dump) => dump,
        Err(_) => {
            return Err(Error {
                kind: ErrorKind::JsonParse,
                value: Some(filepath.to_string_lossy().to_string()),
            })
        }
    };
    Ok(dump.operations)
}

#[derive(Serialize, Deserialize)]
struct DumpFormat {
    date: String,
    operations: Operations,
}
