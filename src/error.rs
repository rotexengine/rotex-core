use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Fatal,
    Warning,
}

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("No compatible physical device found")]
    NoCompatibleDevice,

    #[error("Unsupported operation: {0}")]
    Unsupported(&'static str),

    #[error("Backend error: {0}")]
    Backend(String),
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub severity: Severity,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.severity, self.kind)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

impl Error {
    pub fn fatal(kind: ErrorKind) -> Self {
        Self {
            kind,
            severity: Severity::Fatal,
        }
    }

    pub fn warning(kind: ErrorKind) -> Self {
        Self {
            kind,
            severity: Severity::Warning,
        }
    }
}
