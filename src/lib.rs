//! # Cronet Rust Bindings
//!
//! Safe Rust bindings for Google's Cronet networking library.
//!
//! ## Features
//! - HTTP/1.1, HTTP/2, and HTTP/3 (QUIC) support
//! - Asynchronous requests with async/await
//! - Streaming responses for large downloads
//! - TLS and certificate handling
//! - Request/response interception
//!
//! ## Quick Start
//!
//! ### Using the reliable async wrapper (recommended)
//! ```rust
//! use cronet_rs::async_wrapper::AsyncRequester;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
//!
//!     match requester.fetch("https://httpbin.org/get").await {
//!         Ok((protocol, data)) => {
//!             println!("Success! Protocol: {}, Size: {} bytes", protocol, data.len());
//!         }
//!         Err(e) => {
//!             println!("Error: {}", e);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Using the original async request (DEPRECATED - has segmentation fault issues)
//! ⚠️ **Note**: The original `AsyncUrlRequest` has been removed from the codebase because it has known segmentation fault issues.
//! Please use `AsyncRequester` above instead.
//!     match request.send().await {
//!         Ok((protocol, data)) => {
//!             println!("Success! Protocol: {}, Size: {} bytes", protocol, data.len());
//!         }
//!         Err(e) => {
//!             println!("Error: {}", e);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Warning
//! The original `AsyncUrlRequest` implementation has been removed from the codebase due to known segmentation fault issues during cleanup.
//! It is recommended to use `async_wrapper::AsyncRequester` for reliable operation.
//!
//! ## Examples
//! See the `examples/` directory for complete working examples.
//!
//! ## Platform Support
//! - Linux (x86_64)
//! - Windows (x86_64) - planned
//! - macOS (x86_64, arm64) - planned
//!
//! ## License
//! MIT

pub mod async_wrapper;
pub mod error;
pub mod request;



use libloading::{Library, Symbol};
use std::ffi::{CString, c_void};
use std::os::raw::{c_char, c_int};
use std::sync::Arc;

use crate::error::{Error, Result};

use crate::request::Executor;

/// The core API context manager.
/// It holds the loaded dynamic library (`.dll` or `.so`).

pub type CallbackCreateWithFn = unsafe extern "C" fn(
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *const c_char),
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, u64),
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void),
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
) -> *mut c_void;


macro_rules! define_cronet_api {
    (
        $( $field:ident : $func_name:expr => $type:ty ),* $(,)?
    ) => {
        pub struct CronetApi {
            lib: Library,
            $( pub $field: $type, )*
        }

        impl CronetApi {
            pub fn new(path: &str) -> Result<Arc<Self>> {
                let lib = unsafe { Library::new(path)? };
                let api = Self {
                    $( $field: unsafe {
                        *lib.get::<$type>($func_name)
                            .map_err(|e| Error::SymbolNotFound(
                                format!("{}: {}", std::str::from_utf8($func_name).unwrap_or("<invalid>"), e)
                            ))?
                    }, )*
                    lib,
                };
                Ok(Arc::new(api))
            }

            /// Extract a symbol from the loaded Cronet shared library.
            ///
            /// # Safety
            ///
            /// This function is unsafe because dynamically loaded symbols may not match their expected type signatures.
            /// The caller must guarantee that the type `T` requested corresponds to the actual signature
            /// of the symbol exported by the Cronet DLL.
            pub unsafe fn get_sym<'a, T>(
                &'a self,
                name: &[u8],
            ) -> Result<Symbol<'a, T>> {
                unsafe { self.lib.get(name).map_err(Error::from) }
            }
        }
    };
}

