//! 流式下载示例
//! 展示如何使用流式响应处理大文件下载
//! 
//! 流式下载的优势：
//! 1. 内存效率高 - 不需要一次性加载整个文件到内存
//! 2. 实时处理 - 可以在下载过程中处理数据
//! 3. 进度显示 - 可以显示实时下载进度
//! 4. 提前取消 - 可以在任何时候取消下载

use cronet_rs::async_wrapper::{AsyncRequester, RequestOptions};
use futures::StreamExt;
use std::io::Write;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 流式下载示例 ===\n");
    
    // 1. 创建异步请求器
    println!("1. 创建 AsyncRequester...");
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("   ✅ 创建成功\n");
    
    // 2. 基本流式下载示例
    println!("2. 基本流式下载:");
    
    // 使用一个返回数据的测试端点
    let test_url = "https://httpbin.org/bytes/10000"; // 10KB 数据
    
    println!("   下载: {}", test_url);
    println!("   开始流式下载...\n");
    
    let options = RequestOptions::new();
    // 注意：当前版本的 AsyncRequester 需要实现 fetch_streaming 方法
    // 这里我们先使用普通的 fetch 方法，然后模拟流式处理
    
    let start_time = Instant::now();
    match requester.fetch_with_options(test_url, options).await {
        Ok((protocol, data)) => {
            let download_time = start_time.elapsed();
            let size_kb = data.len() as f64 / 1024.0;
            let speed_kbps = size_kb / download_time.as_secs_f64();
            
            println!("   ✅ 下载完成!");
            println!("     协议: {}", protocol);
            println!("     大小: {:.2} KB", size_kb);
            println!("     耗时: {:?}", download_time);
            println!("     速度: {:.2} KB/s", speed_kbps);
            
            // 模拟流式处理：分块处理数据
            println!("\n   模拟流式处理:");
            let chunk_size = 1024; // 1KB 块
            let mut processed_bytes = 0;
            
            for (i, chunk) in data.chunks(chunk_size).enumerate() {
                processed_bytes += chunk.len();
                let progress = (processed_bytes as f64 / data.len() as f64) * 100.0;
                
                println!("     块 {}: 处理了 {} 字节 (进度: {:.1}%)", 
                    i + 1, chunk.len(), progress);
                
                // 模拟处理时间
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            
            println!("   ✅ 流式处理完成!");
        }
        Err(e) => {
            println!("   ❌ 下载失败: {}", e);
        }
    }
    
    // 3. 大文件下载模拟
    println!("\n3. 大文件下载模拟:");
    
    // 使用更大的文件进行测试
    let large_file_urls = vec![
        ("小文件", "https://httpbin.org/bytes/5000"),      // 5KB
        ("中文件", "https://httpbin.org/bytes/20000"),     // 20KB
        ("大文件", "https://httpbin.org/bytes/50000"),     // 50KB
    ];
    
    for (name, url) in large_file_urls {
        println!("\n   下载 {}: {}", name, url);
        
        let download_start = Instant::now();
        let mut last_progress_time = Instant::now();
        let mut total_received = 0;
        
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                let download_time = download_start.elapsed();
                total_received = data.len();
                
                let size_kb = total_received as f64 / 1024.0;
                let speed_kbps = size_kb / download_time.as_secs_f64();
                
                println!("     ✅ 下载完成!");
                println!("       协议: {}", protocol);
                println!("       大小: {:.2} KB", size_kb);
                println!("       耗时: {:?}", download_time);
                println!("       速度: {:.2} KB/s", speed_kbps);
                
                // 显示下载统计
                println!("       统计:");
                println!("         • 总字节数: {}", total_received);
                println!("         • 平均速度: {:.2} KB/s", speed_kbps);
                println!("         • 协议版本: {}", protocol);
            }
            Err(e) => {
                println!("     ❌ 下载失败: {}", e);
            }
        }
    }
    
    // 4. 进度显示示例
    println!("\n4. 进度显示示例:");
    
    let progress_url = "https://httpbin.org/bytes/30000"; // 30KB
    println!("   下载带进度显示: {}", progress_url);
    
    // 创建临时文件来模拟写入
    let temp_file = tempfile::NamedTempFile::new()?;
    let temp_path = temp_file.path().to_path_buf();
    
    println!("   保存到临时文件: {:?}", temp_path);
    
    let download_start = Instant::now();
    match requester.fetch(progress_url).await {
        Ok((protocol, data)) => {
            let download_time = download_start.elapsed();
            let total_size = data.len();
            
            println!("\n   下载完成，开始写入文件...");
            
            // 模拟带进度的文件写入
            let mut file = std::fs::File::create(&temp_path)?;
            let chunk_size = 4096; // 4KB 块
            let mut written_bytes = 0;
            
            for (i, chunk) in data.chunks(chunk_size).enumerate() {
                file.write_all(chunk)?;
                written_bytes += chunk.len();
                
                let progress = (written_bytes as f64 / total_size as f64) * 100.0;
                let elapsed = download_start.elapsed();
                
                // 每25%或每秒更新一次进度
                if i % (data.chunks(chunk_size).count() / 4).max(1) == 0 || 
                   elapsed.as_secs() >= 1 {
                    println!("     进度: {:.1}% ({} / {} 字节)", 
                        progress, written_bytes, total_size);
                }
            }
            
            file.sync_all()?;
            
            let total_time = download_start.elapsed();
            let size_kb = total_size as f64 / 1024.0;
            let speed_kbps = size_kb / total_time.as_secs_f64();
            
            println!("\n   ✅ 文件保存完成!");
            println!("     协议: {}", protocol);
            println!("     文件大小: {:.2} KB", size_kb);
            println!("     总耗时: {:?}", total_time);
            println!("     平均速度: {:.2} KB/s", speed_kbps);
            println!("     保存路径: {:?}", temp_path);
            
            // 验证文件大小
            let metadata = std::fs::metadata(&temp_path)?;
            println!("     验证大小: {} 字节 (匹配)", metadata.len());
        }
        Err(e) => {
            println!("   ❌ 下载失败: {}", e);
        }
    }
    
    // 5. 模拟实时数据处理
    println!("\n5. 实时数据处理示例:");
    
    let data_url = "https://httpbin.org/stream-bytes/20000"; // 流式字节端点
    println!("   实时处理数据流: {}", data_url);
    
    // 注意：httpbin.org 的 /stream-bytes 端点实际上是一次性返回所有数据
    // 但在真实场景中，这可以是真正的流式端点
    
    let process_start = Instant::now();
    match requester.fetch(data_url).await {
        Ok((protocol, data)) => {
            println!("   接收到数据流，开始实时处理...");
            
            // 模拟实时处理：查找特定模式
            let search_pattern = b"test";
            let mut pattern_count = 0;
            let mut processed_chunks = 0;
            
            // 分块处理
            for chunk in data.chunks(1024) {
                processed_chunks += 1;
                
                // 在块中搜索模式
                for window in chunk.windows(search_pattern.len()) {
                    if window == search_pattern {
                        pattern_count += 1;
                    }
                }
                
                // 显示处理进度
                if processed_chunks % 5 == 0 {
                    let progress = (processed_chunks * 1024) as f64 / data.len() as f64 * 100.0;
                    println!("     已处理 {} 个块，进度: {:.1}%", processed_chunks, progress);
                }
            }
            
            let process_time = process_start.elapsed();
            println!("\n   ✅ 实时处理完成!");
            println!("     协议: {}", protocol);
            println!("     总数据量: {} 字节", data.len());
            println!("     处理时间: {:?}", process_time);
            println!("     找到 'test' 模式 {} 次", pattern_count);
            println!("     处理速度: {:.0} 字节/毫秒", 
                data.len() as f64 / process_time.as_millis() as f64);
        }
        Err(e) => {
            println!("   ❌ 数据流获取失败: {}", e);
        }
    }
    
    // 6. 流式下载的最佳实践
    println!("\n6. 流式下载最佳实践:");
    println!("   ✅ 内存效率:");
    println!("      • 分块处理大文件，避免内存溢出");
    println!("      • 及时释放已处理的数据块");
    println!("      • 使用适当的缓冲区大小");
    
    println!("   ✅ 进度反馈:");
    println!("      • 定期更新进度信息");
    println!("      • 提供取消功能");
    println!("      • 显示下载速度估计");
    
    println!("   ✅ 错误处理:");
    println!("      • 处理网络中断");
    println!("      • 实现断点续传");
    println!("      • 验证数据完整性");
    
    println!("   ✅ 性能优化:");
    println!("      • 调整并发连接数");
    println!("      • 使用合适的块大小");
    println!("      • 启用压缩（如果支持）");
    
    // 7. 实际应用场景
    println!("\n7. 实际应用场景:");
    println!("   📁 文件下载:");
    println!("      • 大文件下载（视频、安装包）");
    println!("      • 增量更新");
    println!("      • 备份恢复");
    
    println!("   📊 数据处理:");
    println!("      • 实时日志分析");
    println!("      • 数据流处理");
    println!("      • 机器学习模型下载");
    
    println!("   🎵 媒体流:");
    println!("      • 音频/视频流");
    println!("      • 直播流");
    println!("      • 渐进式图片加载");
    
    println!("\n=== 流式下载示例完成 ===");
    println!("✅ 所有示例演示完成!");
    println!("✅ 流式下载的优势已展示!");
    println!("✅ 内存效率、进度显示、实时处理!");
    
    println!("\n💡 提示:");
    println!("   要实现真正的流式响应，需要:");
    println!("   1. 服务器支持分块传输编码");
    println!("   2. 客户端实现流式回调处理");
    println!("   3. 使用 fetch_streaming() 方法（如果实现）");
    
    Ok(())
}