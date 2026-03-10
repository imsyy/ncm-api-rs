// Netease Cloud Music API - Rust SDK
//
// 从 Node.js 版本 (api-enhanced) 移植的 Rust 原生实现
// 支持 weapi / eapi / linuxapi 三种加密方式

pub mod crypto;
pub mod util;
pub mod request;
pub mod api;
pub mod error;

pub use error::NcmError;
pub use request::{ApiClient, ApiResponse, RequestOption, CryptoType};

/// 创建一个新的 API 客户端
pub fn create_client(cookie: Option<String>) -> ApiClient {
    ApiClient::new(cookie)
}
