// 发布版本隐藏 Windows 控制台窗口
// cfg_attr 是条件编译属性，只在非 debug 模式下生效
// windows_subsystem = "windows" 表示不显示控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 程序入口函数
// main() 是 Rust 程序的入口点
/// 主函数，作为 Tauri 应用的入口点
///
/// 该函数负责初始化并启动 Tauri 应用程序。它通过调用库中的 run() 函数来完成这一任务。
/// 在 Tauri 应用中，main() 函数是必需的，它是程序执行的起点。
fn main() {
    // 初始化 tracing 日志系统
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // 调用库中的 run() 函数启动 Tauri 应用
    // hoi4_code_studio_lib 是一个自定义库，提供了运行应用所需的核心功能
    hoi4_code_studio_lib::run()
}
