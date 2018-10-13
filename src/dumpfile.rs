use chrono;
use error::*;
use serde_json;
use solver::Operations;
use std::fs::File;

pub fn dump_to_file(operations: &Operations) -> Result<()> {
    let now = chrono::Local::now();
    let dump = DumpFormat {
        date: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        operations: operations.clone(),
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

#[derive(Serialize, Deserialize)]
struct DumpFormat {
    date: String,
    operations: Operations,
}
