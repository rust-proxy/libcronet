//! 高级异步包装器示例

use cronet_rs::async_wrapper::AsyncRequester;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 高级异步包装器示例 ===");
    
    // 1. 创建异步请求器
    println!("1. 创建异步请求器...");
    let start = Instant::now();
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("   创建耗时: {:?}", start.elapsed());
    
    println!("   Cronet 引擎版本: {}", requester.engine_version()?);
    
    // 2. 并发请求示例
    println!("\n2. 并发请求测试...");
    
    let urls = vec![
        "https://httpbin.org/get",
        "https://httpbin.org/ip",
        "https://httpbin.org/user-agent",
        "https://httpbin.org/headers",
    ];
    
    let start = Instant::now();
    let mut tasks = Vec::new();
    
    for (i, url) in urls.iter().enumerate() {
        let requester = &requester;
        let url = url.to_string();
        let task = tokio::spawn(async move {
            match requester.fetch(&url).await {
                Ok((protocol, data)) => {
                    println!("   请求 {}: 成功! 协议: {}, 大小: {} 字节", i + 1, protocol, data.len());
                    Ok(data.len())
                }
                Err(e) => {
                    println!("   请求 {}: 失败! 错误: {}", i + 1, e);
                    Err(e)
                }
            }
        });
        tasks.push(task);
    }
    
    // 等待所有任务完成
    let mut total_bytes = 0;
    for task in tasks {
        match task.await {
            Ok(Ok(bytes)) => total_bytes += bytes,
            Ok(Err(e)) => println!("   任务错误: {}", e),
            Err(e) => println!("   Join 错误: {}", e),
        }
    }
    
    println!("   并发请求总耗时: {:?}", start.elapsed());
    println!("   总下载字节数: {} 字节", total_bytes);
    
    // 3. 错误处理示例
    println!("\n3. 错误处理测试...");
    
    let error_urls = vec![
        "https://httpbin.org/status/404",
        "https://httpbin.org/status/500",
        "https://httpbin.org/delay/5", // 超时测试
        "https://invalid-domain-that-does-not-exist.example.com/",
    ];
    
    for (i, url) in error_urls.iter().enumerate() {
        println!("   测试 {}: {}", i + 1, url);
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                println!("     成功 (意外): 协议: {}, 大小: {} 字节", protocol, data.len());
            }
            Err(e) => {
                println!("     预期错误: {}", e);
            }
        }
    }
    
    // 4. 性能测试
    println!("\n4. 性能测试 (重复请求)...");
    
    let test_url = "https://httpbin.org/get";
    let iterations = 5;
    let mut total_time = std::time::Duration::new(0, 0);
    
    for i in 0..iterations {
        let start = Instant::now();
        match requester.fetch(test_url).await {
            Ok((protocol, data)) => {
                let elapsed = start.elapsed();
                total_time += elapsed;
                println!("   迭代 {}: 成功! 协议: {}, 大小: {} 字节, 耗时: {:?}", 
                    i + 1, protocol, data.len(), elapsed);
            }
            Err(e) => {
                println!("   迭代 {}: 失败! 错误: {}", i + 1, e);
            }
        }
    }
    
    println!("   平均耗时: {:?}", total_time / iterations as u32);
    
    // 5. 大文件下载测试
    println!("\n5. 大文件下载测试...");
    
    let large_file_url = "https://httpbin.org/bytes/100000"; // 100KB 文件
    let start = Instant::now();
    
    match requester.fetch(large_file_url).await {
        Ok((protocol, data)) => {
            let elapsed = start.elapsed();
            let speed = data.len() as f64 / elapsed.as_secs_f64() / 1024.0; // KB/s
            println!("   大文件下载成功!");
            println!("     协议: {}", protocol);
            println!("     大小: {} 字节", data.len());
            println!("     耗时: {:?}", elapsed);
            println!("     速度: {:.2} KB/s", speed);
        }
        Err(e) => {
            println!("   大文件下载失败: {}", e);
        }
    }
    
    println!("\n=== 示例完成 ===");
    Ok(())
}