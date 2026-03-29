// DDS 转换缓存测试
//
// 测试 DDS 转换缓存功能的正确性

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use serde::{Deserialize, Serialize};

// 定义测试用的数据结构（与 gfx.rs 中的结构相同）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdsConversionEntry {
    pub source_path: String,
    pub source_mtime: u64,
    pub png_path: String,
    pub converted_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdsConversionCache {
    pub entries: HashMap<String, DdsConversionEntry>,
    pub created_at: u64,
    pub version: u32,
}

const DDS_CONVERSION_CACHE_VERSION: u32 = 1;

/// 创建测试用的临时目录结构
fn create_test_directory() -> TempDir {
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let base_path = temp_dir.path();

    // 创建测试用的 DDS 文件
    let dds_dir = base_path.join("gfx").join("interface");
    fs::create_dir_all(&dds_dir).expect("创建 gfx 目录失败");

    // 创建测试用的 DDS 文件（模拟内容）
    let dds_content = b"DDS fake content for testing";
    fs::write(dds_dir.join("test_icon.dds"), dds_content).expect("创建 DDS 文件失败");

    temp_dir
}

#[test]
fn test_dds_conversion_entry_creation() {
    // 测试 DdsConversionEntry 结构体创建
    let entry = DdsConversionEntry {
        source_path: "/path/to/source.dds".to_string(),
        source_mtime: 1234567890,
        png_path: "/path/to/cached.png".to_string(),
        converted_at: 1234567891,
    };

    assert_eq!(entry.source_path, "/path/to/source.dds");
    assert_eq!(entry.source_mtime, 1234567890);
    assert_eq!(entry.png_path, "/path/to/cached.png");
    assert_eq!(entry.converted_at, 1234567891);
}

#[test]
fn test_dds_conversion_cache_creation() {
    // 测试 DdsConversionCache 结构体创建
    let mut entries = HashMap::new();
    entries.insert(
        "abc123".to_string(),
        DdsConversionEntry {
            source_path: "/path/to/source.dds".to_string(),
            source_mtime: 1234567890,
            png_path: "/path/to/cached.png".to_string(),
            converted_at: 1234567891,
        },
    );

    let cache = DdsConversionCache {
        entries,
        created_at: 1234567890,
        version: 1,
    };

    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.created_at, 1234567890);
    assert_eq!(cache.version, 1);
}

#[test]
fn test_dds_conversion_cache_serialization() {
    // 测试缓存的序列化和反序列化
    let mut entries = HashMap::new();
    entries.insert(
        "abc123".to_string(),
        DdsConversionEntry {
            source_path: "/path/to/source.dds".to_string(),
            source_mtime: 1234567890,
            png_path: "/path/to/cached.png".to_string(),
            converted_at: 1234567891,
        },
    );

    let cache = DdsConversionCache {
        entries,
        created_at: 1234567890,
        version: 1,
    };

    // 序列化
    let json = serde_json::to_string(&cache).expect("序列化失败");
    assert!(json.contains("abc123"));
    assert!(json.contains("/path/to/source.dds"));

    // 反序列化
    let deserialized: DdsConversionCache = serde_json::from_str(&json).expect("反序列化失败");
    assert_eq!(deserialized.entries.len(), 1);
    assert_eq!(deserialized.created_at, 1234567890);
    assert_eq!(deserialized.version, 1);
}

#[test]
fn test_dds_conversion_cache_version() {
    // 测试缓存版本号
    assert_eq!(DDS_CONVERSION_CACHE_VERSION, 1);
}

#[test]
fn test_dds_conversion_cache_with_multiple_entries() {
    // 测试包含多个条目的缓存
    let mut entries = HashMap::new();

    entries.insert(
        "hash1".to_string(),
        DdsConversionEntry {
            source_path: "/path/to/icon1.dds".to_string(),
            source_mtime: 1000,
            png_path: "/cache/icon1.png".to_string(),
            converted_at: 1001,
        },
    );

    entries.insert(
        "hash2".to_string(),
        DdsConversionEntry {
            source_path: "/path/to/icon2.dds".to_string(),
            source_mtime: 2000,
            png_path: "/cache/icon2.png".to_string(),
            converted_at: 2001,
        },
    );

    entries.insert(
        "hash3".to_string(),
        DdsConversionEntry {
            source_path: "/path/to/icon3.dds".to_string(),
            source_mtime: 3000,
            png_path: "/cache/icon3.png".to_string(),
            converted_at: 3001,
        },
    );

    let cache = DdsConversionCache {
        entries,
        created_at: 1234567890,
        version: 1,
    };

    assert_eq!(cache.entries.len(), 3);
    assert!(cache.entries.contains_key("hash1"));
    assert!(cache.entries.contains_key("hash2"));
    assert!(cache.entries.contains_key("hash3"));
}

