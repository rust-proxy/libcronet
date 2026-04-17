//! 基于回调方式的简单异步包装示例
//! 展示了如何将回调 API 转换为 async/await API

use cronet_rs::request::{
    Buffer, Executor, UrlRequest, UrlRequestCallback, UrlRequestCallbackHandler, UrlRequestParams,
};
use cronet_rs::{CronetApi, Engine, EngineParams};
use std::ffi::c_void;
use std::os::raw::c_char;
use std::sync::Arc;
use tokio::sync::oneshot;

/// 基于回调的异步请求包装器
pub struct AsyncRequest {
    api: Arc<CronetApi>,
    engine: Engine,
    executor: Executor,
}

impl AsyncRequest {
    pub fn new(dll_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let api = CronetApi::new(dll_path)?;
        let params = EngineParams::new(api.clone())?;
        let engine = Engine::new(api.clone())?;
        engine.start_with_params(&params)?;
        let executor = Executor::new(api.clone())?;
        
        Ok(Self {
            api,
            engine,
            executor,
        })
    }
    
    /// 发送异步请求
    pub async fn fetch(&self, url: &str) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
        let (tx, rx) = oneshot::channel();
        
        let handler = CallbackHandler {
            api: self.api.clone(),
            tx: Some(tx),
            data: Vec::new(),
            protocol: String::new(),
        };
        
        let callback = UrlRequestCallback::new(self.api.clone(), handler)?;
        let params = UrlRequestParams::new(self.api.clone())?;
        
        let request = UrlRequest::new(
            self.api.clone(),
            &self.engine,
            url::Url::parse(url)?,
            params.ptr,
            &callback,
            &self.executor,
        )?;
        
        request.start()?;
        
        // 等待回调完成
        match rx.await {
            Ok(result) => result,
            Err(_) => Err("Channel closed".into()),
        }
    }
}

/// 回调处理器
struct CallbackHandler {
    api: Arc<CronetApi>,
    tx: Option<oneshot::Sender<Result<(String, Vec<u8>)>>>,
    data: Vec<u8>,
    protocol: String,
}

impl UrlRequestCallbackHandler for CallbackHandler {
    fn on_redirect_received(
        &self,
        request: *mut c_void,
        _info: *mut c_void,
        _new_location_url: *const c_char,
    ) {
        unsafe {
            if let Ok(func) = self
                .api
                .get_sym::<unsafe extern "C" fn(*mut c_void)>(b"Cronet_UrlRequest_FollowRedirect")
            {
                func(request);
            }
        }
    }

    fn on_response_started(&self, request: *mut c_void, info: *mut c_void) {
        // 获取协议信息
        use cronet_rs::request::UrlResponseInfo;
        let info_obj = UrlResponseInfo::from_ptr(self.api.clone(), info);
        if let Ok(protocol) = info_obj.negotiated_protocol() {
            // 存储协议信息（这里简化处理）
        }
        
        // 分配 buffer 并开始读取
        if let Ok(buffer) = Buffer::new(self.api.clone()) {
            if buffer.init_with_alloc(32768).is_ok() {
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
                // 防止 buffer 被过早释放
                std::mem::forget(buffer);
            }
        }
    }

    fn on_read_completed(
        &self,
        request: *mut c_void,
        _info: *mut c_void,
        buffer_ptr: *mut c_void,
        bytes_read: u64,
    ) {
        if bytes_read > 0 {
            // 读取数据（这里简化处理）
            // 实际实现需要将数据添加到 self.data 中
            
            // 继续读取
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
        if let Some(tx) = self.tx.as_ref() {
            let _ = tx.send(Ok((self.protocol.clone(), self.data.clone())));
        }
    }

    fn on_failed(&self, _request: *mut c_void, _info: *mut c_void, _error: *mut c_void) {
        if let Some(tx) = self.tx.as_ref() {
            let _ = tx.send(Err(cronet_rs::error::Error::Other("Request failed".to_string()).into()));
        }
    }

    fn on_canceled(&self, _request: *mut c_void, _info: *mut c_void) {
        if let Some(tx) = self.tx.as_ref() {
            let _ = tx.send(Err(cronet_rs::error::Error::Other("Request canceled".to_string()).into()));
        }
    }
}

/// 示例主函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("创建异步请求器...");
    let requester = AsyncRequest::new("dylibs/libcronet-linux-amd64.so")?;
    
    println!("发送请求...");
    match requester.fetch("https://httpbin.org/get").await {
        Ok((protocol, data)) => {
            println!("成功! 协议: {}, 数据大小: {} 字节", protocol, data.len());
            println!("响应: {}", String::from_utf8_lossy(&data));
        }
        Err(e) => {
            println!("错误: {}", e);
        }
    }
    
    Ok(())
}