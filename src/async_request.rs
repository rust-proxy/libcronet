use std::ffi::c_void;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

use crate::request::{
    Buffer, CronetError, Executor, UrlRequest, UrlRequestCallback, UrlRequestCallbackHandler,
    UrlRequestParams, UrlResponseInfo,
};
use crate::{get_lib, Engine};

struct RequestState {
    data: Vec<u8>,
    sender: Option<oneshot::Sender<Result<Vec<u8>, CronetError>>>,
    response_info: Option<UrlResponseInfo>,
}

struct AsyncCallback {
    state: Arc<Mutex<RequestState>>,
}

impl UrlRequestCallbackHandler for AsyncCallback {
    fn on_redirect_received(
        &self,
        request: *mut c_void,
        _info: *mut c_void,
        _new_location_url: *const c_char,
    ) {
        // Automatically follow redirects for simplicity in this wrapper
        unsafe {
            if let Ok(func) = get_lib()
                .get::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_UrlRequest_FollowRedirect")
            {
                func(request);
            }
        }
    }

    fn on_response_started(&self, request: *mut c_void, info: *mut c_void) {
        if let Ok(mut state) = self.state.lock() {
            state.response_info = Some(UrlResponseInfo::from_ptr(info));
        }

        // Start reading
        let buffer = Buffer::new().unwrap();
        buffer.init_with_alloc(32768).unwrap();
        unsafe {
            if let Ok(func) = get_lib()
                .get::<unsafe extern "C" fn(*mut c_void, *mut c_void)>(b"Cronet_UrlRequest_Read")
            {
                func(request, buffer.ptr);
            }
        }
        std::mem::forget(buffer); // We need to manage this properly in a production version
    }

    fn on_read_completed(
        &self,
        request: *mut c_void,
        _info: *mut c_void,
        buffer_ptr: *mut c_void,
        bytes_read: u64,
    ) {
        if bytes_read > 0 {
            unsafe {
                if let Ok(get_data) = get_lib()
                    .get::<unsafe extern "C" fn(*mut c_void) -> *mut c_void>(
                        b"Cronet_Buffer_GetData",
                    )
                {
                    let data_ptr = get_data(buffer_ptr) as *const u8;
                    let slice = std::slice::from_raw_parts(data_ptr, bytes_read as usize);
                    if let Ok(mut state) = self.state.lock() {
                        state.data.extend_from_slice(slice);
                    }
                }
            }

            // Read again
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
        if let Ok(mut state) = self.state.lock() {
            if let Some(sender) = state.sender.take() {
                let data = std::mem::take(&mut state.data);
                let _ = sender.send(Ok(data));
            }
        }
    }

    fn on_failed(&self, _request: *mut c_void, _info: *mut c_void, error: *mut c_void) {
        if let Ok(mut state) = self.state.lock() {
            if let Some(sender) = state.sender.take() {
                let _ = sender.send(Err(CronetError::from_ptr(error)));
            }
        }
    }

    fn on_canceled(&self, _request: *mut c_void, _info: *mut c_void) {
        if let Ok(mut state) = self.state.lock() {
            if let Some(sender) = state.sender.take() {
                // Return an error to represent cancellation, using null ptr as a dummy for now
                let _ = sender.send(Err(CronetError::from_ptr(std::ptr::null_mut())));
            }
        }
    }
}

pub struct AsyncUrlRequest {
    _callback: UrlRequestCallback,
    request: UrlRequest,
    receiver: Option<oneshot::Receiver<Result<Vec<u8>, CronetError>>>,
}

impl AsyncUrlRequest {
    pub fn new(engine: &Engine, executor: &Executor, url: &str) -> Result<Self, libloading::Error> {
        let (tx, rx) = oneshot::channel();
        
        let state = Arc::new(Mutex::new(RequestState {
            data: Vec::new(),
            sender: Some(tx),
            response_info: None,
        }));

        let callback_handler = AsyncCallback { state };
        let callback = UrlRequestCallback::new(callback_handler)?;
        let params = UrlRequestParams::new()?;

        let request = UrlRequest::new(engine, url, params.ptr, &callback, executor)?;

        Ok(Self {
            _callback: callback,
            request,
            receiver: Some(rx),
        })
    }

    pub async fn send(mut self) -> Result<Vec<u8>, CronetError> {
        if let Some(rx) = self.receiver.take() {
            self.request.start().unwrap(); // Should handle error appropriately
            match rx.await {
                Ok(res) => res,
                Err(_) => Err(CronetError::from_ptr(std::ptr::null_mut())),
            }
        } else {
            panic!("AsyncUrlRequest can only be sent once");
        }
    }
}
