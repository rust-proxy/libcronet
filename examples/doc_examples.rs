//! 文档示例
//! 这些示例可以在文档中使用

/// 基本用法示例
/// 
/// ```
/// use cronet_rs::async_wrapper::AsyncRequester;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
///     
///     match requester.fetch("https://httpbin.org/get").await {
///         Ok((protocol, data)) => {
///             println!("Success! Protocol: {}, Size: {} bytes", protocol, data.len());
///         }
///         Err(e) => {
///             println!("Error: {}", e);
///         }
///     }
///     
///     Ok(())
/// }
/// ```
pub fn basic_example() {}

/// 使用 RequestOptions 的示例
/// 
/// ```
/// use cronet_rs::async_wrapper::{AsyncRequester, RequestOptions};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
///     
///     let options = RequestOptions::new()
///         .method("POST")
///         .header("Content-Type", "application/json")
///         .header("Authorization", "Bearer token123")
///         .body(r#"{"key": "value"}"#.as_bytes().to_vec());
///     
///     match requester.fetch_with_options("https://httpbin.org/post", options).await {
///         Ok((protocol, data)) => {
///             println!("POST successful! Protocol: {}, Size: {} bytes", protocol, data.len());
///         }
///         Err(e) => {
///             println!("Error: {}", e);
///         }
///     }
///     
///     Ok(())
/// }
/// ```
pub fn request_options_example() {}

/// 错误处理示例
/// 
/// ```
/// use cronet_rs::async_wrapper::AsyncRequester;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
///     
///     // 尝试多个 URL，处理错误
///     let urls = vec![
///         "https://httpbin.org/get",
///         "https://invalid-url.example.com",
///         "https://httpbin.org/status/500",
///     ];
///     
///     for url in urls {
///         match requester.fetch(url).await {
///             Ok((protocol, data)) => {
///                 println!("{}: Success (protocol: {}, size: {} bytes)", url, protocol, data.len());
///             }
///             Err(e) => {
///                 println!("{}: Error - {}", url, e);
///                 // 根据错误类型采取不同措施
///             }
///         }
///     }
///     
///     Ok(())
/// }
/// ```
pub fn error_handling_example() {}

/// 批量请求示例
/// 
/// ```
/// use cronet_rs::async_wrapper::AsyncRequester;
/// use futures::future::join_all;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
///     
///     let urls = vec![
///         "https://httpbin.org/get",
///         "https://httpbin.org/ip",
///         "https://httpbin.org/user-agent",
///     ];
///     
///     let mut tasks = Vec::new();
///     
///     for url in urls {
///         let requester_clone = &requester;
///         let url_str = url.to_string();
///         
///         let task = tokio::spawn(async move {
///             requester_clone.fetch(&url_str).await
///         });
///         
///         tasks.push(task);
///     }
///     
///     let results = join_all(tasks).await;
///     
///     for (i, result) in results.iter().enumerate() {
///         match result {
///             Ok(Ok((protocol, data))) => {
///                 println!("Request {}: Success (protocol: {}, size: {} bytes)", i, protocol, data.len());
///             }
///             Ok(Err(e)) => {
///                 println!("Request {}: Error - {}", i, e);
///             }
///             Err(e) => {
///                 println!("Request {}: Task error - {}", i, e);
///             }
///         }
///     }
///     
///     Ok(())
/// }
/// ```
pub fn batch_requests_example() {}