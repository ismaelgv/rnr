use ansi_term::Colour::*;
use ansi_term::Style;

pub struct Printer {
    pub colors: Colors,
    silent: bool,
}

pub struct Colors {
    info: Style,
    warn: Style,
    error: Style,
    source: Style,
    target: Style,
}

impl Printer {
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

    fn print(&self, message: &str) {
        if !self.silent {
            println!("{}", message);
        }
    }

    fn eprint(&self, message: &str) {
        if !self.silent {
            eprintln!("{}", message);
        }
    }
}
