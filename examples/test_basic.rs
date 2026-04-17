//! 基本功能测试

use cronet_rs::async_wrapper::AsyncRequester;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试基本功能...");
    
    // 1. 创建请求器
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("✓ 引擎版本: {}", requester.engine_version()?);
    
    // 2. 简单请求
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
    
    println!("\n✅ 所有测试通过!");
    Ok(())
}