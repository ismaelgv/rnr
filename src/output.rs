use ansi_term::Colour::*;
use ansi_term::Style;
use difference::{Changeset, Difference};
use error::*;
use std::path::PathBuf;

pub struct Printer {
    pub colors: Colors,
    silent: bool,
}

pub struct Colors {
    pub info: Style,
    pub warn: Style,
    pub error: Style,
    pub source: Style,
    pub target: Style,
    pub highlight: Style,
}

impl Printer {
    /// Return a printer configured to colorize output
    pub fn colored() -> Printer {
        let colors = Colors {
            info: Style::default().bold(),
            warn: Style::from(Yellow),
            error: Style::from(Red),
            source: Style::from(Fixed(8)), //Dark grey
            target: Style::from(Green),
            highlight: Style::from(Red).bold(),
        };

        Printer {
            colors,
            silent: false,
        }
    }

    /// Return a printer configured to not use colors
    pub fn no_colored() -> Printer {
        let colors = Colors {
            info: Style::default(),
            warn: Style::default(),
            error: Style::default(),
            source: Style::default(),
            target: Style::default(),
            highlight: Style::default(),
        };

        Printer {
            colors,
            silent: false,
        }
    }

    /// Return a printer configured to be in silent mode
    pub fn silent() -> Printer {
        let colors = Colors {
            info: Style::default(),
            warn: Style::default(),
            error: Style::default(),
            source: Style::default(),
            target: Style::default(),
            highlight: Style::default(),
        };

        Printer {
            colors,
            silent: true,
        }
    }

    /// Print string to Stdout when printer is not in silent mode
    pub fn print(&self, message: &str) {
        if !self.silent {
            println!("{}", message);
        }
    }

    /// Print string to Stderr when printer is not in silent mode
    pub fn eprint(&self, message: &str) {
        if !self.silent {
            eprintln!("{}", message);
        }
    }

    /// Print error pretty printed
    pub fn print_error(&self, error: &Error) {
        let error_value = error.value.to_owned().unwrap_or("".to_string());

        self.eprint(&format!(
            "{}{}{}",
            self.colors.error.paint("Error: "),
            error.description(),
            self.colors.error.paint(error_value)
        ));
    }

    /// Pretty print operation
    pub fn print_operation(&self, source: &PathBuf, target: &PathBuf) {
        // Avoid any additional processing costs in silent mode
        if self.silent {
            return;
        }

        let source_parent = source.parent().unwrap().to_string_lossy().to_string();
        let source_name = source.file_name().unwrap().to_string_lossy().to_string();
        let target_parent = target.parent().unwrap().to_string_lossy().to_string();
        let target_name = target.file_name().unwrap().to_string_lossy().to_string();

        self.print(&format!(
            "{}{} -> {}{}",
            self.colors.source.paint(format!("{}/", source_parent)),
            self.colors.source.paint(&source_name),
            self.colors.target.paint(format!("{}/", target_parent)),
            self.string_diff(
                &source_name,
                &target_name,
                self.colors.target,
                self.colors.highlight
            ),
        ));
    }

    /// Generate a colored diff from the given strings
    fn string_diff(
        &self,
        original: &String,
        changed: &String,
        base_color: Style,
        diff_color: Style,
    ) -> String {
        let mut colored_string = String::new();
        let changetset = Changeset::new(original, changed, "");
        for difference in changetset.diffs {
            match difference {
                Difference::Same(string) => {
                    colored_string = format!("{}{}", colored_string, base_color.paint(string))
                }
                Difference::Add(string) => {
                    colored_string = format!("{}{}", colored_string, diff_color.paint(string))
                }
                Difference::Rem(_) => continue,
            }
        }
        colored_string
    }
}
