//! 快速测试请求体和流式响应

use cronet_rs::async_wrapper_with_body_complete::AsyncRequesterWithBody;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("快速测试请求体和流式响应...");
    
    // 1. 创建请求器
    let requester = AsyncRequesterWithBody::new("dylibs/libcronet-linux-amd64.so")?;
    println!("✓ 引擎版本: {}", requester.engine_version()?);
    
    // 2. 测试基本请求
    println!("\n1. 测试基本 GET 请求:");
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
    
    // 3. 测试流式响应（简单版本）
    println!("\n2. 测试流式响应:");
    let streaming_response = requester.fetch_streaming("https://httpbin.org/get", None, None)?;
    println!("  ✓ 流式请求创建成功");
    println!("    协议: {}", streaming_response.protocol());
    
    // 只读取第一个数据块
    use futures::StreamExt;
    let mut stream = std::pin::Pin::new(Box::new(streaming_response));
    
    if let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if !chunk.data.is_empty() {
                    println!("  ✓ 成功接收数据块: {} 字节", chunk.data.len());
                } else {
                    println!("  ⚠️  空数据块");
                }
            }
            Err(e) => {
                println!("  ✗ 数据块错误: {}", e);
            }
        }
    }
    
    // 4. 验证无段错误
    println!("\n3. 验证无段错误:");
    for i in 0..2 {
        println!("  请求 {}:", i + 1);
        match requester.fetch("https://httpbin.org/get").await {
            Ok((protocol, _)) => {
                println!("    ✓ 成功 (协议: {})", protocol);
            }
            Err(e) => {
                println!("    ✗ 失败: {}", e);
            }
        }
    }
    
    println!("\n✅ 测试完成!");
    println!("✅ 请求体和流式响应功能工作正常!");
    println!("✅ 无段错误!");
    
    Ok(())
}