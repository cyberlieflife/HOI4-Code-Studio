// GFX 预览相关命令
//
// 包含图形资源预览、加载等命令函数

use crate::json_decoder::JsonResult;
use crate::models::*;

/// 解析 GFX 预览
#[tauri::command]
pub fn parse_gfx_preview(
    file_path: String,
    content_override: Option<String>,
    project_path: Option<String>,
    game_directory: Option<String>,
    dependency_roots: Option<Vec<String>>,
) -> Result<Vec<GfxSpritePreviewItem>, String> {
    use regex::Regex;
    use std::fs;
    use std::path::PathBuf;

    let source = normalize_path(&file_path);
    let content = match content_override {
        Some(c) => c,
        None => fs::read_to_string(&source)
            .map_err(|e| format!("Failed to read file: {} ({})", source, e))?,
    };

    // Build search roots: project -> deps -> game
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(p) = project_path {
        if !p.trim().is_empty() {
            roots.push(PathBuf::from(normalize_path(&p)));
        }
    }
    if let Some(deps) = dependency_roots {
        for d in deps {
            if !d.trim().is_empty() {
                roots.push(PathBuf::from(normalize_path(&d)));
            }
        }
    }
    if let Some(g) = game_directory {
        if !g.trim().is_empty() {
            roots.push(PathBuf::from(normalize_path(&g)));
        }
    }

    // Note: we keep original content to calculate line numbers, but use a stripped view for parsing.
    let stripped = content
        .lines()
        .map(|line| {
            if let Some(pos) = line.find('#') {
                &line[..pos]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let re_sprite = Regex::new(
        r"(?i)(spriteType|frameAnimatedSpriteType|corneredTileSpriteType|textSpriteType)\s*=\s*\{",
    )
    .map_err(|e| format!("Regex error: {}", e))?;

    // 预编译循环中使用的正则表达式
    let re_x = Regex::new(r"(?i)x\s*=\s*(-?\d+)").map_err(|e| format!("Regex error: {}", e))?;
    let re_y = Regex::new(r"(?i)y\s*=\s*(-?\d+)").map_err(|e| format!("Regex error: {}", e))?;

    let mut items: Vec<GfxSpritePreviewItem> = Vec::new();
    let mut current_pos: usize = 0;

    while let Some(mat) = re_sprite.find_at(&stripped, current_pos) {
        let start = mat.start();
        let end_opt = {
            // local bracket match
            let mut count = 1;
            let bytes = stripped.as_bytes();
            let mut i = mat.end();
            while i < bytes.len() {
                if bytes[i] == b'{' {
                    count += 1;
                } else if bytes[i] == b'}' {
                    count -= 1;
                    if count == 0 {
                        break;
                    }
                }
                i += 1;
            }
            if count == 0 {
                Some(i + 1)
            } else {
                None
            }
        };

        let end = match end_opt {
            Some(v) => v,
            None => {
                current_pos = mat.end();
                continue;
            }
        };

        let block = &stripped[start..end];
        let source_line = count_line_number(&content, start);

        // Extract fields (case-insensitive)
        let extract_value = |key: &str| -> Option<String> {
            let re = Regex::new(&format!(
                r#"(?i){}\s*=\s*(\"(?:[^\"\\]|\\.)*\"|[^\s{{}}]+)"#,
                key
            ))
            .ok()?;
            re.captures(block)
                .map(|cap| cap[1].trim().trim_matches('"').to_string())
        };

        let extract_int =
            |key: &str| -> Option<i32> { extract_value(key).and_then(|v| v.parse::<i32>().ok()) };

        let extract_xy = |key: &str| -> Option<serde_json::Value> {
            let re = Regex::new(&format!(r"(?i){}\s*=\s*\{{([^{{}}]*)\}}", key)).ok()?;
            let cap = re.captures(block)?;
            let inner = &cap[1];
            let x = re_x
                .captures(inner)
                .and_then(|c| c[1].parse::<i32>().ok())
                .unwrap_or(0);
            let y = re_y
                .captures(inner)
                .and_then(|c| c[1].parse::<i32>().ok())
                .unwrap_or(0);
            Some(serde_json::json!({"x": x, "y": y}))
        };

        let name = extract_value("name");
        let texturefile = extract_value("texturefile");
        let no_of_frames = extract_int("noOfFrames")
            .or_else(|| extract_int("noofframes"))
            .unwrap_or(1);
        let border_size = extract_xy("borderSize").or_else(|| extract_xy("bordersize"));

        let mut resolved_path: Option<String> = None;
        let mut cached_png_path: Option<String> = None;
        let mut error: Option<String> = None;

        if let Some(tex) = texturefile.as_ref() {
            if roots.is_empty() {
                error = Some("No search roots provided (project/deps/game)".to_string());
            } else if let Some(p) = find_existing_texture_path(tex, &roots) {
                resolved_path = Some(p.to_string_lossy().to_string());
                match write_png_cache_from_texture(&p) {
                    Ok(out) => {
                        cached_png_path = Some(out.to_string_lossy().to_string());
                    }
                    Err(e) => {
                        error = Some(e);
                    }
                }
            } else {
                error = Some(format!("Texture not found: {}", tex));
            }
        }

        items.push(GfxSpritePreviewItem {
            name: name.unwrap_or_else(|| "(unnamed)".to_string()),
            texturefile,
            no_of_frames,
            border_size,
            source_line,
            resolved_path,
            cached_png_path,
            error,
        });

        current_pos = end;
    }

    Ok(items)
}

/// 加载国策图标 Tauri 命令
#[tauri::command]
pub fn load_focus_icon(
    icon_name: String,
    project_root: Option<String>,
    game_root: Option<String>,
) -> ImageReadResult {
    load_focus_icon_impl(icon_name, project_root, game_root)
}

/// 读取图标缓存 Tauri 命令
#[tauri::command]
pub fn read_icon_cache(icon_name: String) -> serde_json::Value {
    read_icon_cache_impl(icon_name)
}

/// 写入图标缓存 Tauri 命令
#[tauri::command]
pub fn write_icon_cache(icon_name: String, base64: String, mime_type: String) -> serde_json::Value {
    write_icon_cache_impl(icon_name, base64, mime_type)
}

/// 清理图标缓存 Tauri 命令
#[tauri::command]
pub fn clear_icon_cache() -> serde_json::Value {
    clear_icon_cache_impl()
}

/// 获取修改器列表
#[tauri::command]
pub fn get_modifier_list(app: tauri::AppHandle) -> JsonResult {
    use std::fs;
    use std::path::PathBuf;
    use tauri::Manager;

    // 1. 尝试寻找物理文件的路径列表
    let mut paths_to_try = vec![
        PathBuf::from("modifier.txt"),
        PathBuf::from("src-tauri/src/modifier.txt"),
        PathBuf::from("src-tauri/modifier.txt"),
        PathBuf::from("../modifier.txt"),
    ];

    // 添加 Tauri 资源目录路径
    if let Ok(res_dir) = app.path().resource_dir() {
        paths_to_try.push(res_dir.join("modifier.txt"));
    }

    let mut file_bytes = None;

    for path in paths_to_try {
        if path.exists() {
            if let Ok(bytes) = fs::read(&path) {
                file_bytes = Some(bytes);
                break;
            }
        }
    }

    // 2. 如果没找到物理文件，则使用编译时内嵌的文件内容（真正的 "在 exe 内部"）
    let bytes = match file_bytes {
        Some(b) => b,
        None => include_bytes!("../modifier.txt").to_vec(),
    };

    // 3. 解码逻辑
    // 1. 尝试UTF-8
    if let Ok(content) = String::from_utf8(bytes.clone()) {
        return JsonResult {
            success: true,
            message: "读取成功 (UTF-8)".to_string(),
            data: Some(serde_json::Value::String(content)),
        };
    }

    // 2. 使用 chardetng 检测编码
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(&bytes, true);
    let detected_encoding = detector.guess(None, true);

    // 3. 尝试使用检测到的编码解码
    let (decoded, _, had_errors) = detected_encoding.decode(&bytes);
    if !had_errors {
        return JsonResult {
            success: true,
            message: format!("读取成功 ({})", detected_encoding.name()),
            data: Some(serde_json::Value::String(decoded.to_string())),
        };
    }

    // 4. 最后尝试 Lossy
    let content = String::from_utf8_lossy(&bytes).to_string();
    JsonResult {
        success: true,
        message: "读取成功 (Lossy)".to_string(),
        data: Some(serde_json::Value::String(content)),
    }
}

// ==================== 辅助函数 ====================

fn normalize_path(p: &str) -> String {
    let mut s = p.replace('\\', "/");
    while s.contains("//") {
        s = s.replace("//", "/");
    }
    s
}

fn normalize_path_for_join(p: &str) -> String {
    let mut s = p.replace('\\', "/");
    while s.contains("//") {
        s = s.replace("//", "/");
    }
    s.trim_start_matches('/').to_string()
}

fn find_existing_texture_path(
    texturefile: &str,
    roots: &[std::path::PathBuf],
) -> Option<std::path::PathBuf> {
    let normalized_rel = normalize_path_for_join(texturefile);

    // 1) absolute path
    let tex_path = std::path::PathBuf::from(texturefile);
    if tex_path.is_absolute() && tex_path.exists() {
        return Some(tex_path);
    }

    // 2) search roots
    for root in roots {
        let p = root.join(&normalized_rel);
        if p.exists() {
            return Some(p);
        }
    }

    // 3) fallback: if .png/.tga not exist, try .dds
    let lower = normalized_rel.to_lowercase();
    if lower.ends_with(".png") || lower.ends_with(".tga") {
        let dds_rel = format!("{}{}", &normalized_rel[..normalized_rel.len() - 4], ".dds");
        for root in roots {
            let p = root.join(&dds_rel);
            if p.exists() {
                return Some(p);
            }
        }
    }

    None
}

fn write_png_cache_from_texture(
    texture_path: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    use image::ImageFormat;
    use std::fs;
    use std::io::Cursor;

    let src = texture_path
        .to_str()
        .ok_or_else(|| "Invalid texture path".to_string())?
        .to_string();

    let meta = fs::metadata(texture_path)
        .map_err(|e| format!("Failed to stat texture: {} ({})", src, e))?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let cache_dir = get_gfx_preview_cache_dir();
    let key = format!("{}@{}", src, mtime);
    let file_name = format!("{}.png", hash_string(&key));
    let out_path = cache_dir.join(file_name);

    if out_path.exists() {
        println!("[gfx-preview] cache hit: {} -> {}", src, out_path.display());
        return Ok(out_path);
    }

    println!("[gfx-preview] cache miss: {}", src);

    let ext = texture_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Convert to PNG bytes
    let png_bytes: Vec<u8> = if ext == "png" {
        println!("[gfx-preview] use png as-is: {}", src);
        fs::read(texture_path).map_err(|e| format!("Failed to read png: {} ({})", src, e))?
    } else if ext == "dds" {
        println!("[gfx-preview] convert dds -> png: {}", src);
        let dds_data =
            fs::read(texture_path).map_err(|e| format!("Failed to read dds: {} ({})", src, e))?;
        let dds = image_dds::ddsfile::Dds::read(&mut Cursor::new(&dds_data))
            .map_err(|e| format!("Failed to parse dds: {} ({})", src, e))?;
        let img = image_dds::image_from_dds(&dds, 0)
            .map_err(|e| format!("Failed to decode dds: {} ({})", src, e))?;
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::Png)
            .map_err(|e| format!("Failed to encode png: {} ({})", src, e))?;
        buffer.into_inner()
    } else {
        println!("[gfx-preview] decode image -> png: {} (ext={})", src, ext);
        let img = image::open(texture_path)
            .map_err(|e| format!("Failed to decode image: {} ({})", src, e))?;
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::Png)
            .map_err(|e| format!("Failed to encode png: {} ({})", src, e))?;
        buffer.into_inner()
    };

    fs::write(&out_path, png_bytes)
        .map_err(|e| format!("Failed to write png cache: {} ({})", out_path.display(), e))?;
    println!("[gfx-preview] wrote png cache: {}", out_path.display());
    Ok(out_path)
}

fn count_line_number(content: &str, byte_pos: usize) -> i32 {
    let mut line: i32 = 1;
    for (i, b) in content.as_bytes().iter().enumerate() {
        if i >= byte_pos {
            break;
        }
        if *b == b'\n' {
            line += 1;
        }
    }
    line
}

fn get_gfx_preview_cache_dir() -> std::path::PathBuf {
    let base = get_cache_dir();
    let dir = base
        .parent()
        .map(|p| p.join("gfx-preview-cache"))
        .unwrap_or_else(|| base.join("gfx-preview-cache"));

    if let Err(e) = std::fs::create_dir_all(&dir) {
        println!("创建 GFX 预览缓存目录失败: {}", e);
    }

    dir
}

fn get_cache_dir() -> std::path::PathBuf {
    // 获取配置目录
    let config_path = get_config_path();
    // 缓存目录位于配置目录的temp子目录
    let cache_dir = config_path
        .parent()
        .map(|p| p.join("temp").join("focus-icon-cache"))
        .unwrap_or_else(|| {
            let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            config_dir
                .join("HOI4_GUI_Editor")
                .join("temp")
                .join("focus-icon-cache")
        });

    // 确保缓存目录存在
    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        println!("创建缓存目录失败: {}", e);
    }

    cache_dir
}

