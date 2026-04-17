use cronet_rs::request::{
    Buffer, Executor, UrlRequest, UrlRequestCallback, UrlRequestCallbackHandler, UrlRequestParams,
};
use cronet_rs::{CronetApi, Engine, EngineParams};
use std::ffi::c_void;
use std::os::raw::c_char;
use std::sync::Arc;
use std::sync::mpsc;

/// Custom Callback Implementation
struct ExampleCallback {
    api: Arc<CronetApi>,
    tx: mpsc::Sender<String>,
}

impl UrlRequestCallbackHandler for ExampleCallback {
    fn on_redirect_received(
        &self,
        request: *mut c_void,
        _info: *mut c_void,
        _new_location_url: *const c_char,
    ) {
        println!("Redirect received, following...");
        unsafe {
            if let Ok(func) = self
                .api
                .get_sym::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_UrlRequest_FollowRedirect")
            {
                func(request);
            }
        }
    }

    fn on_response_started(&self, request: *mut c_void, _info: *mut c_void) {
        println!("Response started! Allocating buffer for read...");
        let buffer = Buffer::new(self.api.clone()).unwrap();
        buffer.init_with_alloc(32768).unwrap(); // 32KB chunk
        unsafe {
            if let Ok(func) = self
                .api
                .get_sym::<unsafe extern "C" fn(*mut c_void, *mut c_void)>(
                    b"Cronet_UrlRequest_Read",
                )
            {
                func(request, buffer.ptr);
            }
        }
        // Forget buffer to let it live until read is completed (memory management simplified for example)
        std::mem::forget(buffer);
    }

    fn on_read_completed(
        &self,
        request: *mut c_void,
        _info: *mut c_void,
        buffer_ptr: *mut c_void,
        bytes_read: u64,
    ) {
        println!("Read chunk completed: {} bytes", bytes_read);
        if bytes_read > 0 {
            // Keep reading
            unsafe {
                if let Ok(func) = self
                    .api
                    .get_sym::<unsafe extern "C" fn(*mut c_void, *mut c_void)>(
                        b"Cronet_UrlRequest_Read",
                    )
                {
                    func(request, buffer_ptr);
                }
            }
        }
    }

    fn on_succeeded(&self, _request: *mut c_void, _info: *mut c_void) {
        println!("Request succeeded!");
        let _ = self.tx.send("Success".to_string());
    }

    fn on_failed(&self, _request: *mut c_void, _info: *mut c_void, _error: *mut c_void) {
        println!("Request failed!");
        let _ = self.tx.send("Failed".to_string());
    }

    fn on_canceled(&self, _request: *mut c_void, _info: *mut c_void) {
        println!("Request canceled!");
        let _ = self.tx.send("Canceled".to_string());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dll_path = std::env::var("CRONET_LIB_PATH")
        .unwrap_or_else(|_| "dylibs/libcronet-windows-amd64.dll".to_string());
    println!("Loading Cronet from: {}", dll_path);

    let api = CronetApi::new(&dll_path)?;
    let params = EngineParams::new(api.clone())?;
    let engine = Engine::new(api.clone())?;
    engine.start_with_params(&params)?;

    let executor = Executor::new(api.clone())?;

    // Setup communication channel to block main thread
    let (tx, rx) = mpsc::channel();
    let handler = ExampleCallback {
        api: api.clone(),
        tx,
    };

    let callback = UrlRequestCallback::new(api.clone(), handler)?;
    let req_params = UrlRequestParams::new(api.clone())?;

    let request = UrlRequest::new(
        api.clone(),
        &engine,
        url::Url::parse("https://httpbin.org/get").unwrap(),
        req_params.ptr,
        &callback,
        &executor,
    )?;

    println!("Starting Callback-based request...");
    request.start()?;

    // Block main thread waiting for callback completion
    let result = rx.recv()?;
    println!("Final Result from channel: {}", result);

    Ok(())
}
