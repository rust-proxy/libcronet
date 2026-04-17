//! 基于回调的异步包装器示例

use cronet_rs::async_wrapper::AsyncRequester;
use tokio::runtime::Runtime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 基于回调的异步包装器示例 ===");
    
    // 1. 创建异步请求器
    println!("1. 创建异步请求器...");
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    
    println!("Cronet 引擎版本: {}", requester.engine_version()?);
    
    // 2. 发送简单请求
    println!("\n2. 发送简单请求到 httpbin.org...");
    match requester.fetch("https://httpbin.org/get").await {
        Ok((protocol, data)) => {
            println!("成功! 协议: {}, 数据大小: {} 字节", protocol, data.len());
            if data.len() < 500 {
                println!("响应: {}", String::from_utf8_lossy(&data));
            } else {
                println!("响应太大，显示前500字符:");
                println!("{}", String::from_utf8_lossy(&data[..500.min(data.len())]));
            }
        }
        Err(e) => {
            println!("错误: {}", e);
        }
    }
    
    // 3. 测试 HTTP/3
    println!("\n3. 测试 HTTP/3 请求...");
    match requester.fetch("https://cloudflare-quic.com/").await {
        Ok((protocol, data)) => {
            println!("HTTP/3 测试成功! 协议: {}, 数据大小: {} 字节", protocol, data.len());
            if protocol.contains("h3") {
                println!("✅ 成功使用 HTTP/3!");
            } else {
                println!("⚠️  回退到: {}", protocol);
            }
        }
        Err(e) => {
            println!("HTTP/3 测试错误: {}", e);
        }
    }
    
    // 4. 测试错误情况
    println!("\n4. 测试错误请求...");
    match requester.fetch("https://httpbin.org/status/404").await {
        Ok((protocol, data)) => {
            println!("404 请求成功 (但应该失败)");
            println!("协议: {}, 数据大小: {} 字节", protocol, data.len());
        }
        Err(e) => {
            println!("预期错误: {}", e);
        }
    }
    
    println!("\n=== 示例完成 ===");
    Ok(())
}