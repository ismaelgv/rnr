use std::result;

pub type Result<T> = result::Result<T, Error>;

/// Errors during app execution
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// Define the type of error
    pub kind: ErrorKind,
    /// Value associated to the error
    pub value: Option<String>,
}

/// Define type of error
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    CreateBackup,
    ExistingFile,
    RenameFile,
    SameFilename,
    SolveOrder,
}

impl Error {
    pub fn description(&self) -> &str {
        use self::ErrorKind::*;
        match self.kind {
            CreateBackup => "Cannot create a backup of file ",
            ExistingFile => "Conflict with existing file ",
            RenameFile => "Cannot rename file ",
            SameFilename => "Files will have the same name\n",
            SolveOrder => "Cannot solve sorting problem.",
        }
    }
}
