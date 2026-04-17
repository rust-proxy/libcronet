use cronet_rs::Client;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dll_path = "dylibs/libcronet-linux-amd64.so";
    println!("Loading Cronet from: {}", dll_path);

    // 1. Create client
    let client = Client::new(dll_path)?;
    
    println!("Cronet Engine Version: {}", client.engine().get_version_string()?);

    // 2. Create async runtime
    let rt = Runtime::new()?;
    
    rt.block_on(async {
        println!("=== Starting async request ===");
        
        // Try with a simple endpoint
        let result = client.fetch("https://httpbin.org/status/200").await;
        
        match result {
            Ok((protocol, data)) => {
                println!("SUCCESS! Protocol: {}, Received {} bytes", protocol, data.len());
                println!("Response: {:?}", String::from_utf8_lossy(&data));
            }
            Err(e) => {
                println!("ERROR: {}", e);
            }
        }
        
        println!("=== Request completed ===");
    });
    
    println!("=== Main function ending ===");
    
    // Client will be dropped here
    println!("=== Client will be dropped now ===");
    
    Ok(())
}