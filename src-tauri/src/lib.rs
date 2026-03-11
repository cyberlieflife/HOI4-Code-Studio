// 禁止使用 unwrap()，避免 panic
#![deny(clippy::unwrap_used)]

// 本地模块
mod bracket_matcher;
mod commands;
mod country_tags;
mod cwtools;
mod dependency;
mod file_tree;
mod focus_localization;
mod gui_engine;
mod idea_registry;
mod json_decoder;
mod map_engine;
mod mio_parser;
mod models;
mod plugin_manager;
mod services;
mod tag_validator;
mod theme_manager;

use json_decoder::{
    get_json_path, merge_json, parse_json, read_json_file, set_json_path, stringify_json,
    validate_json, write_json_file,
};

use bracket_matcher::{
    find_bracket_matches, find_matching_bracket, get_bracket_depth_map, BracketMatchResult,
};

use commands::cwtools::ValidationServiceState;
use idea_registry::{load_ideas, reset_idea_cache};
use tag_validator::validate_tags;

// ==================== Tauri 命令 ====================

// ==================== 括号匹配命令 ====================

/// Tauri命令：查找所有括号匹配
///
/// # 参数
/// * `content` - 文本内容
#[tauri::command]
fn match_brackets(content: String) -> BracketMatchResult {
    find_bracket_matches(&content)
}

/// Tauri命令：查找光标位置的匹配括号
///
/// # 参数
/// * `content` - 文本内容
/// * `cursor_pos` - 光标位置
#[tauri::command]
fn find_bracket_pair(content: String, cursor_pos: usize) -> Option<usize> {
    find_matching_bracket(&content, cursor_pos)
}

/// Tauri命令：获取括号深度映射
///
/// # 参数
/// * `content` - 文本内容
#[tauri::command]
fn get_bracket_depths(content: String) -> Vec<usize> {
    get_bracket_depth_map(&content)
}

// ==================== 应用入口 ====================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::Manager;
            app.manage(map_engine::MapState::default());
            app.manage(ValidationServiceState::new());

            // 允许 asset protocol 访问插件安装目录。
            // 否则 convertFileSrc() 会生成类似 https://asset.localhost/... 的 URL，但 WebView 无法读取
            // config_dir/HOI4_GUI_Editor/plugins 下的文件，从而导致 iframe 显示“asset.localhost 拒绝连接”。
            //
            // 这里使用运行时 scope 扩展（allow_directory）来放行插件目录（递归）。
            // 注意：如果 allow_directory 失败，我们只打印日志而不让应用启动失败。
            if let Some(config_dir) = dirs::config_dir() {
                let plugins_dir = config_dir.join("HOI4_GUI_Editor").join("plugins");
                if let Err(e) = std::fs::create_dir_all(&plugins_dir) {
                    println!("创建插件目录失败: {} ({})", plugins_dir.display(), e);
                }

                println!(
                    "当前 Tauri 版本不支持运行时 asset protocol scope，已跳过目录放行：{}",
                    plugins_dir.display()
                );
            } else {
                println!("无法解析 config_dir，无法为插件目录添加 asset protocol scope");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_new_project,
            commands::initialize_project,
            commands::open_project,
            commands::get_recent_projects,
            commands::get_recent_project_stats,
            commands::get_recent_project_stats_cached,
            commands::open_file_dialog,
            commands::exit_application,
            commands::open_settings,
            commands::open_folder,
            commands::read_directory,
            commands::create_file,
            commands::create_folder,
            commands::rename_path,
            commands::delete_path,
            commands::load_settings,
            commands::save_settings,
            commands::validate_game_directory,
            commands::launch_game,
            parse_json,
            stringify_json,
            validate_json,
            merge_json,
            get_json_path,
            set_json_path,
            read_json_file,
            write_json_file,
            commands::read_file_content,
            commands::write_file_content,
            commands::search_files,
            commands::build_directory_tree,
            commands::build_directory_tree_fast,
            match_brackets,
            find_bracket_pair,
            get_bracket_depths,
            country_tags::load_country_tags,
            validate_tags,
            load_ideas,
            reset_idea_cache,
            dependency::load_dependencies,
            dependency::save_dependencies,
            dependency::validate_dependency_path,
            dependency::index_dependency,
            plugin_manager::install_plugin,
            plugin_manager::uninstall_plugin,
            plugin_manager::list_installed_plugins,
            plugin_manager::validate_plugin_package,
            theme_manager::list_themes,
            theme_manager::upsert_theme,
            theme_manager::delete_theme,
            commands::pack_project,
            commands::read_image_as_base64,
            commands::load_focus_icon,
            commands::read_icon_cache,
            commands::write_icon_cache,
            commands::clear_icon_cache,
            focus_localization::load_focus_localizations,
            commands::get_modifier_list,
            map_engine::load_map_definitions,
            map_engine::load_default_map,
            map_engine::load_provinces_bmp,
            map_engine::get_province_map_binary,
            map_engine::generate_colored_map,
            map_engine::get_definition_color_map,
            map_engine::load_all_states,
            map_engine::load_country_colors,
            map_engine::get_province_owner_color_map,
            map_engine::initialize_map_context,
            map_engine::get_map_tile_direct,
            map_engine::get_province_at_point,
            map_engine::get_map_metadata,
            map_engine::get_map_preview,
            map_engine::get_province_outline,
            map_engine::get_state_outline,
            gui_engine::parse_gui_file,
            gui_engine::parse_gui_content,
            gui_engine::parse_gfx_file,
            gui_engine::resolve_gui_resource,
            mio_parser::parse_mio_preview,
            commands::parse_gfx_preview,
            // cwtools 语法验证命令
            commands::initialize_validation_service,
            commands::validate_script,
            commands::validate_script_incremental,
            commands::parse_file,
            commands::format_script_command,
            commands::reload_rules,
            commands::clear_validation_cache,
            commands::invalidate_file_cache,
            commands::get_cache_stats,
            commands::get_rule_stats,
            commands::validate_batch,
            commands::load_references,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
