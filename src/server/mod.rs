/// HTTP 服务模块 - 对应 Node.js 版本的 server.js
///
/// 使用 Axum 框架，将 ApiClient 的所有方法自动映射为 REST API 路由
/// 前端可以像调用 Node.js 版一样通过 HTTP 请求调用

pub mod middleware;
pub mod upload;

use crate::api::Query;
use crate::request::{ApiClient, ApiResponse};
use axum::extract::State;
use axum::http::header;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// ============================================================
//  服务器配置
// ============================================================

/// 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// 监听地址，默认 "0.0.0.0"
    pub host: String,
    /// 监听端口，默认 3000
    pub port: u16,
    /// CORS 允许的 Origin，None 表示允许所有
    pub cors_origin: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            cors_origin: None,
        }
    }
}

impl ServerConfig {
    /// 从环境变量读取配置
    /// - `NCM_HOST`: 监听地址
    /// - `NCM_PORT`: 监听端口
    /// - `CORS_ALLOW_ORIGIN`: CORS 允许的 Origin
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("NCM_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("NCM_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            cors_origin: std::env::var("CORS_ALLOW_ORIGIN").ok(),
        }
    }
}

// ============================================================
//  共享状态
// ============================================================

/// Axum 共享状态
#[derive(Clone)]
pub struct AppState {
    pub client: Arc<ApiClient>,
}

// ============================================================
//  查询参数提取
// ============================================================

/// 从 HTTP 请求中提取并合并查询参数
///
/// 合并优先级（后者覆盖前者）：Cookie < URL Query < POST Body
async fn extract_merged_query(
    headers: &HeaderMap,
    uri_query: Option<&str>,
    body: axum::body::Bytes,
    content_type: Option<&str>,
) -> Query {
    let mut query = Query::new();

    // 1. 提取 URL query 参数
    if let Some(qs) = uri_query {
        if let Ok(params) = serde_urlencoded::from_str::<HashMap<String, String>>(qs) {
            for (k, v) in params {
                query.params.insert(k, v);
            }
        }
    }

    // 2. 提取 POST body 参数
    if !body.is_empty() {
        let ct = content_type.unwrap_or("");
        if ct.contains("application/json") {
            if let Ok(map) = serde_json::from_slice::<HashMap<String, Value>>(&body) {
                for (k, v) in map {
                    let s = match &v {
                        Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    };
                    query.params.insert(k, s);
                }
            }
        } else {
            // form-urlencoded 或其他格式
            if let Ok(params) =
                serde_urlencoded::from_bytes::<HashMap<String, String>>(&body)
            {
                for (k, v) in params {
                    query.params.insert(k, v);
                }
            }
        }
    }

    // 3. 处理 cookie
    // 优先使用参数中传入的 cookie
    if let Some(cookie_param) = query.params.remove("cookie") {
        query.cookie = Some(cookie_param);
    } else if let Some(cookie_header) = headers.get(header::COOKIE) {
        if let Ok(c) = cookie_header.to_str() {
            query.cookie = Some(c.to_string());
        }
    }

    // 4. 处理 realIP
    if let Some(real_ip) = query.params.remove("realIP") {
        query.real_ip = Some(real_ip);
    }

    // 5. 处理 proxy
    if let Some(proxy) = query.params.remove("proxy") {
        query.proxy = Some(proxy);
    }

    query
}

// ============================================================
//  响应构建
// ============================================================

/// 构建成功响应
fn build_success_response(api_resp: ApiResponse) -> Response {
    let status = axum::http::StatusCode::from_u16(api_resp.status as u16)
        .unwrap_or(axum::http::StatusCode::OK);
    let mut response = (status, Json(api_resp.body)).into_response();

    // 设置 API 返回的 Cookie
    for cookie_str in &api_resp.cookie {
        if let Ok(val) = header::HeaderValue::from_str(cookie_str) {
            response.headers_mut().append(header::SET_COOKIE, val);
        }
    }

    response
}

/// 构建错误响应
fn build_error_response(err: crate::error::NcmError) -> Response {
    let (status, body) = match &err {
        crate::error::NcmError::Api { code, msg } => {
            let http_status = if *code == 301 {
                axum::http::StatusCode::UNAUTHORIZED
            } else {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            };
            let mut body = json!({ "code": code, "msg": msg });
            if *code == 301 {
                body["msg"] = json!("需要登录");
            }
            (http_status, body)
        }
        _ => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "code": 500, "msg": err.to_string() }),
        ),
    };

    (status, Json(body)).into_response()
}

