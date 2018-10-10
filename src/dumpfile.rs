use chrono;
use error::*;
use fileutils::PathList;
use serde_json;
use solver::RenameMap;
use std::fs::File;

pub fn dump_to_file(rename_order: &PathList, rename_map: &RenameMap) -> Result<()> {
    // Get all operations in order
    let mut operations: Vec<Operation> = Vec::new();
    for target in rename_order {
        let source = &rename_map[target];
        let operation = Operation {
            source: source.to_string_lossy().to_string(),
            target: target.to_string_lossy().to_string(),
        };
        operations.push(operation);
    }

    let now = chrono::Local::now();
    let dump = DumpFormat {
        date: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        operations: operations,
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

// This struct stores required information about a rename operation
#[derive(Serialize, Deserialize)]
struct Operation {
    source: String,
    target: String,
}

#[derive(Serialize, Deserialize)]
struct DumpFormat {
    date: String,
    operations: Vec<Operation>,
}
