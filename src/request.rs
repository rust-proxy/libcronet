use crate::{get_lib, CronetApi, Engine, CRONET_LIB};
use libloading::Symbol;
use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_int, c_longlong};
use std::sync::Arc;

pub struct Executor {
    pub ptr: *mut c_void,
}

impl Executor {
    pub fn new() -> Result<Self, libloading::Error> {
        let ptr = unsafe {
            let func: Symbol<
                unsafe extern "C" fn(unsafe extern "C" fn(*mut c_void, *mut c_void)) -> *mut c_void,
            > = get_lib().get(b"Cronet_Executor_CreateWith")?;
            func(executor_execute)
        };
        Ok(Self { ptr })
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        unsafe {
            if let Ok(func) =
                get_lib().get::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_Executor_Destroy")
            {
                func(self.ptr);
            }
        }
    }
}

pub struct Buffer {
    pub ptr: *mut c_void,
}

impl Buffer {
    pub fn new() -> Result<Self, libloading::Error> {
        let ptr = unsafe {
            let func: Symbol<unsafe extern "C" fn() -> *mut c_void> =
                get_lib().get(b"Cronet_Buffer_Create")?;
            func()
        };
        Ok(Self { ptr })
    }

    pub fn init_with_alloc(&self, size: u64) -> Result<(), libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void, u64)> =
                get_lib().get(b"Cronet_Buffer_InitWithAlloc")?;
            func(self.ptr, size);
        }
        Ok(())
    }

    pub fn get_size(&self) -> Result<u64, libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void) -> u64> =
                get_lib().get(b"Cronet_Buffer_GetSize")?;
            Ok(func(self.ptr))
        }
    }

    pub fn get_data(&self) -> Result<*mut c_void, libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void) -> *mut c_void> =
                get_lib().get(b"Cronet_Buffer_GetData")?;
            Ok(func(self.ptr))
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            if let Ok(func) =
                get_lib().get::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_Buffer_Destroy")
            {
                func(self.ptr);
            }
        }
    }
}

pub struct UrlResponseInfo {
    pub ptr: *mut c_void,
}

unsafe impl Send for UrlResponseInfo {}
unsafe impl Sync for UrlResponseInfo {}

impl UrlResponseInfo {
    pub fn from_ptr(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    pub fn http_status_code(&self) -> Result<i32, libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void) -> i32> =
                get_lib().get(b"Cronet_UrlResponseInfo_http_status_code_get")?;
            Ok(func(self.ptr))
        }
    }

    pub fn http_status_text(&self) -> Result<String, libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void) -> *const c_char> =
                get_lib().get(b"Cronet_UrlResponseInfo_http_status_text_get")?;
            let ptr = func(self.ptr);
            if ptr.is_null() {
                return Ok(String::new());
            }
            Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }
}

#[derive(Debug)]
pub struct CronetError {
    pub ptr: *mut c_void,
}

unsafe impl Send for CronetError {}
unsafe impl Sync for CronetError {}

impl CronetError {
    pub fn from_ptr(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    pub fn error_code(&self) -> Result<i32, libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void) -> i32> =
                get_lib().get(b"Cronet_Error_error_code_get")?;
            Ok(func(self.ptr))
        }
    }

    pub fn message(&self) -> Result<String, libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void) -> *const c_char> =
                get_lib().get(b"Cronet_Error_message_get")?;
            let ptr = func(self.ptr);
            if ptr.is_null() {
                return Ok(String::new());
            }
            Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }
}

pub trait UrlRequestCallbackHandler: Send + Sync {
    fn on_redirect_received(
        &self,
        request: *mut c_void,
        info: *mut c_void,
        new_location_url: *const c_char,
    );
    fn on_response_started(&self, request: *mut c_void, info: *mut c_void);
    fn on_read_completed(
        &self,
        request: *mut c_void,
        info: *mut c_void,
        buffer: *mut c_void,
        bytes_read: u64,
    );
    fn on_succeeded(&self, request: *mut c_void, info: *mut c_void);
    fn on_failed(&self, request: *mut c_void, info: *mut c_void, error: *mut c_void);
    fn on_canceled(&self, request: *mut c_void, info: *mut c_void);
}