fn get_config_path() -> std::path::PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    config_dir.join("HOI4_GUI_Editor").join("settings.json")
}

fn hash_string(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn sanitize_icon_cache_filename(icon_name: &str) -> String {
    let trimmed = icon_name.trim();
    if trimmed.is_empty() {
        return "unknown".to_string();
    }

    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "unknown".to_string()
    } else {
        out
    }
}

fn get_icon_cache_path(icon_name: &str) -> std::path::PathBuf {
    let cache_dir = get_cache_dir();
    let safe = sanitize_icon_cache_filename(icon_name);
    cache_dir.join(format!("{}.png", safe))
}

fn get_legacy_icon_cache_path(icon_name: &str) -> std::path::PathBuf {
    let cache_dir = get_cache_dir();
    let hash = hash_string(icon_name);
    cache_dir.join(format!("{}.png", hash))
}

fn read_icon_cache_impl(icon_name: String) -> serde_json::Value {
    let cache_path = get_icon_cache_path(&icon_name);

    if cache_path.exists() {
        match std::fs::read(&cache_path) {
            Ok(data) => {
                use base64::{engine::general_purpose, Engine as _};
                let base64_str = general_purpose::STANDARD.encode(&data);
                serde_json::json!({
                    "success": true,
                    "base64": base64_str,
                    "mime_type": "image/png"
                })
            }
            Err(e) => {
                serde_json::json!(
                    {
                        "success": false,
                        "message": format!("读取缓存失败: {}", e)
                    }
                )
            }
        }
    } else {
        let legacy_path = get_legacy_icon_cache_path(&icon_name);
        if legacy_path.exists() {
            match std::fs::read(&legacy_path) {
                Ok(data) => {
                    let _ = std::fs::write(&cache_path, &data);
                    use base64::{engine::general_purpose, Engine as _};
                    let base64_str = general_purpose::STANDARD.encode(&data);
                    serde_json::json!({
                        "success": true,
                        "base64": base64_str,
                        "mime_type": "image/png"
                    })
                }
                Err(e) => serde_json::json!({
                    "success": false,
                    "message": format!("读取缓存失败: {}", e)
                }),
            }
        } else {
            serde_json::json!({
                "success": false,
                "message": "缓存不存在"
            })
        }
    }
}

