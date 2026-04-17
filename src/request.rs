use crate::error::{Error, Result};
use crate::CronetApi;
use crate::Engine;
use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;

static GET_CALLBACK_CTX_FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static GET_EXECUTOR_CTX_FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

pub struct CallbackContext {
    pub api: Arc<CronetApi>,
    pub handler: Arc<dyn UrlRequestCallbackHandler>,
}

struct CallbackInner {
    api: Arc<CronetApi>,
    ptr: *mut c_void,
    ctx_raw: *mut c_void,
}

pub struct ExecutorContext {
    pub api: Arc<CronetApi>,
    pub handle: Option<tokio::runtime::Handle>,
}

struct ExecutorInner {
    api: Arc<CronetApi>,
    ptr: *mut c_void,
    ctx_raw: *mut c_void,
}

#[derive(Clone)]
pub struct Executor {
    inner: Arc<ExecutorInner>,
}

impl Executor {
    pub fn new(api: Arc<CronetApi>) -> Result<Self> {
        let ctx = Arc::new(ExecutorContext {
            api: api.clone(),
            handle: tokio::runtime::Handle::try_current().ok(),
        });
        let ctx_raw = Arc::into_raw(ctx) as *mut c_void;
        let ptr = unsafe {
            let func = api.executor_createwith;
            let ptr = func(executor_execute);

            GET_EXECUTOR_CTX_FUNC.store(
                api.executor_getclientcontext as *mut c_void,
                Ordering::SeqCst,
            );
            let set_ctx = api.executor_setclientcontext;
            set_ctx(ptr, ctx_raw);
            ptr
        };
        let inner = Arc::new(ExecutorInner { api, ptr, ctx_raw });
        Ok(Self { inner })
    }

    pub fn ptr(&self) -> *mut c_void {
        self.inner.ptr
    }
}

impl Drop for ExecutorInner {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                let func = self.api.executor_destroy;
                func(self.ptr);
                self.ptr = std::ptr::null_mut();
            }
            // Convert raw pointer back to Arc and drop it
            if !self.ctx_raw.is_null() {
                let _ = Arc::from_raw(self.ctx_raw as *mut ExecutorContext);
            }
        }
    }
}

impl Drop for CallbackInner {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                let func = self.api.urlrequestcallback_destroy;
                func(self.ptr);
                self.ptr = std::ptr::null_mut();
            }
            // Convert raw pointer back to Arc and drop it
            if !self.ctx_raw.is_null() {
                let _ = Arc::from_raw(self.ctx_raw as *mut CallbackContext);
            }
        }
    }
}

pub struct Buffer {
    api: Arc<CronetApi>,
    pub ptr: *mut c_void,
}

// Buffer can be safely sent between threads because the underlying
// Cronet C object is thread-safe for the operations we perform.
unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

impl Buffer {
    pub fn new(api: Arc<CronetApi>) -> Result<Self> {
        let ptr = unsafe {
            if let Ok(func) = api.get_sym::<unsafe extern "C" fn() -> *mut c_void>(b"Cronet_Buffer_Create") {
                func()
            } else {
                return Err(Error::SymbolNotFound("Cronet_Buffer_Create".to_string()));
            }
        };
        Ok(Self { api, ptr })
    }

    pub fn init_with_alloc(&self, size: u64) -> Result<()> {
        unsafe {
            let func = self.api.buffer_initwithalloc;
            func(self.ptr, size);
        }
        Ok(())
    }

    pub fn get_size(&self) -> Result<u64> {
        unsafe {
            let func = self.api.buffer_getsize;
            Ok(func(self.ptr))
        }
    }

    pub fn get_data(&self) -> Result<*mut c_void> {
        unsafe {
            let func = self.api.buffer_getdata;
            Ok(func(self.ptr))
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                let func = self.api.buffer_destroy;
                func(self.ptr);
                self.ptr = std::ptr::null_mut();
            }
        }
    }
}

pub struct UrlResponseInfo {
    api: Arc<CronetApi>,
    pub ptr: *mut c_void,
}

unsafe impl Send for UrlResponseInfo {}
unsafe impl Sync for UrlResponseInfo {}

impl UrlResponseInfo {
    pub fn from_ptr(api: Arc<CronetApi>, ptr: *mut c_void) -> Self {
        Self { api, ptr }
    }

    pub fn http_status_code(&self) -> Result<i32> {
        unsafe {
            let func = self.api.urlresponseinfo_http_status_code_get;
            Ok(func(self.ptr))
        }
    }

