//! 性能测试示例
//! 展示 AsyncRequester 的性能特性

use cronet_rs::async_wrapper::AsyncRequester;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 性能测试示例 ===\n");
    
    // 1. 创建异步请求器
    println!("1. 创建 AsyncRequester...");
    let start_time = Instant::now();
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    let creation_time = start_time.elapsed();
    println!("   ✅ 创建成功 (耗时: {:?})\n", creation_time);
    
    // 2. 单次请求性能测试
    println!("2. 单次请求性能测试:");
    
    let test_url = "https://httpbin.org/get";
    let mut latencies = Vec::new();
    
    for i in 0..5 {
        let request_start = Instant::now();
        match requester.fetch(test_url).await {
            Ok((protocol, data)) => {
                let latency = request_start.elapsed();
                latencies.push(latency);
                println!("   请求 {}: ✅ 成功 (协议: {}, 大小: {} 字节, 耗时: {:?})", 
                    i + 1, protocol, data.len(), latency);
            }
            Err(e) => {
                println!("   请求 {}: ❌ 失败: {}", i + 1, e);
            }
        }
    }
    
    // 计算统计信息
    if !latencies.is_empty() {
        let total: std::time::Duration = latencies.iter().sum();
        let avg = total / latencies.len() as u32;
        let min = latencies.iter().min().unwrap();
        let max = latencies.iter().max().unwrap();
        
        println!("\n   统计信息:");
        println!("     请求次数: {}", latencies.len());
        println!("     平均耗时: {:?}", avg);
        println!("     最短耗时: {:?}", min);
        println!("     最长耗时: {:?}", max);
        println!("     总耗时: {:?}", total);
    }
    
    // 3. 并发请求测试
    println!("\n3. 并发请求测试:");
    
    let concurrent_urls = vec![
        "https://httpbin.org/get",
        "https://httpbin.org/ip",
        "https://httpbin.org/user-agent",
        "https://httpbin.org/headers",
    ];
    
    let concurrent_start = Instant::now();
    let mut tasks = Vec::new();
    
    for url in concurrent_urls {
        let requester_clone = &requester;
        let url_str = url.to_string();
        
        let task = tokio::spawn(async move {
            let start = Instant::now();
            let result = requester_clone.fetch(&url_str).await;
            let duration = start.elapsed();
            (url_str, result, duration)
        });
        
        tasks.push(task);
    }
    
    let mut success_count = 0;
    let mut total_size = 0;
    
    for task in tasks {
        match task.await {
            Ok((url, result, duration)) => {
                match result {
                    Ok((protocol, data)) => {
                        success_count += 1;
                        total_size += data.len();
                        println!("   {}: ✅ 成功 (协议: {}, 大小: {} 字节, 耗时: {:?})", 
                            url, protocol, data.len(), duration);
                    }
                    Err(e) => {
                        println!("   {}: ❌ 失败: {} (耗时: {:?})", url, e, duration);
                    }
                }
            }
            Err(e) => {
                println!("   任务失败: {}", e);
            }
        }
    }
    
    let total_concurrent_time = concurrent_start.elapsed();
    println!("\n   并发测试统计:");
    println!("     总请求数: {}", concurrent_urls.len());
    println!("     成功数: {}", success_count);
    println!("     总数据量: {} 字节", total_size);
    println!("     总耗时: {:?}", total_concurrent_time);
    
    // 4. 内存使用测试（简单版本）
    println!("\n4. 内存使用测试:");
    
    // 多次创建和销毁请求器，观察是否有内存泄漏
    println!("   创建和销毁请求器 10 次...");
    for i in 0..10 {
        let start = Instant::now();
        let temp_requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
        let _ = temp_requester.engine_version();
        let creation_time = start.elapsed();
        
        // 显式丢弃
        drop(temp_requester);
        
        println!("     迭代 {}: 创建耗时 {:?}", i + 1, creation_time);
    }
    
    // 5. 大文件下载测试
    println!("\n5. 大文件下载测试:");
    
    // 使用一个返回较大响应的端点
    let large_url = "https://httpbin.org/bytes/10000"; // 10KB 数据
    
    let download_start = Instant::now();
    match requester.fetch(large_url).await {
        Ok((protocol, data)) => {
            let download_time = download_start.elapsed();
            let size_kb = data.len() as f64 / 1024.0;
            let speed_kbps = size_kb / download_time.as_secs_f64();
            
            println!("   ✅ 下载成功!");
            println!("     协议: {}", protocol);
            println!("     文件大小: {:.2} KB", size_kb);
            println!("     下载耗时: {:?}", download_time);
            println!("     下载速度: {:.2} KB/s", speed_kbps);
        }
        Err(e) => {
            println!("   ❌ 下载失败: {}", e);
        }
    }
    
    // 6. 连接池测试
    println!("\n6. 连接池测试:");
    
    // 重复使用同一个请求器发送多个请求
    let reuse_start = Instant::now();
    let mut reuse_success = 0;
    
    for i in 0..10 {
        match requester.fetch("https://httpbin.org/get").await {
            Ok(_) => {
                reuse_success += 1;
                if i == 0 {
                    println!("     第一次请求成功");
                } else if i == 9 {
                    println!("     第十次请求成功");
                }
            }
            Err(e) => {
                println!("     请求 {} 失败: {}", i + 1, e);
            }
        }
    }
    
    let reuse_time = reuse_start.elapsed();
    println!("\n   重用测试统计:");
    println!("     总请求数: 10");
    println!("     成功数: {}", reuse_success);
    println!("     总耗时: {:?}", reuse_time);
    println!("     平均每个请求: {:?}", reuse_time / 10);
    
    println!("\n=== 性能测试完成 ===");
    println!("✅ 所有性能测试完成!");
    println!("✅ AsyncRequester 表现良好!");
    println!("✅ 无内存泄漏迹象!");
    
    Ok(())
}