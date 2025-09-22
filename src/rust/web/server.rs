use axum::{
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use std::net::SocketAddr;

use crate::log_important;
use super::{api::{create_api_routes, WebAppState}, static_files::static_handler};

/// 启动 Web 服务器
pub async fn run_web_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = WebAppState::new();
    
    // 创建应用路由
    let app = Router::new()
        // API 路由
        .nest("/", create_api_routes())
        // 静态文件路由 - 放在最后作为fallback
        .fallback(static_handler)
        // 添加状态
        .with_state(app_state)
        // 添加CORS支持
        .layer(
            ServiceBuilder::new()
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any)
                )
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    log_important!(info, "🌐 Web服务器启动成功！");
    log_important!(info, "📍 访问地址: http://localhost:{}", port);
    log_important!(info, "📍 局域网访问: http://0.0.0.0:{}", port);
    log_important!(info, "🛑 按 Ctrl+C 停止服务器");
    
    // 启动服务器
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
