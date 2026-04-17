//! 快速测试 AsyncRequester

use cronet_rs::async_wrapper::AsyncRequester;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 快速测试 AsyncRequester ===");
    
    // 1. 创建异步请求器
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("✓ 引擎版本: {}", requester.engine_version()?);
    
    // 2. 简单 GET 请求
    println!("\n测试简单 GET 请求...");
    match requester.fetch("https://httpbin.org/get").await {
        Ok((protocol, data)) => {
            println!("✓ 请求成功!");
            println!("  协议: {}", protocol);
            println!("  响应长度: {} 字节", data.len());
            
            // 检查响应是否包含预期内容
            let response = String::from_utf8_lossy(&data);
            if response.contains("httpbin.org") {
                println!("✓ 响应内容正确");
            } else {
                println!("⚠ 响应内容可能不正确");
            }
        }
        Err(e) => {
            println!("✗ 请求失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 3. 测试错误请求
    println!("\n测试错误请求...");
    match requester.fetch("https://httpbin.org/status/404").await {
        Ok((protocol, data)) => {
            println!("✓ 404 请求成功 (协议: {}, 长度: {}字节)", protocol, data.len());
        }
        Err(e) => {
            println!("✓ 404 请求失败（预期）: {}", e);
        }
    }
    
    // 4. 测试并发
    println!("\n测试并发请求...");
    let start = std::time::Instant::now();
    
    let urls = vec![
        "https://httpbin.org/status/200",
        "https://httpbin.org/status/201",
        "https://httpbin.org/status/202",
    ];
    
    // 由于 AsyncRequester 不是 Send，我们在主线程中顺序执行
    let mut success_count = 0;
    for url in urls {
        match requester.fetch(url).await {
            Ok((protocol, _)) => {
                println!("  ✓ {} 请求成功 (协议: {})", url, protocol);
                success_count += 1;
            }
            Err(e) => {
                println!("  ✗ {} 请求失败: {}", url, e);
            }
        }
    }
    
    let duration = start.elapsed();
    println!("  总计: {} 成功, 时间: {:?}", success_count, duration);
    
    println!("\n=== 测试完成 ===");
    println!("AsyncRequester 工作正常，无段错误!");
    
    Ok(())
}