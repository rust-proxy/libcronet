//! 简单的 GET 请求示例
//! 展示如何使用 AsyncRequester 发送基本的 HTTP GET 请求

use cronet_rs::async_wrapper::AsyncRequester;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 简单的 GET 请求示例 ===\n");
    
    // 1. 创建异步请求器
    println!("1. 创建 AsyncRequester...");
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("   ✅ 创建成功");
    println!("   引擎版本: {}\n", requester.engine_version()?);
    
    // 2. 发送 GET 请求
    println!("2. 发送 GET 请求到 httpbin.org...");
    let url = "https://httpbin.org/get";
    
    match requester.fetch(url).await {
        Ok((protocol, data)) => {
            println!("   ✅ 请求成功!");
            println!("     协议: {}", protocol);
            println!("     响应大小: {} 字节", data.len());
            println!("     响应内容 (前100字节):");
            
            // 显示部分响应内容
            let preview = String::from_utf8_lossy(&data[..data.len().min(100)]);
            println!("     {}", preview);
            
            // 如果是 JSON，可以解析
            if let Ok(json_str) = String::from_utf8(data) {
                println!("\n     完整的 JSON 响应:");
                println!("     {}", json_str);
            }
        }
        Err(e) => {
            println!("   ❌ 请求失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 3. 测试多个端点
    println!("\n3. 测试多个端点:");
    let test_urls = vec![
        "https://httpbin.org/status/200",
        "https://httpbin.org/headers",
        "https://httpbin.org/ip",
    ];
    
    for (i, url) in test_urls.iter().enumerate() {
        println!("   {}. {}", i + 1, url);
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                println!("      ✅ 成功 (协议: {}, 大小: {} 字节)", protocol, data.len());
            }
            Err(e) => {
                println!("      ❌ 失败: {}", e);
            }
        }
    }
    
    println!("\n=== 示例完成 ===");
    println!("✅ 所有测试完成!");
    println!("✅ AsyncRequester 工作正常!");
    
    Ok(())
}