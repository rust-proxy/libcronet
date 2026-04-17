//! HTTP/3 QUIC 示例
//! 展示如何使用 Cronet 的 HTTP/3 (QUIC) 功能
//! 
//! 注意：HTTP/3 需要服务器支持，并且 Cronet 需要正确配置

use cronet_rs::async_wrapper::AsyncRequester;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HTTP/3 QUIC 示例 ===\n");
    
    // 1. 创建异步请求器
    println!("1. 创建 AsyncRequester...");
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("   ✅ 创建成功");
    println!("   引擎版本: {}\n", requester.engine_version()?);
    
    // 2. 测试支持 HTTP/3 的网站
    println!("2. 测试支持 HTTP/3 的网站:");
    
    // 已知支持 HTTP/3 的网站列表
    let http3_sites = vec![
        ("Cloudflare", "https://cloudflare.com"),
        ("Google", "https://www.google.com"),
        ("Facebook", "https://www.facebook.com"),
        ("YouTube", "https://www.youtube.com"),
        ("QUIC.Cloud", "https://quic.cloud"),
    ];
    
    for (name, url) in http3_sites {
        println!("\n   测试 {} ({})...", name, url);
        
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                println!("     ✅ 请求成功!");
                println!("       协议: {}", protocol);
                println!("       大小: {} 字节", data.len());
                
                // 检查是否使用了 HTTP/3
                if protocol.to_lowercase().contains("h3") || protocol.to_lowercase().contains("quic") {
                    println!("       🎉 使用了 HTTP/3 (QUIC)!");
                } else if protocol.to_lowercase().contains("h2") {
                    println!("       使用了 HTTP/2");
                } else if protocol.to_lowercase().contains("http/1") {
                    println!("       使用了 HTTP/1.1");
                } else {
                    println!("       协议: {}", protocol);
                }
            }
            Err(e) => {
                println!("     ❌ 请求失败: {}", e);
            }
        }
        
        // 短暂延迟，避免请求过快
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // 3. 专门测试 QUIC 性能
    println!("\n3. QUIC 性能测试:");
    
    // 使用一个已知支持 QUIC 的测试端点
    let quic_test_urls = vec![
        "https://http3.is/",  // HTTP/3 测试网站
        "https://h3.fastly.com/",
    ];
    
    for url in quic_test_urls {
        println!("\n   测试 {}...", url);
        
        let start_time = std::time::Instant::now();
        
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                let duration = start_time.elapsed();
                println!("     ✅ 请求成功!");
                println!("       协议: {}", protocol);
                println!("       大小: {} 字节", data.len());
                println!("       耗时: {:?}", duration);
                
                // 计算速度
                let size_kb = data.len() as f64 / 1024.0;
                let speed_kbps = size_kb / duration.as_secs_f64();
                println!("       速度: {:.2} KB/s", speed_kbps);
                
                // 检查是否为 HTTP/3
                let is_http3 = protocol.to_lowercase().contains("h3") || 
                              protocol.to_lowercase().contains("quic");
                
                if is_http3 {
                    println!("       🚀 HTTP/3 (QUIC) 激活!");
                    
                    // HTTP/3 的优势通常体现在：
                    // 1. 更快的连接建立（0-RTT）
                    // 2. 更好的多路复用
                    // 3. 改进的拥塞控制
                    println!("       HTTP/3 优势:");
                    println!("       • 更快的连接建立 (0-RTT)");
                    println!("       • 改进的多路复用");
                    println!("       • 更好的移动网络性能");
                }
            }
            Err(e) => {
                println!("     ❌ 请求失败: {}", e);
                println!("       可能原因:");
                println!("       • 服务器不支持 HTTP/3");
                println!("       • 网络配置问题");
                println!("       • 防火墙阻止了 UDP 443 端口 (QUIC 使用 UDP)");
            }
        }
    }
    
    // 4. 多次请求测试连接复用
    println!("\n4. 连接复用测试 (HTTP/3 的优势):");
    
    let test_url = "https://http3.is/";
    println!("   对 {} 进行多次请求...", test_url);
    
    let mut protocols = Vec::new();
    let mut total_size = 0;
    let total_start = std::time::Instant::now();
    
    for i in 0..5 {
        let request_start = std::time::Instant::now();
        
        match requester.fetch(test_url).await {
            Ok((protocol, data)) => {
                let request_duration = request_start.elapsed();
                protocols.push(protocol.clone());
                total_size += data.len();
                
                println!("     请求 {}: 协议={}, 大小={} 字节, 耗时={:?}", 
                    i + 1, protocol, data.len(), request_duration);
            }
            Err(e) => {
                println!("     请求 {} 失败: {}", i + 1, e);
            }
        }
        
        // 短暂延迟
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    
    let total_duration = total_start.elapsed();
    println!("\n   统计信息:");
    println!("     总请求数: 5");
    println!("     总数据量: {} 字节", total_size);
    println!("     总耗时: {:?}", total_duration);
    println!("     平均每个请求: {:?}", total_duration / 5);
    
    // 检查是否保持了相同的协议
    if !protocols.is_empty() {
        let first_protocol = &protocols[0];
        let same_protocol = protocols.iter().all(|p| p == first_protocol);
        
        if same_protocol {
            println!("     所有请求使用相同协议: {}", first_protocol);
            if first_protocol.to_lowercase().contains("h3") {
                println!("     ✅ HTTP/3 连接复用正常工作!");
            }
        } else {
            println!("     协议变化: {:?}", protocols);
            println!("     注意: 协议可能在 HTTP/2 和 HTTP/3 之间切换");
        }
    }
    
    // 5. 大文件下载测试（展示 HTTP/3 的多路复用优势）
    println!("\n5. 大文件下载测试:");
    
    // 使用一个返回较大文件的测试端点
    let large_file_url = "https://http3.is/";
    println!("   下载测试: {}", large_file_url);
    
    let download_start = std::time::Instant::now();
    match requester.fetch(large_file_url).await {
        Ok((protocol, data)) => {
            let download_duration = download_start.elapsed();
            let size_kb = data.len() as f64 / 1024.0;
            let speed_kbps = size_kb / download_duration.as_secs_f64();
            
            println!("     ✅ 下载成功!");
            println!("       协议: {}", protocol);
            println!("       大小: {:.2} KB", size_kb);
            println!("       耗时: {:?}", download_duration);
            println!("       速度: {:.2} KB/s", speed_kbps);
            
            if protocol.to_lowercase().contains("h3") {
                println!("       🚀 HTTP/3 表现良好!");
                println!("       HTTP/3 在大文件下载中的优势:");
                println!("       • 更好的多路复用，减少队头阻塞");
                println!("       • 改进的拥塞控制算法");
                println!("       • 更快的恢复从丢包");
            }
        }
        Err(e) => {
            println!("     ❌ 下载失败: {}", e);
        }
    }
    
    // 6. HTTP/3 配置建议
    println!("\n6. HTTP/3 配置建议:");
    println!("   要启用 HTTP/3，需要:");
    println!("   1. 服务器支持 HTTP/3");
    println!("   2. 客户端（Cronet）启用 QUIC 支持");
    println!("   3. 网络允许 UDP 443 端口流量");
    println!("   4. 没有中间设备干扰 QUIC 流量");
    println!("\n   验证 HTTP/3 是否工作:");
    println!("   • 检查协议字段是否包含 'h3' 或 'quic'");
    println!("   • 使用网络抓包工具查看 UDP 443 流量");
    println!("   • 测试已知支持 HTTP/3 的网站");
    
    println!("\n=== HTTP/3 示例完成 ===");
    println!("✅ HTTP/3 (QUIC) 测试完成!");
    println!("✅ 如果看到 'h3' 或 'quic' 协议，表示 HTTP/3 已启用!");
    println!("✅ Cronet 自动协商最佳可用协议!");
    
    Ok(())
}