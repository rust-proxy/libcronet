use cronet_rs::Client;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Determine DLL path
    let dll_path = std::env::var("CRONET_LIB_PATH")
        .unwrap_or_else(|_| "dylibs/libcronet-linux-amd64.so".to_string());
    println!("Loading Cronet from: {}", dll_path);

    // 2. Create Client with default configuration
    let client = Client::new(&dll_path)?;
    
    println!("Cronet Engine Version: {}", client.engine().get_version_string()?);

    // 3. Create async runtime
    let rt = Runtime::new()?;
    
    rt.block_on(async {
        // Try a simple request to a known working endpoint
        println!("=== Testing with httpbin.org ===");
        match client.fetch("https://httpbin.org/status/200").await {
            Ok((protocol, data)) => {
                println!("Success! Protocol: {}, Received {} bytes", protocol, data.len());
                println!("Response: {:?}", String::from_utf8_lossy(&data));
            }
            Err(e) => println!("Request failed: {}", e),
        }
    });
    
    Ok(())
}