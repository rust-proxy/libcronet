//! 错误处理示例
//! 展示如何处理各种 HTTP 错误和网络问题

use cronet_rs::async_wrapper::AsyncRequester;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 错误处理示例 ===\n");
    
    // 1. 创建异步请求器
    println!("1. 创建 AsyncRequester...");
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("   ✅ 创建成功\n");
    
    // 2. 测试各种错误情况
    println!("2. 测试各种错误情况:");
    
    // 2.1 404 错误
    println!("\n   a) 测试 404 错误:");
    match requester.fetch("https://httpbin.org/status/404").await {
        Ok((protocol, data)) => {
            println!("     收到响应 (协议: {}, 大小: {} 字节)", protocol, data.len());
            println!("     注意: httpbin.org 的 /status/404 端点返回 404 状态码但请求成功");
        }
        Err(e) => {
            println!("     ❌ 请求失败: {}", e);
        }
    }
    
    // 2.2 500 错误
    println!("\n   b) 测试 500 错误:");
    match requester.fetch("https://httpbin.org/status/500").await {
        Ok((protocol, data)) => {
            println!("     收到响应 (协议: {}, 大小: {} 字节)", protocol, data.len());
        }
        Err(e) => {
            println!("     ❌ 请求失败: {}", e);
        }
    }
    
    // 2.3 超时测试（使用延迟端点）
    println!("\n   c) 测试延迟响应:");
    match requester.fetch("https://httpbin.org/delay/1").await {
        Ok((protocol, data)) => {
            println!("     ✅ 延迟请求成功 (协议: {}, 大小: {} 字节)", protocol, data.len());
        }
        Err(e) => {
            println!("     ❌ 延迟请求失败: {}", e);
        }
    }
    
    // 2.4 无效 URL
    println!("\n   d) 测试无效 URL:");
    match requester.fetch("not-a-valid-url").await {
        Ok((protocol, data)) => {
            println!("     意外成功 (协议: {}, 大小: {} 字节)", protocol, data.len());
        }
        Err(e) => {
            println!("     ✅ 如预期失败: {}", e);
        }
    }
    
    // 2.5 不存在的域名
    println!("\n   e) 测试不存在的域名:");
    match requester.fetch("https://this-domain-does-not-exist-12345.example.com").await {
        Ok((protocol, data)) => {
            println!("     意外成功 (协议: {}, 大小: {} 字节)", protocol, data.len());
        }
        Err(e) => {
            println!("     ✅ 如预期失败: {}", e);
            println!("     错误类型: {:?}", e);
        }
    }
    
    // 2.6 连接被拒绝（本地端口）
    println!("\n   f) 测试连接被拒绝:");
    match requester.fetch("http://localhost:9999").await {
        Ok((protocol, data)) => {
            println!("     意外成功 (协议: {}, 大小: {} 字节)", protocol, data.len());
        }
        Err(e) => {
            println!("     ✅ 如预期失败: {}", e);
        }
    }
    
    // 3. 错误恢复策略
    println!("\n3. 错误恢复策略示例:");
    
    let urls_to_try = vec![
        "https://httpbin.org/get",           // 应该成功
        "https://httpbin.org/status/404",    // 404但请求成功
        "https://invalid-url-12345.com",     // 应该失败
        "https://httpbin.org/ip",            // 应该成功
    ];
    
    for (i, url) in urls_to_try.iter().enumerate() {
        println!("   尝试 URL {}: {}", i + 1, url);
        
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                println!("     ✅ 成功 (协议: {}, 大小: {} 字节)", protocol, data.len());
            }
            Err(e) => {
                println!("     ❌ 失败: {}", e);
                
                // 根据错误类型采取不同策略
                let error_str = e.to_string();
                if error_str.contains("network") || error_str.contains("connect") {
                    println!("     网络错误，可以重试或使用备用URL");
                } else if error_str.contains("timeout") {
                    println!("     超时错误，可以增加超时时间");
                } else {
                    println!("     其他错误，检查URL或配置");
                }
            }
        }
    }
    
    // 4. 批量请求的错误处理
    println!("\n4. 批量请求的错误处理:");
    
    let batch_urls = vec![
        "https://httpbin.org/get",
        "https://httpbin.org/status/500",
        "https://httpbin.org/delay/2",
        "https://invalid.example.com",
    ];
    
    for url in batch_urls {
        println!("   请求: {}", url);
        let result = requester.fetch(url).await;
        
        match result {
            Ok((protocol, _)) => {
                println!("     ✅ 成功 (协议: {})", protocol);
            }
            Err(e) => {
                println!("     ❌ 失败: {}", e);
                // 在实际应用中，这里可以记录日志、重试或使用降级策略
            }
        }
    }
    
    println!("\n=== 示例完成 ===");
    println!("✅ 错误处理测试完成!");
    println!("✅ 各种错误情况都已测试!");
    println!("✅ 错误恢复策略演示完成!");
    
    Ok(())
}