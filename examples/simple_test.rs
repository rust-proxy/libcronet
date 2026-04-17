use cronet_rs::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Determine DLL path
    let dll_path = std::env::var("CRONET_LIB_PATH")
        .unwrap_or_else(|_| "dylibs/libcronet-linux-amd64.so".to_string());
    println!("Loading Cronet from: {}", dll_path);

    // 2. Create Client with default configuration
    let client = Client::new(&dll_path)?;
    
    println!("Cronet Engine Version: {}", client.engine().get_version_string()?);
    println!("SUCCESS: Library loaded and version retrieved!");
    
    Ok(())
}