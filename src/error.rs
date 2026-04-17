use std::ffi::{CStr, CString, NulError};
use std::fmt;
use std::io;

use libloading;

/// Unified error type for all cronet-rs operations.
#[derive(Debug)]
pub enum Error {
    /// Failed to load the Cronet dynamic library.
    LibLoading(libloading::Error),

    /// A required symbol was not found in the loaded library.
    SymbolNotFound(String),

    /// A C API returned a null pointer unexpectedly.
    NullPointer(&'static str),

    /// Failed to convert between Rust and C strings.
    Utf8Error(std::str::Utf8Error),

    /// Failed to create a C string (contains null byte).
    NulError(NulError),

    /// Error returned by the Cronet C API.
    CronetApi { error_code: i32, message: String },

    /// Invalid argument passed to a function.
    InvalidArgument(String),

    /// I/O error.
    IoError(io::Error),

    /// Other error.
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::LibLoading(e) => write!(f, "Failed to load Cronet library: {}", e),
            Error::SymbolNotFound(sym) => write!(f, "Symbol not found: {}", sym),
            Error::NullPointer(context) => write!(f, "Null pointer returned: {}", context),
            Error::Utf8Error(e) => write!(f, "UTF-8 conversion error: {}", e),
            Error::NulError(e) => write!(f, "C string conversion error: {}", e),
            Error::CronetApi {
                error_code,
                message,
            } => {
                write!(f, "Cronet API error (code {}): {}", error_code, message)
            }
            Error::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            Error::IoError(e) => write!(f, "I/O error: {}", e),
            Error::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::LibLoading(e) => Some(e),
            Error::Utf8Error(e) => Some(e),
            Error::NulError(e) => Some(e),
            Error::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<libloading::Error> for Error {
    fn from(err: libloading::Error) -> Self {
        Error::LibLoading(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error::Utf8Error(err)
    }
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Self {
        Error::NulError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<crate::request::CronetError> for Error {
    fn from(err: crate::request::CronetError) -> Self {
        match (err.error_code(), err.message()) {
            (Ok(code), Ok(msg)) => Error::CronetApi {
                error_code: code,
                message: msg,
            },
            _ => Error::Other("Failed to get error details from Cronet".to_string()),
        }
    }
}

/// Convenience type alias for results using the cronet-rs error type.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Get the error code if this is a CronetApi error.
    pub fn error_code(&self) -> Option<i32> {
        match self {
            Error::CronetApi { error_code, .. } => Some(*error_code),
            _ => None,
        }
    }

    /// Get the error message if this is a CronetApi error.
    pub fn message(&self) -> Option<&str> {
        match self {
            Error::CronetApi { message, .. } => Some(message),
            _ => None,
        }
    }
}

/// Helper function to convert a null-terminated C string to a Rust String.
pub unsafe fn cstr_to_string(ptr: *const std::os::raw::c_char) -> Result<String> {
    if ptr.is_null() {
        return Ok(String::new());
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    Ok(cstr.to_string_lossy().into_owned())
}

/// Helper function to convert a Rust string to a CString with proper error handling.
pub fn string_to_cstring(s: &str) -> Result<CString> {
    CString::new(s).map_err(Error::NulError)
}