define_cronet_api! {
    buffer_create: b"Cronet_Buffer_Create\0" => unsafe extern "C" fn() -> *mut c_void,
    buffer_destroy: b"Cronet_Buffer_Destroy\0" => unsafe extern "C" fn(*mut c_void),
    buffer_getdata: b"Cronet_Buffer_GetData\0" => unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    buffer_getsize: b"Cronet_Buffer_GetSize\0" => unsafe extern "C" fn(*mut c_void) -> u64,
    buffer_initwithalloc: b"Cronet_Buffer_InitWithAlloc\0" => unsafe extern "C" fn(*mut c_void, u64),
    engineparams_create: b"Cronet_EngineParams_Create\0" => unsafe extern "C" fn() -> *mut c_void,
    engineparams_destroy: b"Cronet_EngineParams_Destroy\0" => unsafe extern "C" fn(*mut c_void),
    engineparams_accept_language_set: b"Cronet_EngineParams_accept_language_set\0" => unsafe extern "C" fn(*mut c_void, *const c_char),
    engineparams_enable_brotli_set: b"Cronet_EngineParams_enable_brotli_set\0" => unsafe extern "C" fn(*mut c_void, bool),
    engineparams_enable_http2_set: b"Cronet_EngineParams_enable_http2_set\0" => unsafe extern "C" fn(*mut c_void, bool),
    engineparams_enable_quic_set: b"Cronet_EngineParams_enable_quic_set\0" => unsafe extern "C" fn(*mut c_void, bool),
    engineparams_http_cache_max_size_set: b"Cronet_EngineParams_http_cache_max_size_set\0" => unsafe extern "C" fn(*mut c_void, i64),
    engineparams_http_cache_mode_set: b"Cronet_EngineParams_http_cache_mode_set\0" => unsafe extern "C" fn(*mut c_void, i32),
    engineparams_quic_hints_add: b"Cronet_EngineParams_quic_hints_add\0" => unsafe extern "C" fn(*mut c_void, *mut c_void),
    engineparams_storage_path_set: b"Cronet_EngineParams_storage_path_set\0" => unsafe extern "C" fn(*mut c_void, *const c_char),
    engineparams_user_agent_set: b"Cronet_EngineParams_user_agent_set\0" => unsafe extern "C" fn(*mut c_void, *const c_char),
    engine_create: b"Cronet_Engine_Create\0" => unsafe extern "C" fn() -> *mut c_void,
    engine_destroy: b"Cronet_Engine_Destroy\0" => unsafe extern "C" fn(*mut c_void),
    engine_getversionstring: b"Cronet_Engine_GetVersionString\0" => unsafe extern "C" fn(*mut c_void) -> *const c_char,
    engine_startwithparams: b"Cronet_Engine_StartWithParams\0" => unsafe extern "C" fn(*mut c_void, *mut c_void) -> c_int,
    error_error_code_get: b"Cronet_Error_error_code_get\0" => unsafe extern "C" fn(*mut c_void) -> i32,
    error_message_get: b"Cronet_Error_message_get\0" => unsafe extern "C" fn(*mut c_void) -> *const c_char,
    executor_createwith: b"Cronet_Executor_CreateWith\0" => unsafe extern "C" fn(unsafe extern "C" fn(*mut c_void, *mut c_void)) -> *mut c_void,
    executor_destroy: b"Cronet_Executor_Destroy\0" => unsafe extern "C" fn(*mut c_void),
    executor_setclientcontext: b"Cronet_Executor_SetClientContext\0" => unsafe extern "C" fn(*mut c_void, *mut c_void),
    executor_getclientcontext: b"Cronet_Executor_GetClientContext\0" => unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    quichint_create: b"Cronet_QuicHint_Create\0" => unsafe extern "C" fn() -> *mut c_void,
    quichint_destroy: b"Cronet_QuicHint_Destroy\0" => unsafe extern "C" fn(*mut c_void),
    quichint_alternate_port_set: b"Cronet_QuicHint_alternate_port_set\0" => unsafe extern "C" fn(*mut c_void, i32),
    quichint_host_set: b"Cronet_QuicHint_host_set\0" => unsafe extern "C" fn(*mut c_void, *const c_char),
    quichint_port_set: b"Cronet_QuicHint_port_set\0" => unsafe extern "C" fn(*mut c_void, i32),
    runnable_destroy: b"Cronet_Runnable_Destroy\0" => unsafe extern "C" fn(*mut c_void),
    runnable_run: b"Cronet_Runnable_Run\0" => unsafe extern "C" fn(*mut c_void),
    urlrequestcallback_createwith: b"Cronet_UrlRequestCallback_CreateWith\0" => CallbackCreateWithFn,
    urlrequestcallback_destroy: b"Cronet_UrlRequestCallback_Destroy\0" => unsafe extern "C" fn(*mut c_void),
    urlrequestcallback_getclientcontext: b"Cronet_UrlRequestCallback_GetClientContext\0" => unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    urlrequestcallback_setclientcontext: b"Cronet_UrlRequestCallback_SetClientContext\0" => unsafe extern "C" fn(*mut c_void, *mut c_void),
    urlrequestparams_create: b"Cronet_UrlRequestParams_Create\0" => unsafe extern "C" fn() -> *mut c_void,
    urlrequestparams_destroy: b"Cronet_UrlRequestParams_Destroy\0" => unsafe extern "C" fn(*mut c_void),
    urlrequestparams_disable_cache_set: b"Cronet_UrlRequestParams_disable_cache_set\0" => unsafe extern "C" fn(*mut c_void, bool),
    urlrequestparams_http_method_set: b"Cronet_UrlRequestParams_http_method_set\0" => unsafe extern "C" fn(*mut c_void, *const c_char),
    urlrequestparams_priority_set: b"Cronet_UrlRequestParams_priority_set\0" => unsafe extern "C" fn(*mut c_void, i32),
    urlrequestparams_request_headers_add: b"Cronet_UrlRequestParams_request_headers_add\0" => unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char),
    urlrequestparams_upload_data_provider_set: b"Cronet_UrlRequestParams_upload_data_provider_set\0" => unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
    urlrequest_cancel: b"Cronet_UrlRequest_Cancel\0" => unsafe extern "C" fn(*mut c_void),
    urlrequest_create: b"Cronet_UrlRequest_Create\0" => unsafe extern "C" fn() -> *mut c_void,
    urlrequest_destroy: b"Cronet_UrlRequest_Destroy\0" => unsafe extern "C" fn(*mut c_void),
    urlrequest_followredirect: b"Cronet_UrlRequest_FollowRedirect\0" => unsafe extern "C" fn(*mut c_void),
    urlrequest_initwithparams: b"Cronet_UrlRequest_InitWithParams\0" => unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_char, *mut c_void, *mut c_void, *mut c_void) -> c_int,
    urlrequest_read: b"Cronet_UrlRequest_Read\0" => unsafe extern "C" fn(*mut c_void, *mut c_void),
    urlrequest_start: b"Cronet_UrlRequest_Start\0" => unsafe extern "C" fn(*mut c_void),
    urlresponseinfo_http_status_code_get: b"Cronet_UrlResponseInfo_http_status_code_get\0" => unsafe extern "C" fn(*mut c_void) -> i32,
    urlresponseinfo_http_status_text_get: b"Cronet_UrlResponseInfo_http_status_text_get\0" => unsafe extern "C" fn(*mut c_void) -> *const c_char,
    urlresponseinfo_negotiated_protocol_get: b"Cronet_UrlResponseInfo_negotiated_protocol_get\0" => unsafe extern "C" fn(*mut c_void) -> *const c_char,
}

