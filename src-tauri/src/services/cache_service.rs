// 缓存管理服务
// 提供缓存管理的核心业务逻辑

use base64::Engine;
use image::ImageFormat;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// 缓存管理服务结构体
pub struct CacheService;

impl CacheService {
    /// 创建新的缓存服务实例
    pub fn new() -> Self {
        CacheService
    }

    /// 获取应用数据根目录
    ///
    /// # 返回值
    /// 返回应用数据根目录路径（如 `{config_dir}/HOI4_GUI_Editor/`）
    fn get_app_data_dir(&self) -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("HOI4_GUI_Editor")
    }

    /// 获取配置文件路径
    ///
    /// # 返回值
    /// 返回应用配置文件的路径
    fn get_config_path(&self) -> PathBuf {
        self.get_app_data_dir().join("settings.json")
    }

    /// 获取默认缓存根目录（不含子目录）
    ///
    /// # 返回值
    /// 返回默认缓存根目录路径（如 `{config_dir}/HOI4_GUI_Editor/`）
    pub fn get_default_cache_root(&self) -> PathBuf {
        self.get_app_data_dir()
    }

    /// 获取当前缓存根目录（考虑用户自定义设置）
    ///
    /// # 返回值
    /// 返回当前缓存根目录路径。如果用户设置了自定义缓存目录，则返回自定义目录；否则返回默认目录。
    pub fn get_cache_root(&self) -> PathBuf {
        // 尝试从设置中读取自定义缓存目录
        let config_path = self.get_config_path();
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(cache_dir) = settings.get("cacheDirectory").and_then(|v| v.as_str()) {
                        if !cache_dir.is_empty() {
                            return PathBuf::from(cache_dir);
                        }
                    }
                }
            }
        }
        self.get_default_cache_root()
    }

    /// 获取缓存目录路径
    ///
    /// # 返回值
    /// 返回缓存目录的路径
    pub fn get_cache_dir(&self) -> PathBuf {
        let cache_root = self.get_cache_root();
        let cache_dir = cache_root.join("temp").join("focus-icon-cache");

        // 确保缓存目录存在
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            println!("创建缓存目录失败: {}", e);
        }

        cache_dir
    }

    /// 获取 GFX 预览缓存目录
    ///
    /// # 返回值
    /// 返回 GFX 预览缓存目录的路径
    pub fn get_gfx_preview_cache_dir(&self) -> PathBuf {
        let cache_root = self.get_cache_root();
        let dir = cache_root.join("temp").join("gfx-preview-cache");

        if let Err(e) = fs::create_dir_all(&dir) {
            println!("创建 GFX 预览缓存目录失败: {}", e);
        }

        dir
    }

    /// 获取 DDS 转换缓存目录
    ///
    /// # 返回值
    /// 返回 DDS 转换缓存目录的路径
    pub fn get_dds_conversion_cache_dir(&self) -> PathBuf {
        let cache_root = self.get_cache_root();
        let dir = cache_root.join("dds-conversion-cache");

        if let Err(e) = fs::create_dir_all(&dir) {
            println!("创建 DDS 转换缓存目录失败: {}", e);
        }

        dir
    }

    /// 迁移缓存目录
    ///
    /// 将旧缓存目录下的所有内容移动到新目录
    ///
    /// # 参数
    /// * `new_cache_dir` - 新的缓存根目录路径
    ///
    /// # 返回值
    /// 返回操作结果的 JSON 对象
    pub fn migrate_cache_directory(&self, new_cache_dir: &str) -> serde_json::Value {
        let new_root = PathBuf::from(new_cache_dir);
        let old_root = self.get_cache_root();

        // 如果新旧目录相同，无需迁移
        if old_root == new_root {
            return serde_json::json!({
                "success": true,
                "message": "缓存目录未变更，无需迁移"
            });
        }

        // 需要迁移的子目录和文件
        let items_to_migrate = [
            "temp",              // 包含 focus-icon-cache 和 gfx-preview-cache
            "dds-conversion-cache",
        ];

        let mut migrated_count = 0u32;
        let mut errors = Vec::new();

        for item in &items_to_migrate {
            let old_path = old_root.join(item);
            let new_path = new_root.join(item);

            if !old_path.exists() {
                continue;
            }

            // 确保新目录的父目录存在
            if let Some(parent) = new_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    errors.push(format!("创建目录 {} 失败: {}", parent.display(), e));
                    continue;
                }
            }

            // 移动目录
            if old_path.is_dir() {
                match Self::move_dir_recursive(&old_path, &new_path) {
                    Ok(count) => migrated_count += count,
                    Err(e) => errors.push(format!("迁移 {} 失败: {}", item, e)),
                }
            }
        }

        if errors.is_empty() {
            serde_json::json!({
                "success": true,
                "message": format!("缓存迁移成功，共迁移 {} 个文件/目录", migrated_count)
            })
        } else {
            serde_json::json!({
                "success": false,
                "message": format!("迁移完成但有错误: {}", errors.join("; ")),
                "migratedCount": migrated_count
            })
        }
    }

    /// 递归移动目录内容
    ///
    /// 将源目录下的所有内容移动到目标目录。
    /// 如果目标目录已存在同名项，则合并（覆盖文件，递归合并目录）。
    fn move_dir_recursive(src: &Path, dst: &Path) -> Result<u32, String> {
        let mut count = 0u32;

        // 确保目标目录存在
        fs::create_dir_all(dst)
            .map_err(|e| format!("创建目标目录 {} 失败: {}", dst.display(), e))?;

        let entries = fs::read_dir(src)
            .map_err(|e| format!("读取源目录 {} 失败: {}", src.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let src_path = entry.path();
            let file_name = src_path
                .file_name()
                .ok_or_else(|| "无法获取文件名".to_string())?;
            let dst_path = dst.join(&file_name);

            if src_path.is_dir() {
                // 递归移动子目录
                count += Self::move_dir_recursive(&src_path, &dst_path)?;
            } else {
                // 移动文件
                if dst_path.exists() {
                    // 目标已存在，先删除再移动
                    let _ = fs::remove_file(&dst_path);
                }
                fs::rename(&src_path, &dst_path)
                    .map_err(|e| format!("移动文件 {} -> {} 失败: {}", src_path.display(), dst_path.display(), e))?;
                count += 1;
            }
        }

        // 尝试删除空的源目录
        let _ = fs::remove_dir(src);

        Ok(count)
    }

    /// 计算字符串的哈希值，用于缓存文件名
    ///
    /// # 参数
    /// * `s` - 要计算哈希的字符串
    ///
    /// # 返回值
    /// 返回十六进制格式的哈希字符串
    pub fn hash_string(&self, s: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// 清理图标缓存文件名
    ///
    /// 将文件名中的非法字符替换为下划线
    ///
    /// # 参数
    /// * `icon_name` - 图标名称
    ///
    /// # 返回值
    /// 返回清理后的文件名
    pub fn sanitize_icon_cache_filename(&self, icon_name: &str) -> String {
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

    /// 获取图标缓存路径
    ///
    /// # 参数
    /// * `icon_name` - 图标名称
    ///
    /// # 返回值
    /// 返回图标缓存文件的路径
    pub fn get_icon_cache_path(&self, icon_name: &str) -> PathBuf {
        let cache_dir = self.get_cache_dir();
        let safe = self.sanitize_icon_cache_filename(icon_name);
        cache_dir.join(format!("{}.png", safe))
    }

    /// 获取旧版图标缓存路径
    ///
    /// # 参数
    /// * `icon_name` - 图标名称
    ///
    /// # 返回值
    /// 返回旧版图标缓存文件的路径
    pub fn get_legacy_icon_cache_path(&self, icon_name: &str) -> PathBuf {
        let cache_dir = self.get_cache_dir();
        let hash = self.hash_string(icon_name);
        cache_dir.join(format!("{}.png", hash))
    }

    /// 读取图标缓存
    ///
    /// # 参数
    /// * `icon_name` - 图标名称
    ///
    /// # 返回值
    /// 返回包含缓存数据的 JSON 对象
    pub fn read_icon_cache(&self, icon_name: &str) -> serde_json::Value {
        let cache_path = self.get_icon_cache_path(icon_name);

        if cache_path.exists() {
            match fs::read(&cache_path) {
                Ok(data) => {
                    let base64 = base64::engine::general_purpose::STANDARD.encode(&data);
                    serde_json::json!({
                        "success": true,
                        "base64": base64,
                        "mime_type": "image/png"
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "success": false,
                        "message": format!("读取缓存失败: {}", e)
                    })
                }
            }
        } else {
            let legacy_path = self.get_legacy_icon_cache_path(icon_name);
            if legacy_path.exists() {
                match fs::read(&legacy_path) {
                    Ok(data) => {
                        let _ = fs::write(&cache_path, &data);
                        let base64 = base64::engine::general_purpose::STANDARD.encode(&data);
                        serde_json::json!({
                            "success": true,
                            "base64": base64,
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

    /// 写入图标缓存
    ///
    /// # 参数
    /// * `icon_name` - 图标名称
    /// * `base64` - Base64 编码的图片数据
    /// * `mime_type` - MIME 类型
    ///
    /// # 返回值
    /// 返回操作结果的 JSON 对象
    pub fn write_icon_cache(
        &self,
        icon_name: &str,
        base64: &str,
        mime_type: &str,
    ) -> serde_json::Value {
        // 只处理 png 格式
        if mime_type != "image/png" {
            return serde_json::json!({
                "success": false,
                "message": "只支持png格式的图标缓存"
            });
        }

        let cache_path = self.get_icon_cache_path(icon_name);

        // 解码 base64
        match base64::engine::general_purpose::STANDARD.decode(base64) {
            Ok(data) => match fs::write(&cache_path, data) {
                Ok(_) => {
                    serde_json::json!({
                        "success": true,
                        "message": "缓存写入成功"
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "success": false,
                        "message": format!("写入缓存失败: {}", e)
                    })
                }
            },
            Err(e) => {
                serde_json::json!({
                    "success": false,
                    "message": format!("base64解码失败: {}", e)
                })
            }
        }
    }

    /// 清理图标缓存
    ///
    /// # 返回值
    /// 返回操作结果的 JSON 对象
    pub fn clear_icon_cache(&self) -> serde_json::Value {
        let cache_dir = self.get_cache_dir();

        if cache_dir.exists() {
            match fs::remove_dir_all(&cache_dir) {
                Ok(_) => {
                    // 重新创建缓存目录
                    if let Err(e) = fs::create_dir_all(&cache_dir) {
                        println!("重新创建缓存目录失败: {}", e);
                    }
                    serde_json::json!({
                        "success": true,
                        "message": "缓存清理成功"
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "success": false,
                        "message": format!("清理缓存失败: {}", e)
                    })
                }
            }
        } else {
            serde_json::json!({
                "success": true,
                "message": "缓存目录不存在，无需清理"
            })
        }
    }

    /// 从纹理文件写入 PNG 缓存
    ///
    /// # 参数
    /// * `texture_path` - 纹理文件路径
    ///
    /// # 返回值
    /// 成功返回缓存文件路径，失败返回错误信息
    pub fn write_png_cache_from_texture(&self, texture_path: &Path) -> Result<PathBuf, String> {
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

        let cache_dir = self.get_gfx_preview_cache_dir();
        let key = format!("{}@{}", src, mtime);
        let file_name = format!("{}.png", self.hash_string(&key));
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

        // 转换为 PNG 字节
        let png_bytes: Vec<u8> = if ext == "png" {
            println!("[gfx-preview] use png as-is: {}", src);
            fs::read(texture_path).map_err(|e| format!("Failed to read png: {} ({})", src, e))?
        } else if ext == "dds" {
            println!("[gfx-preview] convert dds -> png: {}", src);
            let dds_data = fs::read(texture_path)
                .map_err(|e| format!("Failed to read dds: {} ({})", src, e))?;
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
}

impl Default for CacheService {
    fn default() -> Self {
        Self::new()
    }
}