pub struct UrlRequestParams {
    pub ptr: *mut c_void,
}

impl UrlRequestParams {
    pub fn new() -> Result<Self, libloading::Error> {
        let ptr = unsafe {
            let func: Symbol<unsafe extern "C" fn() -> *mut c_void> =
                get_lib().get(b"Cronet_UrlRequestParams_Create")?;
            func()
        };
        Ok(Self { ptr })
    }
}

impl Drop for UrlRequestParams {
    fn drop(&mut self) {
        unsafe {
            if let Ok(func) = get_lib()
                .get::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_UrlRequestParams_Destroy")
            {
                func(self.ptr);
            }
        }
    }
}

pub struct UrlRequestCallback {
    pub ptr: *mut c_void,
    handler: Box<Box<dyn UrlRequestCallbackHandler>>, // Double box to get a raw pointer to a trait object
}

extern "C" fn on_redirect_received_cb(
    callback: *mut c_void,
    request: *mut c_void,
    info: *mut c_void,
    new_location_url: *const c_char,
) {
    unsafe {
        if let Ok(get_ctx) = get_lib().get::<unsafe extern "C" fn(*mut c_void) -> *mut c_void>(
            b"Cronet_UrlRequestCallback_GetClientContext",
        ) {
            let ctx = get_ctx(callback) as *mut Box<dyn UrlRequestCallbackHandler>;
            if !ctx.is_null() {
                (*ctx).on_redirect_received(request, info, new_location_url);
            }
        }
    }
}

extern "C" fn on_response_started_cb(
    callback: *mut c_void,
    request: *mut c_void,
    info: *mut c_void,
) {
    unsafe {
        if let Ok(get_ctx) = get_lib().get::<unsafe extern "C" fn(*mut c_void) -> *mut c_void>(
            b"Cronet_UrlRequestCallback_GetClientContext",
        ) {
            let ctx = get_ctx(callback) as *mut Box<dyn UrlRequestCallbackHandler>;
            if !ctx.is_null() {
                (*ctx).on_response_started(request, info);
            }
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
        if let Ok(get_ctx) = get_lib().get::<unsafe extern "C" fn(*mut c_void) -> *mut c_void>(
            b"Cronet_UrlRequestCallback_GetClientContext",
        ) {
            let ctx = get_ctx(callback) as *mut Box<dyn UrlRequestCallbackHandler>;
            if !ctx.is_null() {
                (*ctx).on_read_completed(request, info, buffer, bytes_read);
            }
        }
    }
}

extern "C" fn on_succeeded_cb(callback: *mut c_void, request: *mut c_void, info: *mut c_void) {
    unsafe {
        if let Ok(get_ctx) = get_lib().get::<unsafe extern "C" fn(*mut c_void) -> *mut c_void>(
            b"Cronet_UrlRequestCallback_GetClientContext",
        ) {
            let ctx = get_ctx(callback) as *mut Box<dyn UrlRequestCallbackHandler>;
            if !ctx.is_null() {
                (*ctx).on_succeeded(request, info);
            }
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
        if let Ok(get_ctx) = get_lib().get::<unsafe extern "C" fn(*mut c_void) -> *mut c_void>(
            b"Cronet_UrlRequestCallback_GetClientContext",
        ) {
            let ctx = get_ctx(callback) as *mut Box<dyn UrlRequestCallbackHandler>;
            if !ctx.is_null() {
                (*ctx).on_failed(request, info, error);
            }
        }
    }
}

extern "C" fn on_canceled_cb(callback: *mut c_void, request: *mut c_void, info: *mut c_void) {
    unsafe {
        if let Ok(get_ctx) = get_lib().get::<unsafe extern "C" fn(*mut c_void) -> *mut c_void>(
            b"Cronet_UrlRequestCallback_GetClientContext",
        ) {
            let ctx = get_ctx(callback) as *mut Box<dyn UrlRequestCallbackHandler>;
            if !ctx.is_null() {
                (*ctx).on_canceled(request, info);
            }
        }
    }
}

impl UrlRequestCallback {
    pub fn new(
        handler: impl UrlRequestCallbackHandler + 'static,
    ) -> Result<Self, libloading::Error> {
        let mut boxed_handler: Box<Box<dyn UrlRequestCallbackHandler>> =
            Box::new(Box::new(handler));

        let ptr = unsafe {
            let func: Symbol<
                unsafe extern "C" fn(
                    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *const c_char),
                    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
                    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, u64),
                    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
                    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void),
                    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
                ) -> *mut c_void,
            > = get_lib().get(b"Cronet_UrlRequestCallback_CreateWith")?;

            let cb_ptr = func(
                on_redirect_received_cb,
                on_response_started_cb,
                on_read_completed_cb,
                on_succeeded_cb,
                on_failed_cb,
                on_canceled_cb,
            );

            let set_ctx: Symbol<unsafe extern "C" fn(*mut c_void, *mut c_void)> =
                get_lib().get(b"Cronet_UrlRequestCallback_SetClientContext")?;
            set_ctx(cb_ptr, boxed_handler.as_mut() as *mut _ as *mut c_void);

            cb_ptr
        };
        Ok(Self {
            ptr,
            handler: boxed_handler,
        })
    }
}