    pub fn http_status_text(&self) -> Result<String> {
        unsafe {
            let func = self.api.urlresponseinfo_http_status_text_get;
            let ptr = func(self.ptr);
            crate::error::cstr_to_string(ptr)
        }
    }

    pub fn negotiated_protocol(&self) -> Result<String> {
        unsafe {
            let func = self.api.urlresponseinfo_negotiated_protocol_get;
            let ptr = func(self.ptr);
            crate::error::cstr_to_string(ptr)
        }
    }
}

impl std::fmt::Debug for CronetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CronetError")
            .field("ptr", &self.ptr)
            .finish()
    }
}

pub struct CronetError {
    api: Arc<CronetApi>,
    pub ptr: *mut c_void,
}

unsafe impl Send for CronetError {}
unsafe impl Sync for CronetError {}

impl CronetError {
    pub fn from_ptr(api: Arc<CronetApi>, ptr: *mut c_void) -> Self {
        Self { api, ptr }
    }

    pub fn error_code(&self) -> Result<i32> {
        unsafe {
            let func = self.api.error_error_code_get;
            Ok(func(self.ptr))
        }
    }

    pub fn message(&self) -> Result<String> {
        unsafe {
            let func = self.api.error_message_get;
            let ptr = func(self.ptr);
            crate::error::cstr_to_string(ptr)
        }
    }
}

/// Trait definition for handling the events of a Cronet URL request lifecycle.
/// Users should implement this trait to hook into the various stages of network execution,
/// such as handling redirects, processing response chunks, and observing success/failure.
pub trait UrlRequestCallbackHandler: Send + Sync {
    /// Invoked when a redirect response is received.
    /// To follow the redirect, `Cronet_UrlRequest_FollowRedirect` must be called on the request.
    fn on_redirect_received(
        &self,
        request: *mut c_void,
        info: *mut c_void,
        new_location_url: *const c_char,
    );

    /// Invoked when the initial response headers are received and the body is ready to be read.
    /// Read must be requested via `Cronet_UrlRequest_Read` to trigger `on_read_completed`.
    fn on_response_started(&self, request: *mut c_void, info: *mut c_void);

    /// Invoked when data has been successfully read into the provided buffer.
    fn on_read_completed(
        &self,
        request: *mut c_void,
        info: *mut c_void,
        buffer: *mut c_void,
        bytes_read: u64,
    );

    /// Invoked when the entire response body has been read and the request is successfully complete.
    fn on_succeeded(&self, request: *mut c_void, info: *mut c_void);

    /// Invoked if the request failed for any reason.
    fn on_failed(&self, request: *mut c_void, info: *mut c_void, error: *mut c_void);

    /// Invoked if the request was canceled via `Cronet_UrlRequest_Cancel`.
    fn on_canceled(&self, request: *mut c_void, info: *mut c_void);
}

pub struct UrlRequestParams {
    api: Arc<CronetApi>,
    pub ptr: *mut c_void,
}

impl UrlRequestParams {
    pub fn new(api: Arc<CronetApi>) -> Result<Self> {
        let ptr = unsafe {
            let func = api.urlrequestparams_create;
            func()
        };
        Ok(Self { api, ptr })
    }

    /// Set the HTTP method (GET, POST, etc.)
    pub fn set_http_method(&self, method: &str) -> Result<()> {
        let c_method = CString::new(method)?;
        unsafe {
            let func = self.api.urlrequestparams_http_method_set;
            func(self.ptr, c_method.as_ptr());
        }
        Ok(())
    }

    /// Enable or disable cache for this request
    pub fn set_disable_cache(&self, disable: bool) -> Result<()> {
        unsafe {
            let func = self.api.urlrequestparams_disable_cache_set;
            func(self.ptr, disable);
        }
        Ok(())
    }

    /// Set request priority
    pub fn set_priority(&self, priority: i32) -> Result<()> {
        unsafe {
            let func = self.api.urlrequestparams_priority_set;
            func(self.ptr, priority);
        }
        Ok(())
    }

    /// Add a request header
    pub fn add_header(&self, name: &str, value: &str) -> Result<()> {
        unsafe {
            let func = self.api.urlrequestparams_request_headers_add;
            let c_name = CString::new(name)?;
            let c_value = CString::new(value)?;
            func(self.ptr, c_name.as_ptr(), c_value.as_ptr());
        }
        Ok(())
    }

