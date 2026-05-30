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

#[cfg(not(target_arch = "wasm32"))]
impl From<rotex_vulkan::Error> for Error {
    fn from(value: rotex_vulkan::Error) -> Self {
        let severity = match value.severity {
            rotex_vulkan::Severity::Fatal => Severity::Fatal,
            rotex_vulkan::Severity::Info
            | rotex_vulkan::Severity::Warning
            | rotex_vulkan::Severity::Recoverable => Severity::Warning,
        };
        let kind = match value.kind {
            rotex_vulkan::ErrorKind::NoCompatibleDevice => ErrorKind::NoCompatibleDevice,
            rotex_vulkan::ErrorKind::Unsupported(message) => ErrorKind::Unsupported(message),
            rotex_vulkan::ErrorKind::Vulkan(code) => {
                ErrorKind::Backend(format!("Vulkan error: {code:?} ({})", code.as_raw()))
            }
        };
        Self { kind, severity }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<rotex_wgpu::Error> for Error {
    fn from(value: rotex_wgpu::Error) -> Self {
        let severity = match value.severity {
            rotex_wgpu::Severity::Fatal => Severity::Fatal,
            rotex_wgpu::Severity::Info
            | rotex_wgpu::Severity::Warning
            | rotex_wgpu::Severity::Recoverable => Severity::Warning,
        };
        let kind = match value.kind {
            rotex_wgpu::ErrorKind::NoCompatibleDevice => ErrorKind::NoCompatibleDevice,
            rotex_wgpu::ErrorKind::Unsupported(message) => ErrorKind::Unsupported(message),
            other => ErrorKind::Backend(format!("{other:?}")),
        };
        Self { kind, severity }
    }
}