impl Drop for UrlRequestCallback {
    fn drop(&mut self) {
        unsafe {
            if let Ok(func) = get_lib()
                .get::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_UrlRequestCallback_Destroy")
            {
                func(self.ptr);
            }
        }
    }
}

pub struct UrlRequest {
    pub ptr: *mut c_void,
}

impl UrlRequest {
    pub fn new(
        engine: &Engine,
        url: &str,
        params: *mut c_void, // Cronet_UrlRequestParams
        callback: &UrlRequestCallback,
        executor: &Executor,
    ) -> Result<Self, libloading::Error> {
        let c_url = CString::new(url).unwrap();
        let ptr = unsafe {
            let func: Symbol<unsafe extern "C" fn() -> *mut c_void> =
                get_lib().get(b"Cronet_UrlRequest_Create")?;
            let req_ptr = func();

            let init_func: Symbol<
                unsafe extern "C" fn(
                    *mut c_void,
                    *mut c_void,
                    *const c_char,
                    *mut c_void,
                    *mut c_void,
                    *mut c_void,
                ) -> c_int,
            > = get_lib().get(b"Cronet_UrlRequest_InitWithParams")?;

            let res = init_func(
                req_ptr,
                engine.ptr,
                c_url.as_ptr(),
                params,
                callback.ptr,
                executor.ptr,
            );
            if res != 0 {
                // handle error
            }
            req_ptr
        };
        Ok(Self { ptr })
    }

    pub fn start(&self) -> Result<(), libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void)> =
                get_lib().get(b"Cronet_UrlRequest_Start")?;
            func(self.ptr);
        }
        Ok(())
    }

    pub fn read(&self, buffer: &Buffer) -> Result<(), libloading::Error> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut c_void, *mut c_void)> =
                get_lib().get(b"Cronet_UrlRequest_Read")?;
            func(self.ptr, buffer.ptr);
        }
        Ok(())
    }
}

impl Drop for UrlRequest {
    fn drop(&mut self) {
        unsafe {
            if let Ok(func) =
                get_lib().get::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_UrlRequest_Destroy")
            {
                func(self.ptr);
            }
        }
    }
}
extern "C" fn executor_execute(_executor_ptr: *mut c_void, runnable_ptr: *mut c_void) {
    let runnable_addr = runnable_ptr as usize;
    std::thread::spawn(move || unsafe {
        let ptr = runnable_addr as *mut c_void;
        if let Ok(run_func) =
            get_lib().get::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_Runnable_Run")
        {
            run_func(ptr);
        }
        if let Ok(destroy_func) =
            get_lib().get::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_Runnable_Destroy")
        {
            destroy_func(ptr);
        }
    });
}
