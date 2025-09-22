use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::{AppConfig, AppState as TauriAppState, load_standalone_config, save_standalone_config};
use crate::mcp::types::{PopupRequest, build_continue_response, build_send_response};
use crate::mcp::handlers::create_tauri_popup;
use crate::log_important;

/// Web API 应用状态
#[derive(Clone)]
pub struct WebAppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub response_channel: Arc<Mutex<Option<tokio::sync::oneshot::Sender<String>>>>,
}

impl WebAppState {
    pub fn new() -> Self {
        let config = match load_standalone_config() {
            Ok(config) => config,
            Err(e) => {
                log_important!(warn, "加载配置失败，使用默认配置: {}", e);
                AppConfig::default()
            }
        };

        Self {
            config: Arc::new(Mutex::new(config)),
            response_channel: Arc::new(Mutex::new(None)),
        }
    }
}

/// 创建 API 路由
pub fn create_api_routes() -> Router<WebAppState> {
    Router::new()
        // 应用信息
        .route("/api/app/info", get(get_app_info))
        .route("/api/app/version", get(get_current_version))
        
        // 配置管理
        .route("/api/config", get(get_config))
        .route("/api/config", post(update_config))
        
        // 主题设置
        .route("/api/theme", get(get_theme))
        .route("/api/theme", post(set_theme))
        
        // 窗口设置（Web模式下这些API返回默认值）
        .route("/api/window/always-on-top", get(get_always_on_top))
        .route("/api/window/always-on-top", post(set_always_on_top))
        
        // 音频设置
        .route("/api/audio/enabled", get(get_audio_notification_enabled))
        .route("/api/audio/enabled", post(set_audio_notification_enabled))
        .route("/api/audio/url", get(get_audio_url))
        .route("/api/audio/url", post(set_audio_url))
        
        // MCP 相关
        .route("/api/mcp/popup", post(handle_mcp_popup))
        .route("/api/mcp/response", post(send_mcp_response))
        
        // Telegram 配置
        .route("/api/telegram/config", get(get_telegram_config))
        .route("/api/telegram/config", post(set_telegram_config))
}

// API 处理函数

async fn get_app_info() -> Json<Value> {
    Json(json!({
        "name": "寸止",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "web"
    }))
}

async fn get_current_version() -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn get_config(State(state): State<WebAppState>) -> Json<Value> {
    let config = state.config.lock().await;
    Json(serde_json::to_value(&*config).unwrap_or_default())
}

#[derive(Deserialize)]
struct UpdateConfigRequest {
    config: AppConfig,
}

