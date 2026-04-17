use cronet_rs::request::{
    Buffer, Executor, UrlRequest, UrlRequestCallback, UrlRequestCallbackHandler, UrlRequestParams,
};
use cronet_rs::{CronetApi, Engine, EngineParams};
use std::ffi::c_void;
use std::os::raw::c_char;
use std::sync::Arc;
use std::sync::mpsc;

/// Simple Callback Implementation
struct SimpleCallback {
    api: Arc<CronetApi>,
    tx: mpsc::Sender<String>,
}

impl UrlRequestCallbackHandler for SimpleCallback {
    fn on_redirect_received(
        &self,
        request: *mut c_void,
        _info: *mut c_void,
        _new_location_url: *const c_char,
    ) {
        println!("DEBUG: on_redirect_received");
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
        println!("DEBUG: on_response_started");
        let buffer = Buffer::new(self.api.clone()).unwrap();
        buffer.init_with_alloc(32768).unwrap();
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
        std::mem::forget(buffer);
    }

    fn on_read_completed(
        &self,
        request: *mut c_void,
        _info: *mut c_void,
        buffer_ptr: *mut c_void,
        bytes_read: u64,
    ) {
        println!("DEBUG: on_read_completed: {} bytes", bytes_read);
        if bytes_read > 0 {
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
        println!("DEBUG: on_succeeded");
        let _ = self.tx.send("Success".to_string());
    }

    fn on_failed(&self, _request: *mut c_void, _info: *mut c_void, _error: *mut c_void) {
        println!("DEBUG: on_failed");
        let _ = self.tx.send("Failed".to_string());
    }

    fn on_canceled(&self, _request: *mut c_void, _info: *mut c_void) {
        println!("DEBUG: on_canceled");
        let _ = self.tx.send("Canceled".to_string());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Loading Cronet");
    let api = CronetApi::new("dylibs/libcronet-linux-amd64.so")?;
    println!("2. CronetApi created");
    
    let params = EngineParams::new(api.clone())?;
    println!("3. EngineParams created");
    
    let engine = Engine::new(api.clone())?;
    println!("4. Engine created");
    
    engine.start_with_params(&params)?;
    println!("5. Engine started");
    
    println!("Cronet Engine Version: {}", engine.get_version_string()?);
    
    let executor = Executor::new(api.clone())?;
    println!("6. Executor created");
    
    // Setup communication channel
    let (tx, rx) = mpsc::channel();
    let handler = SimpleCallback {
        api: api.clone(),
        tx,
    };

    println!("7. Creating callback...");
    let callback = UrlRequestCallback::new(api.clone(), handler)?;
    println!("8. Callback created");
    
    println!("9. Creating request params...");
    let req_params = UrlRequestParams::new(api.clone())?;
    println!("10. Request params created");

    println!("11. Creating URL request...");
    let request = UrlRequest::new(
        api.clone(),
        &engine,
        url::Url::parse("https://httpbin.org/status/200").unwrap(),
        req_params.ptr,
        &callback,
        &executor,
    )?;
    println!("12. URL request created");

    println!("13. Starting request...");
    request.start()?;
    println!("14. Request started");

    // Block main thread waiting for callback completion
    println!("15. Waiting for response...");
    let result = rx.recv()?;
    println!("16. Final Result: {}", result);

    println!("17. Main function ending");
    Ok(())
}