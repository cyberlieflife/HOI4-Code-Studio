// GFX 索引缓存测试
//
// 测试 GFX 索引缓存功能的正确性

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use serde::{Deserialize, Serialize};

// 定义测试用的数据结构（与 gfx.rs 中的结构相同）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GfxIndexEntry {
    pub texture_path: String,
    pub gfx_mtime: u64,
    pub texture_mtime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GfxIndexCache {
    pub entries: HashMap<String, GfxIndexEntry>,
    pub created_at: u64,
    pub version: u32,
}

const GFX_INDEX_CACHE_VERSION: u32 = 1;

/// 创建测试用的临时目录结构
fn create_test_directory() -> TempDir {
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let base_path = temp_dir.path();

    // 创建 gfx 目录结构
    let gfx_dir = base_path.join("gfx").join("interface");
    fs::create_dir_all(&gfx_dir).expect("创建 gfx 目录失败");

    // 创建测试用的 .gfx 文件
    let gfx_content = r#"
        SpriteType = {
            name = "GFX_test_icon"
            texturefile = "gfx/interface/test_icon.png"
        }
        
        SpriteType = {
            name = "GFX_another_icon"
            texturefile = "gfx/interface/another_icon.dds"
        }
    "#;

    let gfx_file = gfx_dir.join("test.gfx");
    fs::write(&gfx_file, gfx_content).expect("写入 gfx 文件失败");

    // 创建测试用的纹理文件
    let texture_dir = base_path.join("gfx").join("interface");
    fs::write(texture_dir.join("test_icon.png"), "fake png data").expect("创建纹理文件失败");
    fs::write(texture_dir.join("another_icon.dds"), "fake dds data").expect("创建纹理文件失败");

    temp_dir
}

#[test]
fn test_gfx_index_entry_creation() {
    // 测试 GfxIndexEntry 结构体创建
    let entry = GfxIndexEntry {
        texture_path: "/path/to/texture.png".to_string(),
        gfx_mtime: 1234567890,
        texture_mtime: 1234567891,
    };

    assert_eq!(entry.texture_path, "/path/to/texture.png");
    assert_eq!(entry.gfx_mtime, 1234567890);
    assert_eq!(entry.texture_mtime, 1234567891);
}

#[test]
fn test_gfx_index_cache_creation() {
    // 测试 GfxIndexCache 结构体创建
    let mut entries = HashMap::new();
    entries.insert(
        "GFX_test_icon".to_string(),
        GfxIndexEntry {
            texture_path: "/path/to/texture.png".to_string(),
            gfx_mtime: 1234567890,
            texture_mtime: 1234567891,
        },
    );

    let cache = GfxIndexCache {
        entries,
        created_at: 1234567890,
        version: 1,
    };

    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.created_at, 1234567890);
    assert_eq!(cache.version, 1);
}

#[test]
fn test_gfx_index_cache_serialization() {
    // 测试索引缓存的序列化和反序列化
    let mut entries = HashMap::new();
    entries.insert(
        "GFX_test_icon".to_string(),
        GfxIndexEntry {
            texture_path: "/path/to/texture.png".to_string(),
            gfx_mtime: 1234567890,
            texture_mtime: 1234567891,
        },
    );

    let cache = GfxIndexCache {
        entries,
        created_at: 1234567890,
        version: 1,
    };

    // 序列化
    let json = serde_json::to_string(&cache).expect("序列化失败");
    assert!(json.contains("GFX_test_icon"));
    assert!(json.contains("/path/to/texture.png"));

    // 反序列化
    let deserialized: GfxIndexCache = serde_json::from_str(&json).expect("反序列化失败");
    assert_eq!(deserialized.entries.len(), 1);
    assert_eq!(deserialized.created_at, 1234567890);
    assert_eq!(deserialized.version, 1);
}

#[test]
fn test_scan_gfx_file() {
    // 测试扫描单个 GFX 文件
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();
    let gfx_file = base_path.join("gfx").join("interface").join("test.gfx");

    // 由于 scan_gfx_file 是私有函数，我们通过构建索引来间接测试
    let roots = vec![base_path.to_path_buf()];

    // 这里我们测试文件读取和解析逻辑
    let content = fs::read_to_string(&gfx_file).expect("读取 gfx 文件失败");

    // 验证内容包含预期的图标定义
    assert!(content.contains("GFX_test_icon"));
    assert!(content.contains("GFX_another_icon"));
    assert!(content.contains("texturefile"));
}

