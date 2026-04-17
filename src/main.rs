use std::fs;

fn main() {
    let dll_path = std::env::var("CRONET_LIB_PATH")
        .unwrap_or_else(|_| "dylibs/libcronet-windows-amd64.dll".to_string());

    println!("Dumping symbols from: {}", dll_path);
    let dll_data = fs::read(&dll_path).unwrap();

    let mut words = Vec::new();
    let mut current_word = String::new();
    for byte in dll_data.iter() {
        if *byte >= 32 && *byte <= 126 {
            current_word.push(*byte as char);
        } else {
            if current_word.starts_with("Cronet_") {
                words.push(current_word.clone());
            }
            current_word.clear();
        }
    }

    words.sort();
    words.dedup();

    let mut out = String::new();
    for w in &words {
        out.push_str(w);
        out.push('\n');
    }

    fs::write("symbols.txt", out).unwrap();
    println!(
        "Successfully extracted {} symbols to symbols.txt",
        words.len()
    );
}
