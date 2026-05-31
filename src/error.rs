use thiserror::Error;

/// How severely an error affects continued operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// The caller must stop or recover before proceeding.
    Fatal,
    /// The caller may continue with degraded or partial results.
    Warning,
}

/// Specific failure reported by the graphics backend.
#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("No compatible physical device found")]
    NoCompatibleDevice,

    #[error("Unsupported operation: {0}")]
    Unsupported(&'static str),

    #[error("Backend error: {0}")]
    Backend(String),
}

/// Backend error paired with its [`Severity`].
#[derive(Debug)]
pub struct Error {
    /// The underlying failure.
    pub kind: ErrorKind,
    /// Whether the failure is fatal or recoverable.
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
    /// Constructs an error with [`Severity::Fatal`].
    pub fn fatal(kind: ErrorKind) -> Self {
        Self {
            kind,
            severity: Severity::Fatal,
        }
    }

    /// Constructs an error with [`Severity::Warning`].
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

