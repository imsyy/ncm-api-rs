use ncm_api::server::{ServerConfig, start_server};

#[tokio::main]
async fn main() {
    // 初始化日志，默认 info 级别，可通过 RUST_LOG 环境变量覆盖
    // 例如: RUST_LOG=debug cargo run --features server --bin ncm-server
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ncm_api=info".into()),
        )
        .init();

    let config = ServerConfig::from_env();
    start_server(config).await;
}