    /// Set upload data provider (for POST/PUT requests)
    ///
    /// # Safety
    /// The `provider_ptr` and `executor_ptr` must be valid pointers to Cronet upload data provider and executor objects.
    pub unsafe fn set_upload_data_provider(
        &self,
        provider_ptr: *mut c_void,
        executor_ptr: *mut c_void,
    ) -> Result<()> {
        unsafe {
            let func = self.api.urlrequestparams_upload_data_provider_set;
            func(self.ptr, provider_ptr, executor_ptr);
        }
        Ok(())
    }
}

impl Drop for UrlRequestParams {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                let func = self.api.urlrequestparams_destroy;
                func(self.ptr);
                self.ptr = std::ptr::null_mut();
            }
        }
    }
}

#[derive(Clone)]
pub struct UrlRequestCallback {
    inner: Arc<CallbackInner>,
}

unsafe impl Send for UrlRequestCallback {}
unsafe impl Sync for UrlRequestCallback {}

extern "C" fn on_redirect_received_cb(
    callback: *mut c_void,
    request: *mut c_void,
    info: *mut c_void,
    new_location_url: *const c_char,
) {
    unsafe {
        let get_ctx_ptr = GET_CALLBACK_CTX_FUNC.load(Ordering::SeqCst);
        if get_ctx_ptr.is_null() {
            return;
        }
        let get_ctx: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
            std::mem::transmute(get_ctx_ptr);
        let ctx = get_ctx(callback) as *mut CallbackContext;
        if !ctx.is_null() {
            (*ctx)
                .handler
                .on_redirect_received(request, info, new_location_url);
        }
    }
}

extern "C" fn on_response_started_cb(
    callback: *mut c_void,
    request: *mut c_void,
    info: *mut c_void,
) {
    unsafe {
        let get_ctx_ptr = GET_CALLBACK_CTX_FUNC.load(Ordering::SeqCst);
        if get_ctx_ptr.is_null() {
            return;
        }
        let get_ctx: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
            std::mem::transmute(get_ctx_ptr);
        let ctx = get_ctx(callback) as *mut CallbackContext;
        if !ctx.is_null() {
            (*ctx).handler.on_response_started(request, info);
        }
    }
}

extern "C" fn on_read_completed_cb(
    callback: *mut c_void,
    request: *mut c_void,
    info: *mut c_void,
    buffer: *mut c_void,
    bytes_read: u64,
) {
    unsafe {
        let get_ctx_ptr = GET_CALLBACK_CTX_FUNC.load(Ordering::SeqCst);
        if get_ctx_ptr.is_null() {
            return;
        }
        let get_ctx: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
            std::mem::transmute(get_ctx_ptr);
        let ctx = get_ctx(callback) as *mut CallbackContext;
        if !ctx.is_null() {
            (*ctx)
                .handler
                .on_read_completed(request, info, buffer, bytes_read);
        }
    }
}

extern "C" fn on_succeeded_cb(callback: *mut c_void, request: *mut c_void, info: *mut c_void) {
    unsafe {
        let get_ctx_ptr = GET_CALLBACK_CTX_FUNC.load(Ordering::SeqCst);
        if get_ctx_ptr.is_null() {
            return;
        }
        let get_ctx: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
            std::mem::transmute(get_ctx_ptr);
        let ctx = get_ctx(callback) as *mut CallbackContext;
        if !ctx.is_null() {
            (*ctx).handler.on_succeeded(request, info);
        }
    }
}

extern "C" fn on_failed_cb(
    callback: *mut c_void,
    request: *mut c_void,
    info: *mut c_void,
    error: *mut c_void,
) {
    unsafe {
        let get_ctx_ptr = GET_CALLBACK_CTX_FUNC.load(Ordering::SeqCst);
        if get_ctx_ptr.is_null() {
            return;
        }
        let get_ctx: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
            std::mem::transmute(get_ctx_ptr);
        let ctx = get_ctx(callback) as *mut CallbackContext;
        if !ctx.is_null() {
            (*ctx).handler.on_failed(request, info, error);
        }
    }
}

extern "C" fn on_canceled_cb(callback: *mut c_void, request: *mut c_void, info: *mut c_void) {
    unsafe {
        let get_ctx_ptr = GET_CALLBACK_CTX_FUNC.load(Ordering::SeqCst);
        if get_ctx_ptr.is_null() {
            return;
        }
        let get_ctx: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
            std::mem::transmute(get_ctx_ptr);
        let ctx = get_ctx(callback) as *mut CallbackContext;
        if !ctx.is_null() {
            (*ctx).handler.on_canceled(request, info);
        }
    }
}

