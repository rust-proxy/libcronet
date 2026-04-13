use cronet_rs::async_request::AsyncUrlRequest;
use cronet_rs::request::Executor;
use cronet_rs::{CronetApi, Engine, EngineParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Determine DLL path
    let dll_path = std::env::var("CRONET_LIB_PATH")
        .unwrap_or_else(|_| "dylibs/libcronet-windows-amd64.dll".to_string());
    println!("Loading Cronet from: {}", dll_path);

    // 2. Initialize Cronet API and Engine
    let api = CronetApi::new(&dll_path)?;
    let params = EngineParams::new(api.clone())?;
    params.set_enable_quic(true)?;
    params.set_user_agent("Cronet-Rust-Tokio-Example/1.0")?;

    let engine = Engine::new(api.clone())?;
    engine.start_with_params(&params)?;

    println!("Cronet Engine Version: {}", engine.get_version_string()?);

    // 3. Create Executor for thread dispatching
    let executor = Executor::new()?;

    // 4. Create and start Async Request
    let url = "https://httpbin.org/get";
    println!("Starting async request to: {}", url);
    
    let async_request = AsyncUrlRequest::new(&engine, &executor, url)?;

    // 5. Await the Future natively using Tokio
    match async_request.send().await {
        Ok(data) => {
            println!("Request succeeded! Received {} bytes.", data.len());
            if let Ok(text) = String::from_utf8(data) {
                println!("Response Body:\n{}", text);
            }
        }
        Err(e) => {
            println!("Request failed with error code: {:?}", e.error_code());
        }
    }

    Ok(())
}
