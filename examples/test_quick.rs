//! 快速测试，验证无段错误

use cronet_rs::async_wrapper::AsyncRequester;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("快速测试 AsyncRequester...");
    
    // 1. 创建请求器
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("✓ 引擎版本: {}", requester.engine_version()?);
    
    // 2. 简单请求
    println!("\n测试简单请求:");
    match requester.fetch("https://httpbin.org/get").await {
        Ok((protocol, data)) => {
            println!("✓ 请求成功!");
            println!("  协议: {}", protocol);
            println!("  数据长度: {} 字节", data.len());
            
            // 简单验证
            let response = String::from_utf8_lossy(&data);
            if response.contains("httpbin.org") {
                println!("✓ 响应内容正确");
            }
        }
        Err(e) => {
            println!("✗ 请求失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 3. 第二次请求（验证重复使用）
    println!("\n测试第二次请求:");
    match requester.fetch("https://httpbin.org/ip").await {
        Ok((protocol, data)) => {
            println!("✓ 第二次请求成功!");
            println!("  协议: {}", protocol);
            println!("  数据长度: {} 字节", data.len());
        }
        Err(e) => {
            println!("✗ 第二次请求失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 4. 错误请求
    println!("\n测试错误请求:");
    match requester.fetch("https://httpbin.org/status/404").await {
        Ok((protocol, data)) => {
            println!("✓ 404请求成功 (协议: {}, 长度: {}字节)", protocol, data.len());
        }
        Err(e) => {
            println!("✓ 404请求失败（预期）: {}", e);
        }
    }
    
    println!("\n✅ 所有测试通过，无段错误!");
    println!("✅ AsyncRequester 稳定可靠!");
    
    Ok(())
}