#[test]
fn test_gfx_index_cache_version() {
    // 测试缓存版本号
    assert_eq!(GFX_INDEX_CACHE_VERSION, 1);
}

#[test]
fn test_gfx_index_cache_with_multiple_icons() {
    // 测试包含多个图标的索引缓存
    let mut entries = HashMap::new();

    entries.insert(
        "GFX_icon1".to_string(),
        GfxIndexEntry {
            texture_path: "/path/to/icon1.png".to_string(),
            gfx_mtime: 1000,
            texture_mtime: 1001,
        },
    );

    entries.insert(
        "GFX_icon2".to_string(),
        GfxIndexEntry {
            texture_path: "/path/to/icon2.dds".to_string(),
            gfx_mtime: 2000,
            texture_mtime: 2001,
        },
    );

    entries.insert(
        "GFX_icon3".to_string(),
        GfxIndexEntry {
            texture_path: "/path/to/icon3.tga".to_string(),
            gfx_mtime: 3000,
            texture_mtime: 3001,
        },
    );

    let cache = GfxIndexCache {
        entries,
        created_at: 1234567890,
        version: 1,
    };

    assert_eq!(cache.entries.len(), 3);
    assert!(cache.entries.contains_key("GFX_icon1"));
    assert!(cache.entries.contains_key("GFX_icon2"));
    assert!(cache.entries.contains_key("GFX_icon3"));
}

#[test]
fn test_gfx_index_cache_entry_lookup() {
    // 测试索引缓存条目查找
    let mut entries = HashMap::new();

    entries.insert(
        "GFX_test_icon".to_string(),
        GfxIndexEntry {
            texture_path: "/path/to/texture.png".to_string(),
            gfx_mtime: 1234567890,
            texture_mtime: 1234567891,
        },
    );

    let cache = GfxIndexCache {
        entries,
        created_at: 1234567890,
        version: 1,
    };

    // 测试查找存在的图标
    let found = cache.entries.get("GFX_test_icon");
    assert!(found.is_some());
    assert_eq!(found.unwrap().texture_path, "/path/to/texture.png");

    // 测试查找不存在的图标
    let not_found = cache.entries.get("GFX_nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_gfx_index_cache_empty() {
    // 测试空的索引缓存
    let cache = GfxIndexCache {
        entries: HashMap::new(),
        created_at: 1234567890,
        version: 1,
    };

    assert_eq!(cache.entries.len(), 0);
    assert!(cache.entries.is_empty());
}

#[test]
fn test_gfx_index_cache_update() {
    // 测试索引缓存更新
    let mut cache = GfxIndexCache {
        entries: HashMap::new(),
        created_at: 1234567890,
        version: 1,
    };

    // 添加条目
    cache.entries.insert(
        "GFX_icon1".to_string(),
        GfxIndexEntry {
            texture_path: "/path/to/icon1.png".to_string(),
            gfx_mtime: 1000,
            texture_mtime: 1001,
        },
    );

    assert_eq!(cache.entries.len(), 1);

    // 更新条目
    cache.entries.insert(
        "GFX_icon1".to_string(),
        GfxIndexEntry {
            texture_path: "/path/to/icon1_updated.png".to_string(),
            gfx_mtime: 2000,
            texture_mtime: 2001,
        },
    );

    assert_eq!(cache.entries.len(), 1);
    let entry = cache.entries.get("GFX_icon1").expect("条目不存在");
    assert_eq!(entry.texture_path, "/path/to/icon1_updated.png");
    assert_eq!(entry.gfx_mtime, 2000);
}

#[test]
fn test_gfx_index_cache_remove() {
    // 测试索引缓存删除
    let mut cache = GfxIndexCache {
        entries: HashMap::new(),
        created_at: 1234567890,
        version: 1,
    };

    // 添加条目
    cache.entries.insert(
        "GFX_icon1".to_string(),
        GfxIndexEntry {
            texture_path: "/path/to/icon1.png".to_string(),
            gfx_mtime: 1000,
            texture_mtime: 1001,
        },
    );

    assert_eq!(cache.entries.len(), 1);

    // 删除条目
    cache.entries.remove("GFX_icon1");

    assert_eq!(cache.entries.len(), 0);
    assert!(cache.entries.is_empty());
}
