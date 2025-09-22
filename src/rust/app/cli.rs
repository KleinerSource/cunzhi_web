use crate::config::load_standalone_telegram_config;
use crate::telegram::handle_telegram_only_mcp_request;
use crate::log_important;
use crate::app::builder::run_tauri_app;
use anyhow::Result;

/// 处理命令行参数
pub fn handle_cli_args() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // 检查环境变量控制启动模式
    if let Ok(mode) = std::env::var("CUNZHI_MODE") {
        match mode.to_lowercase().as_str() {
            "web" => {
                // Web模式：从环境变量获取端口，默认3000
                let port = std::env::var("CUNZHI_WEB_PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse::<u16>()
                    .unwrap_or(3000);
                return handle_web_mode(port);
            }
            "desktop" => {
                // 桌面模式：继续处理命令行参数
            }
            _ => {
                eprintln!("无效的 CUNZHI_MODE 值: {}，支持的值: desktop, web", mode);
                std::process::exit(1);
            }
        }
    }

    match args.len() {
        // 无参数：正常启动GUI
        1 => {
            run_tauri_app();
        }
        // 单参数：帮助或版本
        2 => {
            match args[1].as_str() {
                "--help" | "-h" => print_help(),
                "--version" | "-v" => print_version(),
                _ => {
                    eprintln!("未知参数: {}", args[1]);
                    print_help();
                    std::process::exit(1);
                }
            }
        }
        // 多参数：MCP请求模式
        _ => {
            if args[1] == "--mcp-request" && args.len() >= 3 {
                handle_mcp_request(&args[2])?;
            } else {
                eprintln!("无效的命令行参数");
                print_help();
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

/// 处理MCP请求
fn handle_mcp_request(request_file: &str) -> Result<()> {
    // 检查Telegram配置，决定是否启用纯Telegram模式
    match load_standalone_telegram_config() {
        Ok(telegram_config) => {
            if telegram_config.enabled && telegram_config.hide_frontend_popup {
                // 纯Telegram模式：不启动GUI，直接处理
                if let Err(e) = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(handle_telegram_only_mcp_request(request_file))
                {
                    log_important!(error, "处理Telegram请求失败: {}", e);
                    std::process::exit(1);
                }
            } else {
                // 正常模式：启动GUI处理弹窗
                run_tauri_app();
            }
        }
        Err(e) => {
            log_important!(warn, "加载Telegram配置失败: {}，使用默认GUI模式", e);
            // 配置加载失败时，使用默认行为（启动GUI）
            run_tauri_app();
        }
    }
    Ok(())
}

/// 处理Web模式
fn handle_web_mode(port: u16) -> Result<()> {
    log_important!(info, "启动Web服务器模式，端口: {}", port);

    // 创建异步运行时并启动Web服务器
    if let Err(e) = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::web::run_web_server(port))
    {
        log_important!(error, "Web服务器启动失败: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

/// 显示帮助信息
fn print_help() {
    println!("寸止 - 智能代码审查工具");
    println!();
    println!("用法:");
    println!("  等一下                       启动桌面界面");
    println!("  等一下 --mcp-request <文件>  处理 MCP 请求");
    println!("  等一下 --help               显示此帮助信息");
    println!("  等一下 --version            显示版本信息");
    println!();
    println!("环境变量:");
    println!("  CUNZHI_MODE=desktop          启动桌面界面 (默认)");
    println!("  CUNZHI_MODE=web              启动Web界面");
    println!("  CUNZHI_WEB_PORT=8080         指定Web端口 (默认3000)");
    println!();
    println!("示例:");
    println!("  等一下                       # 桌面模式");
    println!("  CUNZHI_MODE=web 等一下       # Web模式，端口3000");
    println!("  CUNZHI_MODE=web CUNZHI_WEB_PORT=8080 等一下  # Web模式，端口8080");
}

/// 显示版本信息
fn print_version() {
    println!("寸止 v{}", env!("CARGO_PKG_VERSION"));
}
