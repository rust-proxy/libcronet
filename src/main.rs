use cronet_rs::{CronetApi, Engine, EngineParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dll_path = std::env::var("CRONET_LIB_PATH")
        .unwrap_or_else(|_| "dylibs/libcronet-windows-amd64.dll".to_string());
    println!("Loading {}...", dll_path);
    let api = CronetApi::new(&dll_path)?;
    println!("Successfully loaded the library!");

    // Set up Engine Params
    let params = EngineParams::new(api.clone())?;
    params.set_enable_quic(true)?;
    params.set_user_agent("Cronet-Rust/1.0")?;
    println!("Successfully created and configured EngineParams.");

    // Initialize Engine
    let engine = Engine::new(api.clone())?;
    println!("Successfully created Cronet Engine.");

    // Start Engine
    let result = engine.start_with_params(&params)?;
    if result == 0 {
        println!("Successfully started Cronet Engine.");
    } else {
        println!("Failed to start Cronet Engine. Result code: {}", result);
    }

    // Get Version
    let version = engine.get_version_string()?;
    if !version.is_empty() {
        println!("Cronet Engine Version: {}", version);
    } else {
        println!("Cronet Engine Version is null");
    }

    // Rust's Drop trait will automatically call Cronet_EngineParams_Destroy and Cronet_Engine_Destroy
    // at the end of the scope!
    println!("Destroying Cronet Engine via automatic Drop.");

    Ok(())
}
