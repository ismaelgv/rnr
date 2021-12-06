use ansi_term::Colour::*;
use ansi_term::Style;
use difference::{Changeset, Difference};
use error::*;
use std::path::Path;

#[derive(PartialEq)]
enum PrinterMode {
    Silent,
    NoColor,
    Color,
}

pub struct Printer {
    pub colors: Colors,
    mode: PrinterMode,
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
    pub fn color() -> Printer {
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
            mode: PrinterMode::Color,
        }
    }

    /// Return a printer configured to not use colors
    pub fn no_color() -> Printer {
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
            mode: PrinterMode::NoColor,
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
            mode: PrinterMode::Silent,
        }
    }

    /// Print string to Stdout when printer is not in silent mode
    pub fn print(&self, message: &str) {
        match self.mode {
            PrinterMode::Color | PrinterMode::NoColor => {
                println!("{}", message);
            }
            PrinterMode::Silent => {}
        }
    }

    /// Print string to Stderr when printer is not in silent mode
    pub fn eprint(&self, message: &str) {
        match self.mode {
            PrinterMode::Color | PrinterMode::NoColor => {
                eprintln!("{}", message);
            }
            PrinterMode::Silent => {}
        }
    }

    /// Print error pretty printed
    pub fn print_error(&self, error: &Error) {
        let error_value = error.value.to_owned().unwrap_or_else(|| String::from(""));

        self.eprint(&format!(
            "{}{}{}",
            self.colors.error.paint("Error: "),
            error.description(),
            self.colors.error.paint(error_value)
        ));
    }

    /// Pretty print operation
    pub fn print_operation(&self, source: &Path, target: &Path) {
        // Avoid any additional processing costs if silent mode
        if self.mode == PrinterMode::Silent {
            return;
        }

        let mut source_parent = source.parent().unwrap().to_string_lossy().to_string();
        let mut source_name = source.file_name().unwrap().to_string_lossy().to_string();
        let mut target_parent = target.parent().unwrap().to_string_lossy().to_string();
        let mut target_name = target.file_name().unwrap().to_string_lossy().to_string();

        // Avoid diffing if not coloring output
        if self.mode == PrinterMode::Color {
            target_name = self.string_diff(
                &source_name,
                &target_name,
                self.colors.target,
                self.colors.highlight,
            )
        }

        source_name = self.colors.source.paint(&source_name).to_string();

        if !source_parent.is_empty() {
            source_parent = self
                .colors
                .source
                .paint(format!("{}/", source_parent))
                .to_string();
        }
        if !target_parent.is_empty() {
            target_parent = self
                .colors
                .target
                .paint(format!("{}/", target_parent))
                .to_string();
        }

        self.print(&format!(
            "{}{} -> {}{}",
            source_parent, source_name, target_parent, target_name
        ));
    }

    /// Generate a colored diff from the given strings
    fn string_diff(
        &self,
        original: &str,
        changed: &str,
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
