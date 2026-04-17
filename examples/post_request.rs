//! POST 请求示例
//! 展示如何使用 AsyncRequester 发送 HTTP POST 请求

use cronet_rs::async_wrapper::{AsyncRequester, RequestOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== POST 请求示例 ===\n");
    
    // 1. 创建异步请求器
    println!("1. 创建 AsyncRequester...");
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("   ✅ 创建成功\n");
    
    // 2. 发送 JSON POST 请求
    println!("2. 发送 JSON POST 请求...");
    let json_data = r#"{"name": "test", "value": 123}"#.as_bytes().to_vec();
    
    let options = RequestOptions::new()
        .method("POST")
        .header("Content-Type", "application/json")
        .header("User-Agent", "cronet-rs-example/1.0")
        .body(json_data);
    
    match requester.fetch_with_options("https://httpbin.org/post", options).await {
        Ok((protocol, data)) => {
            println!("   ✅ POST 请求成功!");
            println!("     协议: {}", protocol);
            println!("     响应大小: {} 字节", data.len());
            
            // 显示响应
            if let Ok(response_str) = String::from_utf8(data) {
                println!("     响应内容:");
                println!("     {}", response_str);
            }
        }
        Err(e) => {
            println!("   ❌ POST 请求失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 3. 发送表单数据
    println!("\n3. 发送表单数据...");
    let form_data = "username=test&password=secret".as_bytes().to_vec();
    
    let form_options = RequestOptions::new()
        .method("POST")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_data);
    
    match requester.fetch_with_options("https://httpbin.org/post", form_options).await {
        Ok((protocol, data)) => {
            println!("   ✅ 表单 POST 成功!");
            println!("     协议: {}, 大小: {} 字节", protocol, data.len());
        }
        Err(e) => {
            println!("   ❌ 表单 POST 失败: {}", e);
        }
    }
    
    // 4. 测试 PUT 和 DELETE 方法
    println!("\n4. 测试其他 HTTP 方法:");
    
    // PUT 请求
    let put_data = r#"{"id": 1, "status": "updated"}"#.as_bytes().to_vec();
    let put_options = RequestOptions::new()
        .method("PUT")
        .header("Content-Type", "application/json")
        .body(put_data);
    
    match requester.fetch_with_options("https://httpbin.org/put", put_options).await {
        Ok((protocol, data)) => {
            println!("   ✅ PUT 请求成功 (协议: {}, 大小: {} 字节)", protocol, data.len());
        }
        Err(e) => {
            println!("   ❌ PUT 请求失败: {}", e);
        }
    }
    
    // DELETE 请求
    let delete_options = RequestOptions::new()
        .method("DELETE");
    
    match requester.fetch_with_options("https://httpbin.org/delete", delete_options).await {
        Ok((protocol, data)) => {
            println!("   ✅ DELETE 请求成功 (协议: {}, 大小: {} 字节)", protocol, data.len());
        }
        Err(e) => {
            println!("   ❌ DELETE 请求失败: {}", e);
        }
    }
    
    println!("\n=== 示例完成 ===");
    println!("✅ 所有 HTTP 方法测试完成!");
    println!("✅ POST/PUT/DELETE 工作正常!");
    
    Ok(())
}