async fn update_config(
    State(state): State<WebAppState>,
    Json(payload): Json<UpdateConfigRequest>,
) -> Result<Json<Value>, StatusCode> {
    {
        let mut config = state.config.lock().await;
        *config = payload.config;
    }
    
    // 保存到文件
    let config = state.config.lock().await.clone();
    if let Err(e) = save_standalone_config(&config) {
        log_important!(error, "保存配置失败: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok(Json(json!({"success": true})))
}

async fn get_theme(State(state): State<WebAppState>) -> Json<Value> {
    let config = state.config.lock().await;
    Json(json!({
        "theme": config.ui_config.theme
    }))
}

#[derive(Deserialize)]
struct SetThemeRequest {
    theme: String,
}

async fn set_theme(
    State(state): State<WebAppState>,
    Json(payload): Json<SetThemeRequest>,
) -> Result<Json<Value>, StatusCode> {
    {
        let mut config = state.config.lock().await;
        config.ui_config.theme = payload.theme;
    }
    
    // 保存配置
    let config = state.config.lock().await.clone();
    if let Err(e) = save_standalone_config(&config) {
        log_important!(error, "保存主题配置失败: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok(Json(json!({"success": true})))
}

async fn get_always_on_top(State(state): State<WebAppState>) -> Json<Value> {
    let config = state.config.lock().await;
    Json(json!({
        "always_on_top": config.ui_config.always_on_top
    }))
}

#[derive(Deserialize)]
struct SetAlwaysOnTopRequest {
    always_on_top: bool,
}

async fn set_always_on_top(
    State(state): State<WebAppState>,
    Json(payload): Json<SetAlwaysOnTopRequest>,
) -> Json<Value> {
    {
        let mut config = state.config.lock().await;
        config.ui_config.always_on_top = payload.always_on_top;
    }
    
    // Web模式下无法实际设置窗口置顶，只保存配置
    Json(json!({"success": true}))
}

async fn get_audio_notification_enabled(State(state): State<WebAppState>) -> Json<Value> {
    let config = state.config.lock().await;
    Json(json!({
        "enabled": config.audio_config.notification_enabled
    }))
}

#[derive(Deserialize)]
struct SetAudioEnabledRequest {
    enabled: bool,
}

async fn set_audio_notification_enabled(
    State(state): State<WebAppState>,
    Json(payload): Json<SetAudioEnabledRequest>,
) -> Result<Json<Value>, StatusCode> {
    {
        let mut config = state.config.lock().await;
        config.audio_config.notification_enabled = payload.enabled;
    }
    
    // 保存配置
    let config = state.config.lock().await.clone();
    if let Err(e) = save_standalone_config(&config) {
        log_important!(error, "保存音频配置失败: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok(Json(json!({"success": true})))
}

async fn get_audio_url(State(state): State<WebAppState>) -> Json<Value> {
    let config = state.config.lock().await;
    Json(json!({
        "url": config.audio_config.custom_url
    }))
}

#[derive(Deserialize)]
struct SetAudioUrlRequest {
    url: String,
}

async fn set_audio_url(
    State(state): State<WebAppState>,
    Json(payload): Json<SetAudioUrlRequest>,
) -> Result<Json<Value>, StatusCode> {
    {
        let mut config = state.config.lock().await;
        config.audio_config.custom_url = payload.url;
    }
    
    // 保存配置
    let config = state.config.lock().await.clone();
    if let Err(e) = save_standalone_config(&config) {
        log_important!(error, "保存音频URL配置失败: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok(Json(json!({"success": true})))
}

async fn handle_mcp_popup(
    State(state): State<WebAppState>,
    Json(request): Json<PopupRequest>,
) -> Result<Json<Value>, StatusCode> {
    // 在Web模式下，我们需要不同的处理方式
    // 这里暂时返回一个简单的响应，实际实现可能需要WebSocket或SSE
    log_important!(info, "收到MCP弹窗请求: {}", request.message);
    
    // 返回一个默认的继续响应
    let response = build_continue_response("继续");
    Ok(Json(serde_json::to_value(response).unwrap_or_default()))
}

async fn send_mcp_response(
    State(state): State<WebAppState>,
    Json(response): Json<Value>,
) -> Json<Value> {
    log_important!(info, "收到MCP响应: {:?}", response);
    
    // 在Web模式下，这里可能需要通过WebSocket发送响应
    // 暂时只记录日志
    Json(json!({"success": true}))
}

async fn get_telegram_config(State(state): State<WebAppState>) -> Json<Value> {
    let config = state.config.lock().await;
    Json(serde_json::to_value(&config.telegram_config).unwrap_or_default())
}

#[derive(Deserialize)]
struct SetTelegramConfigRequest {
    config: crate::config::TelegramConfig,
}

async fn set_telegram_config(
    State(state): State<WebAppState>,
    Json(payload): Json<SetTelegramConfigRequest>,
) -> Result<Json<Value>, StatusCode> {
    {
        let mut config = state.config.lock().await;
        config.telegram_config = payload.config;
    }
    
    // 保存配置
    let config = state.config.lock().await.clone();
    if let Err(e) = save_standalone_config(&config) {
        log_important!(error, "保存Telegram配置失败: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok(Json(json!({"success": true})))
}