// ============================================================
//  通用请求处理
// ============================================================

/// 通用 API 请求处理函数
async fn handle_api_request<F>(state: &AppState, headers: HeaderMap, uri: &axum::http::Uri, body: axum::body::Bytes, api_fn: F) -> Response
where
    F: for<'a> FnOnce(
        &'a ApiClient,
        &'a Query,
    ) -> Pin<Box<dyn Future<Output = crate::error::Result<ApiResponse>> + Send + 'a>>,
{
    let path = uri.path().to_string();
    let start = std::time::Instant::now();

    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    let query = extract_merged_query(&headers, uri.query(), body, content_type).await;

    match api_fn(&state.client, &query).await {
        Ok(resp) => {
            tracing::info!("{} -> {} ({:.1?})", path, resp.status, start.elapsed());
            build_success_response(resp)
        }
        Err(e) => {
            tracing::warn!("{} -> ERROR: {} ({:.1?})", path, e, start.elapsed());
            build_error_response(e)
        }
    }
}

// ============================================================
//  路由注册宏
// ============================================================

/// 批量注册 API 路由的宏
///
/// 将每个 ApiClient 方法映射为 GET + POST 路由
macro_rules! api_routes {
    ($router:expr, $( $method:ident => $route:expr ),* $(,)?) => {{
        let router = $router;
        $(
            let router = router.route(
                $route,
                get(|State(state): State<AppState>, headers: HeaderMap, uri: axum::http::Uri| async move {
                    handle_api_request(&state, headers, &uri, axum::body::Bytes::new(), |client, q| Box::pin(client.$method(q))).await
                })
                .post(|State(state): State<AppState>, headers: HeaderMap, uri: axum::http::Uri, body: axum::body::Bytes| async move {
                    handle_api_request(&state, headers, &uri, body, |client, q| Box::pin(client.$method(q))).await
                }),
            );
        )*
        router
    }};
}

// ============================================================
//  路由注册
// ============================================================

/// 注册所有 API 路由
///
/// 标准路由由 build.rs 自动从 src/api/mod.rs 扫描生成
/// 上传路由（avatar_upload, voice_upload）需要 multipart 处理，单独注册
fn register_routes(router: Router<AppState>) -> Router<AppState> {
    // 自动生成的标准路由（build.rs → api_routes_generated.rs）
    let router = {
        include!(concat!(env!("OUT_DIR"), "/api_routes_generated.rs"))
    };

    // 文件上传路由（特殊签名，需要 multipart 处理）
    let router = router
        .route("/avatar/upload", post(upload::handle_avatar_upload))
        .route("/voice/upload", post(upload::handle_voice_upload));

    router
}

// ============================================================
//  构建与启动
// ============================================================

/// 构建 Axum 应用（不启动监听）
///
/// 可用于测试或自定义部署
pub fn build_app(client: ApiClient) -> Router {
    let state = AppState {
        client: Arc::new(client),
    };

    let router = Router::new();
    let router = register_routes(router);

    // 根路由
    let router = router.route(
        "/",
        get(|| async {
            Json(json!({
                "code": 200,
                "msg": "NCM API Rust Server is running",
            }))
        }),
    );

    router
        .layer(middleware::cors_layer(None))
        .with_state(state)
}

/// 构建带自定义 CORS 的 Axum 应用
pub fn build_app_with_config(client: ApiClient, config: &ServerConfig) -> Router {
    let state = AppState {
        client: Arc::new(client),
    };

    let router = Router::new();
    let router = register_routes(router);

    let router = router.route(
        "/",
        get(|| async {
            Json(json!({
                "code": 200,
                "msg": "NCM API Rust Server is running",
            }))
        }),
    );

    router
        .layer(middleware::cors_layer(config.cors_origin.as_deref()))
        .with_state(state)
}

/// 启动 HTTP 服务器
pub async fn start_server(config: ServerConfig) {
    let client = ApiClient::new(None);
    let app = build_app_with_config(client, &config);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("NCM API Server listening on http://{}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