impl UrlRequestCallback {
    pub fn new(
        api: Arc<CronetApi>,
        handler: impl UrlRequestCallbackHandler + 'static,
    ) -> Result<Self> {
        let ctx = Arc::new(CallbackContext {
            api: api.clone(),
            handler: Arc::new(handler),
        });
        let ctx_raw = Arc::into_raw(ctx) as *mut c_void;
        let ptr = unsafe {
            let func = api.urlrequestcallback_createwith;

            let cb_ptr = func(
                on_redirect_received_cb,
                on_response_started_cb,
                on_read_completed_cb,
                on_succeeded_cb,
                on_failed_cb,
                on_canceled_cb,
            );

            GET_CALLBACK_CTX_FUNC.store(
                api.urlrequestcallback_getclientcontext as *mut c_void,
                Ordering::SeqCst,
            );
            let set_ctx = api.urlrequestcallback_setclientcontext;
            set_ctx(cb_ptr, ctx_raw);
            cb_ptr
        };
        let inner = Arc::new(CallbackInner { api, ptr, ctx_raw });
        Ok(Self { inner })
    }

    pub fn ptr(&self) -> *mut c_void {
        self.inner.ptr
    }
}

pub struct UrlRequest {
    api: Arc<CronetApi>,
    pub ptr: *mut c_void,
}

unsafe impl Send for UrlRequest {}
unsafe impl Sync for UrlRequest {}

impl UrlRequest {
    /// Create a new URL request.
    ///
    /// # Safety
    /// The `params` pointer must be a valid pointer to a Cronet_UrlRequestParams object.
    /// The caller must ensure the params object remains valid for the lifetime of the request.
    pub unsafe fn new(
        api: Arc<CronetApi>,
        engine: &Engine,
        url: url::Url,
        params: *mut c_void, // Cronet_UrlRequestParams
        callback: &UrlRequestCallback,
        executor: &Executor,
    ) -> Result<Self> {
        let c_url = CString::new(url.as_str())?;
        let ptr = unsafe {
            let create_func = api.urlrequest_create;
            let req_ptr = create_func();
            
            let init_func = api.urlrequest_initwithparams;
            let res = init_func(
                req_ptr,
                engine.ptr,
                c_url.as_ptr(),
                params,
                callback.ptr(),
                executor.ptr(),
            );
            if res != 0 {
                // Init failed, destroy the created request and return error
                let destroy_func = api.urlrequest_destroy;
                destroy_func(req_ptr);
                return Err(Error::CronetApi { 
                    error_code: res, 
                    message: format!("UrlRequest_InitWithParams failed with code {}", res) 
                });
            }
            req_ptr
        };
        Ok(Self { api, ptr })
    }

    pub fn start(&self) -> Result<()> {
        unsafe {
            let func = self.api.urlrequest_start;
            func(self.ptr);
        }
        Ok(())
    }

    pub fn read(&self, buffer: &Buffer) -> Result<()> {
        unsafe {
            let func = self.api.urlrequest_read;
            func(self.ptr, buffer.ptr);
        }
        Ok(())
    }
}

impl Drop for UrlRequest {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                let func = self.api.urlrequest_destroy;
                func(self.ptr);
                self.ptr = std::ptr::null_mut();
            }
        }
    }
}
extern "C" fn executor_execute(_executor_ptr: *mut c_void, runnable_ptr: *mut c_void) {
    let runnable_addr = runnable_ptr as usize;
    let exec_addr = _executor_ptr as usize;

    unsafe {
        let get_ctx_ptr = GET_EXECUTOR_CTX_FUNC.load(Ordering::SeqCst);
        if get_ctx_ptr.is_null() {
            return;
        }
        let get_ctx: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
            std::mem::transmute(get_ctx_ptr);
        let ctx_ptr = get_ctx(exec_addr as *mut c_void) as *mut ExecutorContext;
        if ctx_ptr.is_null() {
            return;
        }
        let api = (*ctx_ptr).api.clone();
        let handle = (*ctx_ptr).handle.clone();

        let task = move || {
            let ptr = runnable_addr as *mut c_void;
            let run_func = api.runnable_run;
            run_func(ptr);
            let destroy_func = api.runnable_destroy;
            destroy_func(ptr);
        };

        if let Some(h) = handle {
            h.spawn_blocking(task);
        } else {
            std::thread::spawn(task);
        }
    }
}
