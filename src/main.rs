use axum::{
    body::Bytes,
    extract::RawQuery,
    http::{HeaderMap, StatusCode},
    routing::any,
    Router,
};
use clap::Parser;
use std::{fs, net::SocketAddr, path::PathBuf};
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::fmt::writer::BoxMakeWriter;

/// 命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// 监听端口 (默认 8080)
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// 日志目录 (默认 ./log)
    #[arg(short, long, default_value = "log")]
    log_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // 日志目录
    if !args.log_dir.exists() {
        fs::create_dir_all(&args.log_dir).expect("无法创建日志目录");
    }
    let log_file = args.log_dir.join("server.log");
    let file = fs::File::create(log_file).expect("无法创建日志文件");
    let writer = BoxMakeWriter::new(file);

    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_max_level(Level::INFO)
        .init();

    // router
    let app = Router::new().route("/*path", any(handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    info!("服务器启动: http://{}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app,
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn handler(
    headers: HeaderMap,
    RawQuery(query): RawQuery,
    body: Bytes,
) -> (StatusCode, &'static str) {
    info!("==== 新请求 ====");
    info!("Headers: {:?}", headers);
    info!("Query: {:?}", query);
    if !body.is_empty() {
        match String::from_utf8(body.to_vec()) {
            Ok(s) => info!("Body: {}", s),
            Err(_) => info!("Body(非UTF8): {:?}", body),
        }
    }
    info!("================\n");

    (StatusCode::OK, "")
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    println!("收到 Ctrl+C, 正在关闭服务器...");
}
