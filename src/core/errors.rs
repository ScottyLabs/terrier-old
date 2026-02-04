#[cfg(feature = "server")]
use dioxus::prelude::ServerFnError;

/// Extension trait for Result types to provide convenient error conversion to ServerFnError
#[cfg(feature = "server")]
pub trait ResultExt<T> {
    /// Convert a Result to a ServerFnError with context
    fn to_server_error(self, context: &str) -> Result<T, ServerFnError>;
}

#[cfg(feature = "server")]
impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn to_server_error(self, context: &str) -> Result<T, ServerFnError> {
        self.map_err(|e| ServerFnError::new(format!("{}: {}", context, e)))
    }
}

/// Extension trait for Option types to provide convenient error conversion
#[cfg(feature = "server")]
pub trait OptionExt<T> {
    /// Convert an Option to a Result with a ServerFnError
    fn ok_or_server_error(self, message: &str) -> Result<T, ServerFnError>;
}

#[cfg(feature = "server")]
impl<T> OptionExt<T> for Option<T> {
    fn ok_or_server_error(self, message: &str) -> Result<T, ServerFnError> {
        self.ok_or_else(|| ServerFnError::new(message))
    }
}
