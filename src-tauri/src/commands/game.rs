// 游戏集成相关命令
//
// 包含游戏启动、验证等命令函数

use crate::models::*;

/// 验证游戏目录
#[tauri::command]
pub fn validate_game_directory(path: String) -> serde_json::Value {
    use std::path::Path;

    let game_path = Path::new(&path);

    if !game_path.exists() || !game_path.is_dir() {
        return serde_json::json!({
            "valid": false,
            "message": "目录不存在"
        });
    }

    let required_dirs = vec!["common", "history", "events", "interface"];

    let mut found_count = 0;

    for dir in required_dirs {
        if game_path.join(dir).exists() {
            found_count += 1;
        }
    }

    if found_count >= 2 {
        serde_json::json!({
            "valid": true,
            "message": "有效的 HOI4 游戏目录"
        })
    } else {
        serde_json::json!({
            "valid": false,
            "message": "不是有效的 HOI4 游戏目录"
        })
    }
}

/// 启动游戏
#[tauri::command]
pub fn launch_game() -> LaunchGameResult {
    use std::path::Path;
    use std::process::Command;

    // 加载设置
    let settings_result = super::settings::load_settings();
    if !settings_result.success {
        return LaunchGameResult {
            success: false,
            message: "无法加载设置".to_string(),
        };
    }

    let settings = match settings_result.data {
        Some(data) => data,
        None => {
            return LaunchGameResult {
                success: false,
                message: "设置数据为空".to_string(),
            };
        }
    };

    // 获取启动选项
    let use_steam = settings
        .get("useSteamVersion")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let use_pirate = settings
        .get("usePirateVersion")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let launch_with_debug = settings
        .get("launchWithDebug")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Steam 版本启动
    if use_steam {
        let steam_url = if launch_with_debug {
            "steam://rungameid/394360//--debug"
        } else {
            "steam://rungameid/394360"
        };

        #[cfg(target_os = "windows")]
        {
            match Command::new("cmd").args(["/C", "start", steam_url]).spawn() {
                Ok(_) => {
                    return LaunchGameResult {
                        success: true,
                        message: if launch_with_debug {
                            "正在通过 Steam 启动游戏（调试模式）...".to_string()
                        } else {
                            "正在通过 Steam 启动游戏...".to_string()
                        },
                    };
                }
                Err(e) => {
                    return LaunchGameResult {
                        success: false,
                        message: format!("启动 Steam 失败: {}", e),
                    };
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            match Command::new("xdg-open").arg(steam_url).spawn() {
                Ok(_) => {
                    return LaunchGameResult {
                        success: true,
                        message: if launch_with_debug {
                            "正在通过 Steam 启动游戏（调试模式）...".to_string()
                        } else {
                            "正在通过 Steam 启动游戏...".to_string()
                        },
                    };
                }
                Err(e) => {
                    return LaunchGameResult {
                        success: false,
                        message: format!("启动 Steam 失败: {}", e),
                    };
                }
            }
        }
    }

    // 学习版版本启动
    if use_pirate {
        let game_path = settings
            .get("gameDirectory")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if game_path.is_empty() {
            return LaunchGameResult {
                success: false,
                message: "未设置游戏目录，请在设置中配置 HOI4 游戏目录".to_string(),
            };
        }

        // 获取用户选择的启动程序
        let pirate_executable = settings
            .get("pirateExecutable")
            .and_then(|v| v.as_str())
            .unwrap_or("dowser");

        let exe_name = format!("{}.exe", pirate_executable);

        #[cfg(target_os = "windows")]
        {
            let game_exe = Path::new(game_path).join(&exe_name);

            if !game_exe.exists() {
                return LaunchGameResult {
                    success: false,
                    message: format!(
                        "找不到游戏文件: {}，请确认游戏目录包含 {} 文件",
                        game_exe.display(),
                        exe_name
                    ),
                };
            }

            // 构建启动参数
            let mut args = Vec::new();
            if launch_with_debug {
                args.push("--debug");
            }

            let mut cmd = Command::new(&game_exe);
            cmd.current_dir(game_path);
            if !args.is_empty() {
                cmd.args(&args);
            }

            match cmd.spawn() {
                Ok(_) => {
                    return LaunchGameResult {
                        success: true,
                        message: if launch_with_debug {
                            "正在启动游戏（调试模式）...".to_string()
                        } else {
                            "正在启动游戏...".to_string()
                        },
                    };
                }
                Err(e) => {
                    return LaunchGameResult {
                        success: false,
                        message: format!("启动游戏失败: {}", e),
                    };
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            return LaunchGameResult {
                success: false,
                message: "学习版启动仅支持 Windows 平台".to_string(),
            };
        }
    }

    // 如果两个都未启用
    LaunchGameResult {
        success: false,
        message: "未启用任何游戏启动方式，请在设置中配置".to_string(),
    }
}
