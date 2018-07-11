use ansi_term::Colour::*;
use args::Config;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;
use walkdir::WalkDir;

pub struct Renamer {
    files: Vec<String>,
    config: Arc<Config>,
}

impl Renamer {
    pub fn new(config: &Arc<Config>) -> Renamer {
        let input_files = get_files(&config);
        Renamer {
            files: input_files,
            config: config.clone(),
        }
    }

    /// Process file batch
    pub fn process(&mut self) {
        self.cleanup();

        for file in &self.files {
            let target = self.replace_match(file);
            if target != *file {
                self.rename(file, &target);
            }
        }
    }

    /// Clean files that does not exists, broken links and directories
    fn cleanup(&mut self) {
        self.files.retain(|file| {
            if !Path::new(&file).exists() {
                // Checks if non-existing path is actually a symlink
                match fs::read_link(&file) {
                    Ok(_) => true,
                    Err(_) => {
                        eprintln!(
                            "{}File '{}' is not accesible",
                            Yellow.paint("Warn: "),
                            Yellow.paint(file.as_str())
                        );
                        false
                    }
                }
            } else {
                !fs::metadata(&file).unwrap().is_dir()
            }
        });
    }

    /// Replace expression match in the given file using stored config.
    fn replace_match(&self, file: &str) -> String {
        let expression = &self.config.expression;
        let replacement = &self.config.replacement;

        let file_name = Path::new(&file).file_name().unwrap().to_str().unwrap();
        let file_path = Path::new(&file).parent().unwrap().to_str().unwrap();

        let target_name = expression.replace(file_name, &replacement[..]);
        match file_path {
            "" => String::from(target_name),
            _ => format!("{}/{}", file_path, target_name),
        }
    }

    /// Rename file in the filesystem or simply print renaming information. Checks if target
    /// filename exists before renaming.
    fn rename(&self, file: &str, target: &str) {
        if Path::new(&target).exists() {
            eprintln!(
                "{}File already exists - {}",
                Red.paint("Error: "),
                Red.paint(format!("{} -> {}", file, target))
            );
        } else if self.config.force {
            if self.config.backup {
                self.backup(file);
            }

            if fs::rename(&file, &target).is_err() {
                eprintln!(
                    "{}File {} renaming failed.",
                    Red.paint("Error: "),
                    Red.paint(file)
                );
            } else {
                println!("{} -> {}", Blue.paint(file), Green.paint(target),);
            }
        } else {
            println!("{} -> {}", Blue.paint(file), Green.paint(target),);
        }
    }

    /// Create a backup of the file
    fn backup(&self, file: &str) {
        let backup_name = format!("{}.bk", file);
        let mut backup = backup_name.clone();
        let mut backup_index = 0;

        while Path::new(&backup).exists() {
            backup_index += 1;
            backup = format!("{}.{}", backup_name, backup_index);
        }

        println!(
            "{} Creating a backup - {}",
            Blue.paint("Info: "),
            Blue.paint(format!("{} -> {}", file, backup))
        );

        if fs::copy(file, backup).is_err() {
            eprintln!("{}File backup failed.", Red.paint("Error: "));
            process::exit(1);
        }
    }
}

/// Return a list of files for the given configuration.
fn get_files(config: &Config) -> Vec<String> {
    if config.recursive.active {
        // Get recursive list of files walking directories
        let path = &config.recursive.path;
        let walkdir = match config.recursive.max_depth {
            Some(max_depth) => WalkDir::new(path).max_depth(max_depth),
            None => WalkDir::new(path),
        };
        walkdir
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|x| match x.path().to_str() {
                Some(s) => Some(s.to_string()),
                None => {
                    eprintln!(
                        "{}File '{}' contains invalid characters",
                        Yellow.paint("Warn: "),
                        Yellow.paint(x.path().to_string_lossy())
                    );
                    None
                }
            })
            .collect()
    } else {
        // Get file list directly from argument list
        config.file_args.clone().unwrap()
    }
}

#[cfg(test)]
mod test {
    extern crate tempfile;
    use super::Renamer;
    use args::*;
    use regex::Regex;
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;

    #[test]
    fn renamer_test() {
        // Create and set a new temporal directory
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        env::set_current_dir(&tempdir).expect("Error changing to temp directory");

        // Generate a mock configuration
        let mock_files: Vec<String> = vec![
            "test_file_1.txt".to_string(),
            "mock_dir/test_file_1.txt".to_string(),
            "mock_dir/test_file_2.txt".to_string(),
        ];
        // Generate files
        fs::create_dir("mock_dir").expect("Error creating mock directory...");
        for file in &mock_files {
            fs::File::create(file).expect("Error creating mock file...");
        }

        // Create config
        let mock_config = Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: true,
            backup: true,
            recursive: RecursiveMode {
                active: false,
                path: "./".to_string(),
                max_depth: None,
            },
            file_args: Some(mock_files),
        };

        // Run renamer
        let mut renamer = Renamer::new(&Arc::new(mock_config));
        renamer.process();

        assert!(Path::new("passed_file_1.txt").exists());
        assert!(Path::new("mock_dir/passed_file_1.txt").exists());
        assert!(Path::new("mock_dir/passed_file_2.txt").exists());

        assert!(Path::new("test_file_1.txt.bk").exists());
        assert!(Path::new("mock_dir/test_file_1.txt.bk").exists());
        assert!(Path::new("mock_dir/test_file_2.txt.bk").exists());
    }
}
