//! 集成测试
//! 这些测试需要网络连接和 DLL 文件

use cronet_rs::async_wrapper::{AsyncRequester, RequestOptions};

/// 测试基本的 GET 请求
#[tokio::test]
async fn test_basic_get_request() {
    // 跳过测试如果 DLL 不存在
    let requester = match AsyncRequester::new("dylibs/libcronet-linux-amd64.so") {
        Ok(r) => r,
        Err(_) => {
            println!("Skipping test: DLL not found");
            return;
        }
    };
    
    // 测试获取引擎版本
    let version = requester.engine_version();
    assert!(version.is_ok(), "Should get engine version");
    println!("Engine version: {}", version.unwrap());
    
    // 测试简单的 GET 请求
    let result = requester.fetch("https://httpbin.org/get").await;
    assert!(result.is_ok(), "GET request should succeed");
    
    let (protocol, data) = result.unwrap();
    assert!(!protocol.is_empty(), "Protocol should not be empty");
    assert!(!data.is_empty(), "Response data should not be empty");
    
    println!("GET request successful: protocol={}, size={} bytes", protocol, data.len());
}

/// 测试 POST 请求
#[tokio::test]
async fn test_post_request() {
    let requester = match AsyncRequester::new("dylibs/libcronet-linux-amd64.so") {
        Ok(r) => r,
        Err(_) => {
            println!("Skipping test: DLL not found");
            return;
        }
    };
    
    let json_data = r#"{"test": "data"}"#.as_bytes().to_vec();
    
    let options = RequestOptions::new()
        .method("POST")
        .header("Content-Type", "application/json")
        .body(json_data);
    
    let result = requester.fetch_with_options("https://httpbin.org/post", options).await;
    assert!(result.is_ok(), "POST request should succeed");
    
    let (protocol, data) = result.unwrap();
    assert!(!protocol.is_empty(), "Protocol should not be empty");
    assert!(!data.is_empty(), "Response data should not be empty");
    
    // 验证响应包含我们发送的数据
    let response_str = String::from_utf8_lossy(&data);
    assert!(response_str.contains("\"test\": \"data\""), "Response should contain sent data");
    
    println!("POST request successful: protocol={}, size={} bytes", protocol, data.len());
}

/// 测试错误处理
#[tokio::test]
async fn test_error_handling() {
    let requester = match AsyncRequester::new("dylibs/libcronet-linux-amd64.so") {
        Ok(r) => r,
        Err(_) => {
            println!("Skipping test: DLL not found");
            return;
        }
    };
    
    // 测试 404 端点（httpbin 的 /status/404 返回 404 但请求成功）
    let result = requester.fetch("https://httpbin.org/status/404").await;
    assert!(result.is_ok(), "404 endpoint should return response");
    
    // 测试 500 端点
    let result = requester.fetch("https://httpbin.org/status/500").await;
    assert!(result.is_ok(), "500 endpoint should return response");
    
    // 测试延迟端点
    let result = requester.fetch("https://httpbin.org/delay/1").await;
    assert!(result.is_ok(), "Delayed endpoint should succeed");
    
    println!("Error handling tests passed");
}

/// 测试请求选项
#[tokio::test]
async fn test_request_options() {
    let requester = match AsyncRequester::new("dylibs/libcronet-linux-amd64.so") {
        Ok(r) => r,
        Err(_) => {
            println!("Skipping test: DLL not found");
            return;
        }
    };
    
    // 测试自定义头
    let options = RequestOptions::new()
        .header("X-Custom-Header", "test-value")
        .header("User-Agent", "cronet-rs-test/1.0");
    
    let result = requester.fetch_with_options("https://httpbin.org/headers", options).await;
    assert!(result.is_ok(), "Request with custom headers should succeed");
    
    let (_, data) = result.unwrap();
    let response_str = String::from_utf8_lossy(&data);
    
    // 验证自定义头在响应中
    assert!(response_str.contains("X-Custom-Header"), "Response should contain custom header");
    assert!(response_str.contains("cronet-rs-test"), "Response should contain custom User-Agent");
    
    println!("Request options test passed");
}

/// 测试多次请求（验证无段错误）
#[tokio::test]
async fn test_multiple_requests() {
    let requester = match AsyncRequester::new("dylibs/libcronet-linux-amd64.so") {
        Ok(r) => r,
        Err(_) => {
            println!("Skipping test: DLL not found");
            return;
        }
    };
    
    let urls = vec![
        "https://httpbin.org/get",
        "https://httpbin.org/ip",
        "https://httpbin.org/user-agent",
        "https://httpbin.org/headers",
    ];
    
    let mut success_count = 0;
    
    for url in urls {
        match requester.fetch(url).await {
            Ok((protocol, data)) => {
                success_count += 1;
                println!("Request to {} succeeded: protocol={}, size={}", url, protocol, data.len());
            }
            Err(e) => {
                println!("Request to {} failed: {}", url, e);
            }
        }
    }
    
    // 至少应该有一些请求成功
    assert!(success_count > 0, "At least some requests should succeed");
    println!("Multiple requests test: {}/{} succeeded", success_count, urls.len());
}

/// 测试并发请求
#[tokio::test]
async fn test_concurrent_requests() {
    let requester = match AsyncRequester::new("dylibs/libcronet-linux-amd64.so") {
        Ok(r) => r,
        Err(_) => {
            println!("Skipping test: DLL not found");
            return;
        }
    };
    
    let urls = vec![
        "https://httpbin.org/get",
        "https://httpbin.org/ip",
        "https://httpbin.org/user-agent",
    ];
    
    let mut tasks = Vec::new();
    
    for url in urls {
        let requester_clone = &requester;
        let url_str = url.to_string();
        
        let task = tokio::spawn(async move {
            requester_clone.fetch(&url_str).await
        });
        
        tasks.push(task);
    }
    
    let mut success_count = 0;
    
    for task in tasks {
        match task.await {
            Ok(Ok((protocol, data))) => {
                success_count += 1;
                println!("Concurrent request succeeded: protocol={}, size={}", protocol, data.len());
            }
            Ok(Err(e)) => {
                println!("Concurrent request failed: {}", e);
            }
            Err(e) => {
                println!("Task failed: {}", e);
            }
        }
    }
    
    assert!(success_count > 0, "At least some concurrent requests should succeed");
    println!("Concurrent requests test: {}/{} succeeded", success_count, urls.len());
}