/// Represents a hint for the Cronet engine to try QUIC/HTTP3 immediately on a given host/port.
pub struct QuicHint {
    api: Arc<CronetApi>,
    ptr: *mut c_void,
}

impl QuicHint {
    /// Create a new QUIC hint for the specified host and ports.
    pub fn new(
        api: Arc<CronetApi>,
        host: &str,
        port: i32,
        alternate_port: i32,
    ) -> Result<Self> {
        let ptr = unsafe {
            let func = api.quichint_create;
            func()
        };

        let c_host = CString::new(host)?;
        unsafe {
            let set_host = api.quichint_host_set;
            set_host(ptr, c_host.as_ptr());

            let set_port = api.quichint_port_set;
            set_port(ptr, port);

            let set_alt_port = api.quichint_alternate_port_set;
            set_alt_port(ptr, alternate_port);
        }

        Ok(Self { api, ptr })
    }
}

impl Drop for QuicHint {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                if let Ok(func) = self
                    .api
                    .get_sym::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_QuicHint_Destroy")
                {
                    func(self.ptr);
                }
                self.ptr = std::ptr::null_mut();
            }
        }
    }
}

/// Configuration parameters for initializing a Cronet engine.
pub struct EngineParams {
    api: Arc<CronetApi>,
    ptr: *mut c_void,
}

impl EngineParams {
    /// Create new default Engine Parameters.
    pub fn new(api: Arc<CronetApi>) -> Result<Self> {
        let ptr = unsafe {
            let func = api.engineparams_create;
            func()
        };
        Ok(Self { api, ptr })
    }

