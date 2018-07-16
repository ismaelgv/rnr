use ansi_term::Colour::*;
use ansi_term::Style;

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
}

impl Printer {
    /// Return a printer configured to colorize output
    pub fn colored() -> Printer {
        let colors = Colors {
            info: White.bold(),
            warn: Style::from(Yellow),
            error: Style::from(Red),
            source: Style::from(Blue),
            target: Style::from(Green),
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
}
