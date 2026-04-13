use crate::async_request::AsyncUrlRequest;
use crate::request::{
    Buffer, Executor, UrlRequest, UrlRequestCallback, UrlRequestCallbackHandler, UrlRequestParams,
};
use crate::{get_lib, CronetApi, Engine, EngineParams};
use std::ffi::c_void;
use std::os::raw::c_char;
use std::sync::mpsc;

struct MyCallback {
    tx: mpsc::Sender<String>,
}

impl UrlRequestCallbackHandler for MyCallback {
    fn on_redirect_received(
        &self,
        request: *mut c_void,
        _info: *mut c_void,
        _new_location_url: *const c_char,
    ) {
        println!("Redirect received!");
        unsafe {
            if let Ok(func) = get_lib()
                .get::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_UrlRequest_FollowRedirect")
            {
                func(request);
            }
        }
    }

    fn on_response_started(&self, request: *mut c_void, _info: *mut c_void) {
        println!("Response started!");
        let buffer = Buffer::new().unwrap();
        buffer.init_with_alloc(32768).unwrap();
        unsafe {
            if let Ok(func) = get_lib()
                .get::<unsafe extern "C" fn(*mut c_void, *mut c_void)>(b"Cronet_UrlRequest_Read")
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
        println!("Read completed: {} bytes", bytes_read);
        if bytes_read > 0 {
            unsafe {
                if let Ok(func) = get_lib().get::<unsafe extern "C" fn(*mut c_void, *mut c_void)>(
                    b"Cronet_UrlRequest_Read",
                ) {
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

#[test]
fn test_http_get_callback() {
    let dll_path = std::env::var("CRONET_LIB_PATH")
        .unwrap_or_else(|_| "dylibs/libcronet-windows-amd64.dll".to_string());
    let api = CronetApi::new(&dll_path).unwrap();
    let params = EngineParams::new(api.clone()).unwrap();
    let engine = Engine::new(api.clone()).unwrap();
    engine.start_with_params(&params).unwrap();

    let executor = Executor::new().unwrap();

    let (tx, rx) = mpsc::channel();
    let handler = MyCallback { tx };
    let callback = UrlRequestCallback::new(handler).unwrap();
    let req_params = UrlRequestParams::new().unwrap();

    let request = UrlRequest::new(
        &engine,
        "https://httpbin.org/get",
        req_params.ptr,
        &callback,
        &executor,
    )
    .unwrap();

    request.start().unwrap();

    let result = rx.recv().unwrap();
    assert_eq!(result, "Success");
}





#[tokio::test]
async fn test_http_get_async_failed() {
    let dll_path = std::env::var("CRONET_LIB_PATH")
        .unwrap_or_else(|_| "dylibs/libcronet-windows-amd64.dll".to_string());
    let api = CronetApi::new(&dll_path).unwrap();
    let params = EngineParams::new(api.clone()).unwrap();
    let engine = Engine::new(api.clone()).unwrap();
    engine.start_with_params(&params).unwrap();

    let executor = Executor::new().unwrap();

    // Invalid URL should cause a failure
    let async_request = AsyncUrlRequest::new(
        &engine,
        &executor,
        "https://this-is-a-non-existent-domain.internal/get",
    )
    .unwrap();

    let result = async_request.send().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    let code = err.error_code().unwrap_or(0);
    println!("Expected error code received: {}", code);
    assert!(code != 0);
}

#[test]
fn test_buffer_allocation() {
    let dll_path = std::env::var("CRONET_LIB_PATH")
        .unwrap_or_else(|_| "dylibs/libcronet-windows-amd64.dll".to_string());
    let _api = CronetApi::new(&dll_path).unwrap();

    let buffer = Buffer::new().unwrap();
    buffer.init_with_alloc(1024).unwrap();

    let size = buffer.get_size().unwrap();
    assert_eq!(size, 1024);

    let data_ptr = buffer.get_data().unwrap();
    assert!(!data_ptr.is_null());
}

#[test]
fn test_executor_lifecycle() {
    let dll_path = std::env::var("CRONET_LIB_PATH")
        .unwrap_or_else(|_| "dylibs/libcronet-windows-amd64.dll".to_string());
    let _api = CronetApi::new(&dll_path).unwrap();

    let executor = Executor::new();
    assert!(executor.is_ok());
}


