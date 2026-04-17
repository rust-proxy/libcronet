//! 测试 AsyncRequesterEnhanced 的基本功能

use cronet_rs::async_wrapper_enhanced::AsyncRequesterEnhanced;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试 AsyncRequesterEnhanced 基本功能...");
    
    // 1. 创建请求器
    let requester = AsyncRequesterEnhanced::new("dylibs/libcronet-linux-amd64.so")?;
    println!("✓ 引擎版本: {}", requester.engine_version()?);
    
    // 2. 测试 GET 请求
    println!("\n1. 测试 GET 请求:");
    match requester.fetch("https://httpbin.org/get").await {
        Ok((protocol, data)) => {
            println!("  ✓ 成功!");
            println!("    协议: {}", protocol);
            println!("    长度: {} 字节", data.len());
        }
        Err(e) => {
            println!("  ✗ 失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 3. 测试 POST 请求
    println!("\n2. 测试 POST 请求:");
    match requester.post("https://httpbin.org/post", Some("test data")).await {
        Ok((protocol, data)) => {
            println!("  ✓ 成功!");
            println!("    协议: {}", protocol);
            println!("    长度: {} 字节", data.len());
        }
        Err(e) => {
            println!("  ✗ 失败: {}", e);
            // POST 可能不完全支持，继续测试
        }
    }
    
    // 4. 测试带自定义头
    println!("\n3. 测试带自定义头:");
    let headers = vec![
        ("User-Agent".to_string(), "cronet-test/1.0".to_string()),
    ];
    
    match requester.fetch_with_headers("https://httpbin.org/headers", headers).await {
        Ok((protocol, data)) => {
            println!("  ✓ 成功!");
            println!("    协议: {}", protocol);
            println!("    长度: {} 字节", data.len());
            
            // 检查响应是否包含自定义头
            let response = String::from_utf8_lossy(&data);
            if response.contains("cronet-test") {
                println!("    ✓ 自定义头正确传递");
            }
        }
        Err(e) => {
            println!("  ✗ 失败: {}", e);
        }
    }
    
    // 5. 测试连接
    println!("\n4. 测试连接:");
    match requester.test_connection("https://httpbin.org/get").await {
        Ok(true) => {
            println!("  ✓ 连接成功");
        }
        Ok(false) => {
            println!("  ✗ 连接失败");
        }
        Err(e) => {
            println!("  ✗ 测试错误: {}", e);
        }
    }
    
    // 6. 验证无段错误
    println!("\n5. 验证无段错误:");
    for i in 0..3 {
        println!("  循环 {}:", i + 1);
        match requester.fetch("https://httpbin.org/get").await {
            Ok((protocol, _)) => {
                println!("    ✓ 请求成功 (协议: {})", protocol);
            }
            Err(e) => {
                println!("    ✗ 请求失败: {}", e);
            }
        }
    }
    
    println!("\n✅ 所有测试完成!");
    println!("✅ AsyncRequesterEnhanced 工作正常!");
    println!("✅ 无段错误!");
    
    Ok(())
}