    /// Set whether QUIC (HTTP/3) is enabled globally.
    pub fn set_enable_quic(&self, enable: bool) -> Result<()> {
        unsafe {
            if let Ok(func) = self.api.get_sym::<unsafe extern "C" fn(*mut c_void, bool)>(b"Cronet_EngineParams_enable_quic_set") {
                func(self.ptr, enable);
            }
        }
        Ok(())
    }

    /// Set the User-Agent header value for all requests going through this engine.
    pub fn set_user_agent(&self, user_agent: &str) -> Result<()> {
        let c_str = CString::new(user_agent)?;
        unsafe {
            if let Ok(func) = self.api.get_sym::<unsafe extern "C" fn(*mut c_void, *const c_char)>(b"Cronet_EngineParams_user_agent_set") {
                func(self.ptr, c_str.as_ptr());
            }
        }
        Ok(())
    }

    /// Add a QUIC hint to pre-warm the engine's capability to use HTTP/3 for a specific host.
    pub fn add_quic_hint(&self, hint: &QuicHint) -> Result<()> {
        unsafe {
            if let Ok(func) = self.api.get_sym::<unsafe extern "C" fn(*mut c_void, *mut c_void)>(b"Cronet_EngineParams_quic_hints_add") {
                func(self.ptr, hint.ptr);
            }
        }
        Ok(())
    }

    /// Enable or disable Brotli compression support
    pub fn set_enable_brotli(&self, enable: bool) -> Result<()> {
        unsafe {
            if let Ok(func) = self.api.get_sym::<unsafe extern "C" fn(*mut c_void, bool)>(b"Cronet_EngineParams_enable_brotli_set") {
                func(self.ptr, enable);
            }
        }
        Ok(())
    }

    /// Enable or disable HTTP/2 support
    pub fn set_enable_http2(&self, enable: bool) -> Result<()> {
        unsafe {
            if let Ok(func) = self.api.get_sym::<unsafe extern "C" fn(*mut c_void, bool)>(b"Cronet_EngineParams_enable_http2_set") {
                func(self.ptr, enable);
            }
        }
        Ok(())
    }

    /// Set HTTP cache mode
    pub fn set_http_cache_mode(&self, mode: i32) -> Result<()> {
        unsafe {
            if let Ok(func) = self.api.get_sym::<unsafe extern "C" fn(*mut c_void, i32)>(b"Cronet_EngineParams_http_cache_mode_set") {
                func(self.ptr, mode);
            }
        }
        Ok(())
    }

    /// Set maximum HTTP cache size in bytes
    pub fn set_http_cache_max_size(&self, size: i64) -> Result<()> {
        unsafe {
            if let Ok(func) = self.api.get_sym::<unsafe extern "C" fn(*mut c_void, i64)>(b"Cronet_EngineParams_http_cache_max_size_set") {
                func(self.ptr, size);
            }
        }
        Ok(())
    }

    /// Set Accept-Language header value
    pub fn set_accept_language(&self, language: &str) -> Result<()> {
        let c_str = CString::new(language)?;
        unsafe {
            if let Ok(func) = self.api.get_sym::<unsafe extern "C" fn(*mut c_void, *const c_char)>(b"Cronet_EngineParams_accept_language_set") {
                func(self.ptr, c_str.as_ptr());
            }
        }
        Ok(())
    }

    /// Set storage path for HTTP cache and other persisted data
    pub fn set_storage_path(&self, path: &str) -> Result<()> {
        let c_str = CString::new(path)?;
        unsafe {
            if let Ok(func) = self.api.get_sym::<unsafe extern "C" fn(*mut c_void, *const c_char)>(b"Cronet_EngineParams_storage_path_set") {
                func(self.ptr, c_str.as_ptr());
            }
        }
        Ok(())
    }
}

impl Drop for EngineParams {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                if let Ok(func) = self
                    .api
                    .get_sym::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_EngineParams_Destroy")
                {
                    func(self.ptr);
                }
                self.ptr = std::ptr::null_mut();
            }
        }
    }
}

/// The central component that executes URL requests.
#[derive(Clone)]
pub struct Engine {
    pub api: Arc<CronetApi>,
    pub ptr: *mut c_void,
}

impl Engine {
    /// Create a new instance of the Engine.
    pub fn new(api: Arc<CronetApi>) -> Result<Self> {
        let ptr = unsafe {
            if let Ok(func) = api.get_sym::<unsafe extern "C" fn() -> *mut c_void>(b"Cronet_Engine_Create") {
                func()
            } else {
                return Err(Error::SymbolNotFound("Cronet_Engine_Create".to_string()));
            }
        };
        Ok(Self { api, ptr })
    }

