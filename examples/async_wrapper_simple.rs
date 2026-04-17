//! 简单异步包装器示例

use cronet_rs::async_wrapper::AsyncRequester;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 简单异步包装器示例 ===");
    
    // 1. 创建异步请求器
    println!("1. 创建异步请求器...");
    let start = Instant::now();
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("   创建耗时: {:?}", start.elapsed());
    
    println!("   Cronet 引擎版本: {}", requester.engine_version()?);
    
    // 2. 顺序请求示例
    println!("\n2. 顺序请求测试...");
    
    let urls = vec![
        "https://httpbin.org/get",
        "https://httpbin.org/ip",
        "https://httpbin.org/user-agent",
    ];
    
    let start = Instant::now();
    let mut total_bytes = 0;
    
    for (i, url) in urls.iter().enumerate() {
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                println!("   请求 {}: 成功! 协议: {}, 大小: {} 字节", i + 1, protocol, data.len());
                total_bytes += data.len();
                
                // 显示部分内容
                if data.len() < 300 {
                    println!("     内容: {}", String::from_utf8_lossy(&data));
                }
            }
            Err(e) => {
                println!("   请求 {}: 失败! 错误: {}", i + 1, e);
            }
        }
    }
    
    println!("   顺序请求总耗时: {:?}", start.elapsed());
    println!("   总下载字节数: {} 字节", total_bytes);
    
    // 3. HTTP/3 测试
    println!("\n3. HTTP/3 测试...");
    
    let http3_urls = vec![
        "https://cloudflare-quic.com/",
        "https://quic.nginx.org/",
    ];
    
    for (i, url) in http3_urls.iter().enumerate() {
        println!("   测试 {}: {}", i + 1, url);
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                println!("     成功! 协议: {}, 大小: {} 字节", protocol, data.len());
                if protocol.contains("h3") {
                    println!("     ✅ 使用 HTTP/3!");
                } else {
                    println!("     ⚠️  回退到: {}", protocol);
                }
            }
            Err(e) => {
                println!("     失败: {}", e);
            }
        }
    }
    
    // 4. 错误处理
    println!("\n4. 错误处理测试...");
    
    let error_cases = vec![
        ("404 错误", "https://httpbin.org/status/404"),
        ("500 错误", "https://httpbin.org/status/500"),
        ("超时", "https://httpbin.org/delay/3"),
    ];
    
    for (desc, url) in error_cases {
        println!("   测试: {}", desc);
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                println!("     成功 (意外): 协议: {}, 大小: {} 字节", protocol, data.len());
            }
            Err(e) => {
                println!("     预期错误: {}", e);
            }
        }
    }
    
    // 5. 性能测试
    println!("\n5. 性能测试...");
    
    let test_url = "https://httpbin.org/get";
    let iterations = 3;
    let mut total_time = std::time::Duration::new(0, 0);
    
    for i in 0..iterations {
        let start = Instant::now();
        match requester.fetch(test_url).await {
            Ok((protocol, data)) => {
                let elapsed = start.elapsed();
                total_time += elapsed;
                println!("   迭代 {}: 成功! 耗时: {:?}, 大小: {} 字节", i + 1, elapsed, data.len());
            }
            Err(e) => {
                println!("   迭代 {}: 失败! 错误: {}", i + 1, e);
            }
        }
    }
    
    if iterations > 0 {
        println!("   平均耗时: {:?}", total_time / iterations as u32);
    }
    
    println!("\n=== 示例完成 ===");
    println!("✅ 异步包装器工作正常，无段错误!");
    Ok(())
}