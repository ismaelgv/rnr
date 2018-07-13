use ansi_term::Colour::*;
use args::Config;
use args::RunMode;
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
                            "{}File '{}' is not accessible",
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
                create_backup(file);
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

}

/// Return a list of files for the given configuration.
fn get_files(config: &Config) -> Vec<String> {
    match &config.mode {
        RunMode::Recursive { path, max_depth } => {
        // Get recursive list of files walking directories
        let walkdir = match max_depth {
            Some(max_depth) => WalkDir::new(path).max_depth(*max_depth),
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
    },
        RunMode::FileList(file_list) =>
        file_list.clone()
    }
}

/// Generate a non-existing name adding numbers to the end of the file. It also supports adding a
/// suffix to the original name.
fn get_unique_filename(file: &str, suffix: &str) -> String {
    let base_name = format!("{}{}", file,  suffix);
    let mut unique_name = base_name.clone();
    let mut index = 0;

    while Path::new(&unique_name).exists() {
        index += 1;
        unique_name = format!("{}.{}", base_name, index);
    }

    unique_name.to_string()
}

/// Create a backup of the file
fn create_backup(file: &str) {
    let backup = get_unique_filename(file, ".bk");
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

    /// Create and set a new temporal directory
    fn set_temp_directory() -> tempfile::TempDir {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        env::set_current_dir(&tempdir).expect("Error changing to temp directory");
        println!("Running test in '{:?}'", tempdir);

        tempdir
    }

    #[test]
    fn renamer_test() {
        let _tempdir = set_temp_directory();

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
            mode: RunMode::FileList(mock_files),
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

    #[test]
    fn backup_test() {
        let _tempdir = set_temp_directory();

        let mock_files: Vec<String> = vec![
            "test_file_1.txt".to_string(),
            "test_file_2.txt".to_string(),
            "test_file_3.txt".to_string(),
        ];

        for file in &mock_files {
            fs::File::create(file).expect("Error creating mock file...");
            super::create_backup(file);
        }

        assert!(Path::new("test_file_1.txt.bk").exists());
        assert!(Path::new("test_file_2.txt.bk").exists());
        assert!(Path::new("test_file_3.txt.bk").exists());
    }
}