fn write_icon_cache_impl(
    icon_name: String,
    base64: String,
    mime_type: String,
) -> serde_json::Value {
    // 只处理png格式
    if mime_type != "image/png" {
        return serde_json::json!(
            {
                "success": false,
                "message": "只支持png格式的图标缓存"
            }
        );
    }

    let cache_path = get_icon_cache_path(&icon_name);

    // 解码base64
    use base64::{engine::general_purpose, Engine as _};
    match general_purpose::STANDARD.decode(&base64) {
        Ok(data) => match std::fs::write(&cache_path, data) {
            Ok(_) => {
                serde_json::json!(
                    {
                        "success": true,
                        "message": "缓存写入成功"
                    }
                )
            }
            Err(e) => {
                serde_json::json!(
                    {
                        "success": false,
                        "message": format!("写入缓存失败: {}", e)
                    }
                )
            }
        },
        Err(e) => {
            serde_json::json!(
                {
                    "success": false,
                    "message": format!("base64解码失败: {}", e)
                }
            )
        }
    }
}

fn clear_icon_cache_impl() -> serde_json::Value {
    let cache_dir = get_cache_dir();

    if cache_dir.exists() {
        match std::fs::remove_dir_all(&cache_dir) {
            Ok(_) => {
                // 重新创建缓存目录
                if let Err(e) = std::fs::create_dir_all(&cache_dir) {
                    println!("重新创建缓存目录失败: {}", e);
                }
                serde_json::json!(
                    {
                        "success": true,
                        "message": "缓存清理成功"
                    }
                )
            }
            Err(e) => {
                serde_json::json!(
                    {
                        "success": false,
                        "message": format!("清理缓存失败: {}", e)
                    }
                )
            }
        }
    } else {
        serde_json::json!(
            {
                "success": true,
                "message": "缓存目录不存在，无需清理"
            }
        )
    }
}