    /// Initialize and start the engine using the given EngineParams.
    pub fn start_with_params(&self, params: &EngineParams) -> Result<c_int> {
        unsafe {
            if let Ok(func) = self.api.get_sym::<unsafe extern "C" fn(*mut c_void, *mut c_void) -> c_int>(b"Cronet_Engine_StartWithParams") {
                Ok(func(self.ptr, params.ptr))
            } else {
                Err(Error::SymbolNotFound("Cronet_Engine_StartWithParams".to_string()))
            }
        }
    }

    /// Retrieve the version string of the loaded Cronet engine.
    pub fn get_version_string(&self) -> Result<String> {
        unsafe {
            if let Ok(func) = self.api.get_sym::<unsafe extern "C" fn(*mut c_void) -> *const c_char>(b"Cronet_Engine_GetVersionString") {
                let ptr = func(self.ptr);
                crate::error::cstr_to_string(ptr)
            } else {
                Err(Error::SymbolNotFound("Cronet_Engine_GetVersionString".to_string()))
            }
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            if let Ok(func) = self
                .api
                .get_sym::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_Engine_Destroy")
            {
                func(self.ptr);
            }
        }
    }
}

/// HTTP client that manages Cronet engine and executor lifecycle.
/// This provides a higher-level, more convenient API for making HTTP requests.
pub struct Client {
    api: Arc<CronetApi>,
    executor: Executor,  // Dropped before engine
    engine: Engine,      // Dropped after executor
}

impl Client {
    /// Create a new Client with default configuration.
    pub fn new(dll_path: &str) -> Result<Self> {
        let api = CronetApi::new(dll_path)?;

        // Default params
        let params = EngineParams::new(api.clone())?;
        params.set_enable_quic(true)?;
        params.set_user_agent(&format!("cronet-rs/{}", env!("CARGO_PKG_VERSION")))?;

        let engine = Engine::new(api.clone())?;
        engine.start_with_params(&params)?;

        let executor = Executor::new(api.clone())?;

        // Note: executor is declared before engine so it drops first
        Ok(Self { api, executor, engine })
    }

    /// Create a new Client with custom configuration.
    pub fn with_config<F>(dll_path: &str, config_fn: F) -> Result<Self>
    where
        F: FnOnce(&EngineParams) -> Result<()>,
    {
        let api = CronetApi::new(dll_path)?;
        let params = EngineParams::new(api.clone())?;
        config_fn(&params)?;

        let engine = Engine::new(api.clone())?;
        engine.start_with_params(&params)?;

        let executor = Executor::new(api.clone())?;

        // Note: executor is declared before engine so it drops first
        Ok(Self { api, executor, engine })
    }

    /// Get a reference to the underlying engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Get a reference to the underlying executor.
    pub fn executor(&self) -> &Executor {
        &self.executor
    }




    /// Execute an asynchronous request with retry logic.
    ///
    /// ⚠️ **DEPRECATED** - This method depends on the removed `fetch` method.
    /// Please use `async_wrapper::AsyncRequester` and implement retry logic manually.
    #[deprecated(since = "0.1.0", note = "This method depends on the removed fetch() method. Use async_wrapper::AsyncRequester and implement retry logic manually.")]
    pub async fn fetch_with_retry(&self, _url: &str, _max_retries: u32) -> Result<(String, Vec<u8>)> {
        Err(Error::Other("fetch_with_retry is deprecated. Use async_wrapper::AsyncRequester instead.".to_string()))
    }
    
    /// Execute an asynchronous request with timeout.
    ///
    /// ⚠️ **DEPRECATED** - This method depends on the removed `fetch` method.
    /// Please use `async_wrapper::AsyncRequester` and implement timeout logic manually.
    #[deprecated(since = "0.1.0", note = "This method depends on the removed fetch() method. Use async_wrapper::AsyncRequester and implement timeout logic manually.")]
    pub async fn fetch_with_timeout(&self, _url: &str, _timeout: std::time::Duration) -> Result<(String, Vec<u8>)> {
        Err(Error::Other("fetch_with_timeout is deprecated. Use async_wrapper::AsyncRequester instead.".to_string()))
    }
}
