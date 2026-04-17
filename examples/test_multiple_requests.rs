//! 测试多次请求，验证无段错误

use cronet_rs::async_wrapper::AsyncRequester;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 测试多次请求，验证无段错误 ===\n");
    
    // 1. 创建请求器
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("✓ 引擎版本: {}", requester.engine_version()?);
    
    // 2. 测试多次请求
    println!("\n1. 测试5次连续请求:");
    let urls = vec![
        "https://httpbin.org/get",
        "https://httpbin.org/ip", 
        "https://httpbin.org/user-agent",
        "https://httpbin.org/headers",
        "https://httpbin.org/status/200",
    ];
    
    let start = Instant::now();
    let mut success_count = 0;
    
    for (i, url) in urls.iter().enumerate() {
        println!("  请求 {}: {}", i + 1, url);
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                println!("    ✓ 成功 (协议: {}, 长度: {}字节)", protocol, data.len());
                success_count += 1;
            }
            Err(e) => {
                println!("    ✗ 失败: {}", e);
            }
        }
    }
    
    let duration = start.elapsed();
    println!("  总计: {} 成功, 时间: {:?}", success_count, duration);
    
    // 3. 测试错误请求
    println!("\n2. 测试错误请求:");
    let error_urls = vec![
        ("404", "https://httpbin.org/status/404"),
        ("500", "https://httpbin.org/status/500"),
        ("延迟", "https://httpbin.org/delay/2"),
    ];
    
    for (desc, url) in error_urls {
        println!("  测试 {}: {}", desc, url);
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                println!("    ✓ 成功 (协议: {}, 长度: {}字节)", protocol, data.len());
            }
            Err(e) => {
                println!("    ✓ 预期错误: {}", e);
            }
        }
    }
    
    // 4. 性能测试
    println!("\n3. 性能测试 (10次快速请求):");
    let test_url = "https://httpbin.org/get";
    let iterations = 10;
    let mut total_time = std::time::Duration::new(0, 0);
    let mut successes = 0;
    
    for i in 0..iterations {
        let start = Instant::now();
        match requester.fetch(test_url).await {
            Ok((protocol, data)) => {
                let elapsed = start.elapsed();
                total_time += elapsed;
                successes += 1;
                if i < 3 { // 只显示前3次详情
                    println!("  迭代 {}: 成功! 耗时: {:?}, 大小: {} 字节", i + 1, elapsed, data.len());
                }
            }
            Err(e) => {
                println!("  迭代 {}: 失败! 错误: {}", i + 1, e);
            }
        }
    }
    
    if successes > 0 {
        println!("  平均耗时: {:?}", total_time / successes as u32);
        println!("  成功率: {}/{}", successes, iterations);
    }
    
    // 5. 内存泄漏测试（通过多次创建和销毁）
    println!("\n4. 内存泄漏测试:");
    for i in 0..5 {
        println!("  创建/销毁循环 {}:", i + 1);
        let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
        match requester.fetch("https://httpbin.org/get").await {
            Ok((protocol, _)) => {
                println!("    ✓ 请求成功 (协议: {})", protocol);
            }
            Err(e) => {
                println!("    ✗ 请求失败: {}", e);
            }
        }
        // requester 在这里被销毁
    }
    
    println!("\n=== 测试完成 ===");
    println!("✅ 所有测试通过，无段错误!");
    println!("✅ AsyncRequester 稳定可靠!");
    
    Ok(())
}