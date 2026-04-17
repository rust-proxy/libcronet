use cronet_rs::{CronetApi, Engine, EngineParams};
use cronet_rs::request::{UrlRequest, UrlRequestCallback, UrlRequestCallbackHandler, UrlRequestParams, UrlResponseInfo};
use std::sync::{Arc, Mutex};
use std::ffi::c_void;
use std::os::raw::c_char;
use url::Url as UrlType;

struct SimpleCallback {
    result: Arc<Mutex<Option<Result<(String, Vec<u8>), String>>>>,
}

impl UrlRequestCallbackHandler for SimpleCallback {
    fn on_redirect_received(
        &self,
        request: *mut c_void,
        _info: *mut c_void,
        _new_location_url: *const c_char,
    ) {
        unsafe {
            // Follow redirect
            // This would need the function pointer
        }
    }

    fn on_response_started(&self, _request: *mut c_void, _info: *mut c_void) {
        println!("Response started");
    }

    fn on_read_completed(
        &self,
        _request: *mut c_void,
        _info: *mut c_void,
        _buffer: *mut c_void,
        _bytes_read: u64,
    ) {
        println!("Read completed");
    }

    fn on_succeeded(&self, _request: *mut c_void, _info: *mut c_void) {
        println!("Request succeeded");
        let mut guard = self.result.lock().unwrap();
        *guard = Some(Ok(("unknown".to_string(), Vec::new())));
    }

    fn on_failed(&self, _request: *mut c_void, _info: *mut c_void, _error: *mut c_void) {
        println!("Request failed");
        let mut guard = self.result.lock().unwrap();
        *guard = Some(Err("Request failed".to_string()));
    }

    fn on_canceled(&self, _request: *mut c_void, _info: *mut c_void) {
        println!("Request canceled");
        let mut guard = self.result.lock().unwrap();
        *guard = Some(Err("Request canceled".to_string()));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Determine DLL path
    let dll_path = std::env::var("CRONET_LIB_PATH")
        .unwrap_or_else(|_| "dylibs/libcronet-linux-amd64.so".to_string());
    println!("Loading Cronet from: {}", dll_path);

    // 2. Initialize Cronet API and Engine
    let api = CronetApi::new(&dll_path)?;
    let params = EngineParams::new(api.clone())?;
    params.set_enable_quic(true)?;
    
    let engine = Engine::new(api.clone())?;
    engine.start_with_params(&params)?;
    
    println!("Cronet Engine Version: {}", engine.get_version_string()?);
    
    // 3. Create executor
    let executor = cronet_rs::request::Executor::new(api.clone())?;
    
    // 4. Create callback
    let result = Arc::new(Mutex::new(None));
    let callback_handler = SimpleCallback {
        result: result.clone(),
    };
    let callback = UrlRequestCallback::new(api.clone(), callback_handler)?;
    
    // 5. Create request params
    let request_params = UrlRequestParams::new(api.clone())?;
    
    // 6. Create and start request
    let url = UrlType::parse("https://httpbin.org/status/200")?;
    let request = UrlRequest::new(
        api.clone(),
        &engine,
        url,
        request_params.ptr,
        &callback,
        &executor,
    )?;
    
    println!("Starting request...");
    request.start()?;
    
    // 7. Wait for result (simple polling)
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let guard = result.lock().unwrap();
        if guard.is_some() {
            println!("Got result!");
            break;
        }
    }
    
    println!("Test completed");
    Ok(())
}