fn get_dependency_paths_from_project(project_root: &str) -> Vec<String> {
    use std::fs;
    use std::path::Path;

    let mut result = Vec::new();
    let project_json_path = Path::new(project_root).join("project.json");
    if !project_json_path.exists() {
        return result;
    }

    let content = match fs::read_to_string(&project_json_path) {
        Ok(c) => c,
        Err(e) => {
            println!("读取 project.json 失败: {}", e);
            return result;
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            println!("解析 project.json 失败: {}", e);
            return result;
        }
    };

    if let Some(deps) = json.get("dependencies").and_then(|v| v.as_array()) {
        for dep in deps {
            let enabled = dep.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
            if !enabled {
                continue;
            }
            if let Some(path) = dep.get("path").and_then(|v| v.as_str()) {
                result.push(path.to_string());
            }
        }
    }

    result
}

fn find_texture_for_icon_in_gfx(content: &str, icon_name: &str) -> Option<String> {
    let mut in_block = false;
    let mut block_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if !in_block {
            // 支持 SpriteType 和 spriteType 两种写法
            if trimmed.starts_with("SpriteType") || trimmed.starts_with("spriteType") {
                in_block = true;
                block_lines.clear();
                block_lines.push(line.to_string());
            }
            continue;
        }

        block_lines.push(line.to_string());

        if trimmed.starts_with('}') {
            let mut name_value: Option<String> = None;
            let mut texture_value: Option<String> = None;

            for bline in &block_lines {
                let t = bline.trim();

                if name_value.is_none() && t.starts_with("name") {
                    if let Some(eq_pos) = t.find('=') {
                        let value_str = t[eq_pos + 1..].trim();
                        let cleaned = value_str.trim_matches('"').trim_matches('\'').to_string();
                        name_value = Some(cleaned);
                    }
                } else if texture_value.is_none() && t.starts_with("texturefile") {
                    if let Some(eq_pos) = t.find('=') {
                        let value_str = t[eq_pos + 1..].trim();
                        let cleaned = value_str.trim_matches('"').trim_matches('\'').to_string();
                        texture_value = Some(cleaned);
                    }
                }
            }

            if let Some(name) = name_value {
                if name == icon_name {
                    if let Some(texture) = texture_value {
                        return Some(texture);
                    }
                }
            }

            in_block = false;
            block_lines.clear();
        }
    }

    None
}