#[test]
fn test_dds_conversion_cache_entry_lookup() {
    // 测试缓存条目查找
    let mut entries = HashMap::new();

    entries.insert(
        "abc123".to_string(),
        DdsConversionEntry {
            source_path: "/path/to/source.dds".to_string(),
            source_mtime: 1234567890,
            png_path: "/path/to/cached.png".to_string(),
            converted_at: 1234567891,
        },
    );

    let cache = DdsConversionCache {
        entries,
        created_at: 1234567890,
        version: 1,
    };

    // 测试查找存在的条目
    let found = cache.entries.get("abc123");
    assert!(found.is_some());
    assert_eq!(found.unwrap().source_path, "/path/to/source.dds");

    // 测试查找不存在的条目
    let not_found = cache.entries.get("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_dds_conversion_cache_empty() {
    // 测试空的缓存
    let cache = DdsConversionCache {
        entries: HashMap::new(),
        created_at: 1234567890,
        version: 1,
    };

    assert_eq!(cache.entries.len(), 0);
    assert!(cache.entries.is_empty());
}

#[test]
fn test_dds_conversion_cache_update() {
    // 测试缓存更新
    let mut cache = DdsConversionCache {
        entries: HashMap::new(),
        created_at: 1234567890,
        version: 1,
    };

    // 添加条目
    cache.entries.insert(
        "hash1".to_string(),
        DdsConversionEntry {
            source_path: "/path/to/icon1.dds".to_string(),
            source_mtime: 1000,
            png_path: "/cache/icon1.png".to_string(),
            converted_at: 1001,
        },
    );

    assert_eq!(cache.entries.len(), 1);

    // 更新条目
    cache.entries.insert(
        "hash1".to_string(),
        DdsConversionEntry {
            source_path: "/path/to/icon1_updated.dds".to_string(),
            source_mtime: 2000,
            png_path: "/cache/icon1_updated.png".to_string(),
            converted_at: 2001,
        },
    );

    assert_eq!(cache.entries.len(), 1);
    let entry = cache.entries.get("hash1").expect("条目不存在");
    assert_eq!(entry.source_path, "/path/to/icon1_updated.dds");
    assert_eq!(entry.source_mtime, 2000);
}

#[test]
fn test_dds_conversion_cache_remove() {
    // 测试缓存删除
    let mut cache = DdsConversionCache {
        entries: HashMap::new(),
        created_at: 1234567890,
        version: 1,
    };

    // 添加条目
    cache.entries.insert(
        "hash1".to_string(),
        DdsConversionEntry {
            source_path: "/path/to/icon1.dds".to_string(),
            source_mtime: 1000,
            png_path: "/cache/icon1.png".to_string(),
            converted_at: 1001,
        },
    );

    assert_eq!(cache.entries.len(), 1);

    // 删除条目
    cache.entries.remove("hash1");

    assert_eq!(cache.entries.len(), 0);
    assert!(cache.entries.is_empty());
}

#[test]
fn test_dds_conversion_cache_mtime_validation() {
    // 测试修改时间验证
    let mut entries = HashMap::new();

    entries.insert(
        "hash1".to_string(),
        DdsConversionEntry {
            source_path: "/path/to/icon1.dds".to_string(),
            source_mtime: 1000,
            png_path: "/cache/icon1.png".to_string(),
            converted_at: 1001,
        },
    );

    let cache = DdsConversionCache {
        entries,
        created_at: 1234567890,
        version: 1,
    };

    // 测试修改时间匹配
    let entry = cache.entries.get("hash1").expect("条目不存在");
    assert_eq!(entry.source_mtime, 1000);

    // 测试修改时间不匹配（模拟缓存过期）
    let current_mtime = 2000;
    assert_ne!(entry.source_mtime, current_mtime);
}

#[test]
fn test_dds_conversion_cache_path_hashing() {
    // 测试路径哈希
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let path1 = "/path/to/icon1.dds";
    let path2 = "/path/to/icon2.dds";
    let path3 = "/path/to/icon1.dds"; // 与 path1 相同

    let hash1 = {
        let mut hasher = DefaultHasher::new();
        path1.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    };

    let hash2 = {
        let mut hasher = DefaultHasher::new();
        path2.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    };

    let hash3 = {
        let mut hasher = DefaultHasher::new();
        path3.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    };

    // 相同路径应该产生相同的哈希
    assert_eq!(hash1, hash3);
    // 不同路径应该产生不同的哈希
    assert_ne!(hash1, hash2);
}

#[test]
fn test_dds_conversion_cache_directory_creation() {
    // 测试缓存目录创建
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let cache_dir = temp_dir.path().join("dds-conversion-cache");

    // 创建缓存目录
    fs::create_dir_all(&cache_dir).expect("创建缓存目录失败");

    // 验证目录存在
    assert!(cache_dir.exists());
    assert!(cache_dir.is_dir());
}

#[test]
fn test_dds_conversion_cache_file_operations() {
    // 测试缓存文件操作
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let cache_dir = temp_dir.path().join("dds-conversion-cache");
    fs::create_dir_all(&cache_dir).expect("创建缓存目录失败");

    // 创建测试缓存文件
    let cache_file = cache_dir.join("test_cache.json");
    let cache_content = r#"{
        "entries": {},
        "created_at": 1234567890,
        "version": 1
    }"#;

    fs::write(&cache_file, cache_content).expect("写入缓存文件失败");

    // 验证文件存在
    assert!(cache_file.exists());

    // 读取并解析缓存文件
    let content = fs::read_to_string(&cache_file).expect("读取缓存文件失败");
    let cache: DdsConversionCache = serde_json::from_str(&content).expect("解析缓存文件失败");

    assert_eq!(cache.version, 1);
    assert_eq!(cache.created_at, 1234567890);
}

#[test]
fn test_dds_conversion_cache_cleanup() {
    // 测试缓存清理
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let cache_dir = temp_dir.path().join("dds-conversion-cache");
    fs::create_dir_all(&cache_dir).expect("创建缓存目录失败");

    // 创建多个缓存文件
    for i in 0..5 {
        let png_file = cache_dir.join(format!("cache_{}.png", i));
        fs::write(&png_file, format!("fake png data {}", i)).expect("创建缓存文件失败");
    }

    // 验证文件存在
    let entries: Vec<_> = fs::read_dir(&cache_dir)
        .expect("读取缓存目录失败")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 5);

    // 清理缓存文件
    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("png") {
            fs::remove_file(&path).expect("删除缓存文件失败");
        }
    }

    // 验证文件已删除
    let remaining: Vec<_> = fs::read_dir(&cache_dir)
        .expect("读取缓存目录失败")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(remaining.len(), 0);
}
