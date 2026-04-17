//! 测试不进行网络请求

use cronet_rs::async_wrapper::AsyncRequester;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试创建 AsyncRequester...");
    
    // 1. 创建请求器
    let requester = AsyncRequester::new("dylibs/libcronet-linux-amd64.so")?;
    println!("✓ 创建成功");
    
    // 2. 获取版本
    println!("引擎版本: {}", requester.engine_version()?);
    
    // 3. 不进行网络请求，直接退出
    println!("✓ 测试完成，无段错误");
    
    Ok(())
}