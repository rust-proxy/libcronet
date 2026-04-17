use cronet_rs::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dll_path = "dylibs/libcronet-linux-amd64.so";
    println!("Loading Cronet from: {}", dll_path);

    // Just create and immediately drop client
    let client = Client::new(dll_path)?;
    
    println!("Cronet Engine Version: {}", client.engine().get_version_string()?);
    println!("SUCCESS: Created and will drop client");
    
    // Client drops here automatically
    Ok(())
}