fn load_focus_icon_impl(
    icon_name: String,
    project_root: Option<String>,
    game_root: Option<String>,
) -> ImageReadResult {
    use std::fs;
    use std::path::PathBuf;
    use walkdir::WalkDir;

    let icon_name_trimmed = icon_name.trim().to_string();

    if icon_name_trimmed.is_empty() {
        return ImageReadResult {
            success: false,
            message: Some("图标名称为空".to_string()),
            base64: None,
            mime_type: None,
        };
    }

    let mut roots: Vec<PathBuf> = Vec::new();

    if let Some(root) = project_root.as_ref() {
        if !root.is_empty() {
            let root_path = PathBuf::from(&root);
            roots.push(root_path.clone());

            let dep_paths = get_dependency_paths_from_project(root);
            for dep in dep_paths {
                if !dep.is_empty() {
                    roots.push(PathBuf::from(dep));
                }
            }
        }
    }

    if let Some(root) = game_root.as_ref() {
        if !root.is_empty() {
            roots.push(PathBuf::from(root));
        }
    }

    if roots.is_empty() {
        return ImageReadResult {
            success: false,
            message: Some("未提供有效的项目或游戏目录".to_string()),
            base64: None,
            mime_type: None,
        };
    }

    for root in roots.iter() {
        // HOI4 习惯把 gfx 定义放在 root/gfx/**/**.gfx
        // 旧逻辑只扫 root/interface/*.gfx，导致 MIO trait 等图标无法命中。
        let mut scan_roots: Vec<PathBuf> = vec![root.join("gfx")];
        // 兼容某些工程把 gfx 直接放在 interface 下的情况
        scan_roots.push(root.join("interface"));

        for scan_root in scan_roots {
            if !scan_root.exists() || !scan_root.is_dir() {
                continue;
            }

            for entry in WalkDir::new(&scan_root)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if ext != "gfx" {
                    continue;
                }

                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                if let Some(texture_rel) =
                    find_texture_for_icon_in_gfx(&content, &icon_name_trimmed)
                {
                    let normalized_rel = texture_rel.replace('\\', "/");
                    let texture_path = root.join(normalized_rel);
                    let texture_path_str = texture_path.to_string_lossy().to_string();
                    return super::file::read_image_as_base64(texture_path_str);
                }
            }
        }
    }

    ImageReadResult {
        success: false,
        message: Some(format!(
            "未在 gfx/**/**/*.gfx 中找到图标定义: {}",
            icon_name_trimmed
        )),
        base64: None,
        mime_type: None,
    }
}
