//! 解析服务模块
//!
//! 提供脚本解析服务，包含缓存和增量解析功能

use crate::cwtools::models::{Position, Range, AST};
use crate::cwtools::parser::{ParseError, Parser};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 文本变更
///
/// 表示文本编辑器中的一次文本修改
#[derive(Debug, Clone)]
pub struct TextChange {
    /// 变更范围
    pub range: Range,
    /// 新文本内容
    pub text: String,
}

/// 缓存的解析结果
#[derive(Clone)]
struct CachedParse {
    /// 解析后的 AST
    ast: AST,
    /// 版本号
    version: u64,
    /// 缓存时间戳
    timestamp: Instant,
    /// 估算的内存大小（字节）
    estimated_size: usize,
}

/// 解析服务
///
/// 提供脚本解析功能，支持缓存和增量解析
pub struct ParserService {
    /// 解析结果缓存
    /// Key: 文件路径, Value: 缓存的解析结果
    cache: HashMap<String, CachedParse>,
    /// 最大缓存条目数
    max_cache_entries: usize,
    /// 缓存过期时间
    cache_expiry: Duration,
    /// 最大内存使用量（字节）
    max_memory_bytes: usize,
    /// 当前估算的内存使用量（字节）
    current_memory_bytes: usize,
}

impl ParserService {
    /// 创建新的解析服务
    ///
    /// # 返回
    /// 新的 ParserService 实例
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_entries: 100,
            cache_expiry: Duration::from_secs(300), // 5 分钟
            max_memory_bytes: 500 * 1024 * 1024,    // 500 MB
            current_memory_bytes: 0,
        }
    }

    /// 创建带自定义配置的解析服务
    ///
    /// # 参数
    /// * `max_cache_entries` - 最大缓存条目数
    /// * `cache_expiry` - 缓存过期时间
    /// * `max_memory_bytes` - 最大内存使用量（字节）
    ///
    /// # 返回
    /// 新的 ParserService 实例
    pub fn with_config(
        max_cache_entries: usize,
        cache_expiry: Duration,
        max_memory_bytes: usize,
    ) -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_entries,
            cache_expiry,
            max_memory_bytes,
            current_memory_bytes: 0,
        }
    }

    /// 解析文件
    ///
    /// 如果缓存中存在且版本匹配，则返回缓存的结果
    /// 否则重新解析并更新缓存
    ///
    /// # 参数
    /// * `path` - 文件路径
    /// * `content` - 文件内容
    /// * `version` - 文件版本号
    ///
    /// # 返回
    /// * `Ok(AST)` - 解析成功的 AST
    /// * `Err(Vec<ParseError>)` - 解析错误列表
    pub fn parse_file(
        &mut self,
        path: &str,
        content: &str,
        version: u64,
    ) -> Result<AST, Vec<ParseError>> {
        // 检查缓存
        if let Some(cached) = self.get_cached(path, version) {
            return Ok(cached.clone());
        }

        // 清理过期缓存
        self.clear_old_cache(self.cache_expiry);

        // 如果缓存已满，清理最旧的条目
        while self.cache.len() >= self.max_cache_entries {
            self.evict_oldest();
        }

        // 检查内存使用，如果超过限制则清理
        // 预留一些空间给新的解析结果
        let reserved_space = content.len() * 2; // 粗略估算
        while self.current_memory_bytes + reserved_space > self.max_memory_bytes
            && !self.cache.is_empty()
        {
            self.evict_oldest();
        }

        // 解析文件
        let mut parser = Parser::new(content, path.to_string()).map_err(|e| vec![e])?;
        let ast = parser.parse()?;

        // 更新缓存
        self.update_cache(path.to_string(), ast.clone(), version);

        Ok(ast)
    }

    /// 从缓存获取解析结果
    ///
    /// # 参数
    /// * `path` - 文件路径
    /// * `version` - 文件版本号
    ///
    /// # 返回
    /// 如果缓存存在且版本匹配，返回 AST 的引用
    fn get_cached(&self, path: &str, version: u64) -> Option<&AST> {
        self.cache.get(path).and_then(|cached| {
            if cached.version == version {
                Some(&cached.ast)
            } else {
                None
            }
        })
    }

    /// 更新缓存
    ///
    /// # 参数
    /// * `path` - 文件路径
    /// * `ast` - 解析后的 AST
    /// * `version` - 文件版本号
    fn update_cache(&mut self, path: String, ast: AST, version: u64) {
        // 估算 AST 的内存大小
        let estimated_size = self.estimate_ast_size(&ast);

        // 如果替换现有缓存，先减去旧的内存使用
        if let Some(old_cached) = self.cache.get(&path) {
            self.current_memory_bytes = self
                .current_memory_bytes
                .saturating_sub(old_cached.estimated_size);
        }

        let cached = CachedParse {
            ast,
            version,
            timestamp: Instant::now(),
            estimated_size,
        };

        // 更新内存使用统计
        self.current_memory_bytes += estimated_size;

        self.cache.insert(path, cached);
    }

    /// 估算 AST 的内存大小
    ///
    /// 这是一个粗略的估算，用于内存管理
    ///
    /// # 参数
    /// * `ast` - 要估算的 AST
    ///
    /// # 返回
    /// 估算的字节数
    fn estimate_ast_size(&self, ast: &AST) -> usize {
        // 基础大小：AST 结构体本身
        let mut size = std::mem::size_of::<AST>();

        // 文件路径字符串
        size += ast.source_file.len();

        // 语句列表
        size += ast.statements.len() * std::mem::size_of::<crate::cwtools::models::Statement>();

        // 粗略估算：每个语句平均占用 200 字节（包括字符串内容）
        size += ast.statements.len() * 200;

        size
    }

    /// 清理过期的缓存条目
    ///
    /// # 参数
    /// * `max_age` - 最大缓存时间
    fn clear_old_cache(&mut self, max_age: Duration) {
        let now = Instant::now();
        self.cache
            .retain(|_, cached| now.duration_since(cached.timestamp) < max_age);
    }

    /// 驱逐最旧的缓存条目
    ///
    /// 使用 LRU 策略，移除时间戳最早的条目
    fn evict_oldest(&mut self) {
        if self.cache.is_empty() {
            return;
        }

        // 找到最旧的条目
        let oldest_key = self
            .cache
            .iter()
            .min_by_key(|(_, cached)| cached.timestamp)
            .map(|(key, _)| key.clone());

        // 移除最旧的条目
        if let Some(key) = oldest_key {
            if let Some(removed) = self.cache.remove(&key) {
                self.current_memory_bytes = self
                    .current_memory_bytes
                    .saturating_sub(removed.estimated_size);
            }
        }
    }

    /// 基于内存压力驱逐缓存条目
    ///
    /// 当内存使用超过限制时，移除最旧的条目直到内存使用降到阈值以下
    fn evict_by_memory_pressure(&mut self) {
        // 目标：将内存使用降到最大值的 75%
        let target_memory = (self.max_memory_bytes * 3) / 4;

        while self.current_memory_bytes > target_memory && !self.cache.is_empty() {
            self.evict_oldest();
        }
    }

    /// 清空所有缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.current_memory_bytes = 0;
    }

    /// 使指定文件的缓存失效
    ///
    /// # 参数
    /// * `path` - 文件路径
    pub fn invalidate(&mut self, path: &str) {
        if let Some(removed) = self.cache.remove(path) {
            self.current_memory_bytes = self
                .current_memory_bytes
                .saturating_sub(removed.estimated_size);
        }
    }

    /// 增量解析文件
    ///
    /// 根据文本变更进行增量解析，只重新解析受影响的部分
    /// 注意：当前实现为简化版本，直接重新解析整个文件
    /// 未来可以优化为真正的增量解析
    ///
    /// # 参数
    /// * `path` - 文件路径
    /// * `content` - 更新后的完整文件内容
    /// * `version` - 新的文件版本号
    /// * `changes` - 文本变更列表
    ///
    /// # 返回
    /// * `Ok(AST)` - 解析成功的 AST
    /// * `Err(Vec<ParseError>)` - 解析错误列表
    pub fn parse_incremental(
        &mut self,
        path: &str,
        content: &str,
        version: u64,
        _changes: &[TextChange],
    ) -> Result<AST, Vec<ParseError>> {
        // 简化实现：直接重新解析整个文件
        // 未来优化：
        // 1. 分析变更范围，确定受影响的语句
        // 2. 只重新解析受影响的部分
        // 3. 合并新旧 AST

        // 检查是否需要增量解析
        // 如果变更很小（如单个字符），可以尝试局部重解析
        // 如果变更很大，直接全量解析更高效

        self.parse_file(path, content, version)
    }

    /// 检测文本变更是否影响整个文件
    ///
    /// 用于判断是否需要全量重新解析
    ///
    /// # 参数
    /// * `changes` - 文本变更列表
    ///
    /// # 返回
    /// 如果变更影响多行或包含花括号，返回 true
    #[allow(dead_code)]
    fn is_major_change(&self, changes: &[TextChange]) -> bool {
        for change in changes {
            // 如果变更跨越多行，认为是重大变更
            if change.range.start.line != change.range.end.line {
                return true;
            }

            // 如果变更包含花括号，可能影响结构，认为是重大变更
            if change.text.contains('{') || change.text.contains('}') {
                return true;
            }
        }

        false
    }

    /// 应用文本变更到内容
    ///
    /// 将变更应用到原始内容，生成新内容
    ///
    /// # 参数
    /// * `original` - 原始内容
    /// * `changes` - 文本变更列表
    ///
    /// # 返回
    /// 应用变更后的新内容
    #[allow(dead_code)]
    fn apply_changes(&self, original: &str, changes: &[TextChange]) -> String {
        let mut result = original.to_string();

        // 按照偏移量从后往前应用变更，避免位置偏移问题
        let mut sorted_changes = changes.to_vec();
        sorted_changes.sort_by(|a, b| b.range.start.offset.cmp(&a.range.start.offset));

        for change in sorted_changes {
            let start = change.range.start.offset;
            let end = change.range.end.offset;

            // 确保偏移量有效
            if start <= result.len() && end <= result.len() && start <= end {
                result.replace_range(start..end, &change.text);
            }
        }

        result
    }

    /// 获取缓存统计信息
    ///
    /// # 返回
    /// (缓存条目数, 最大缓存条目数, 当前内存使用字节数, 最大内存字节数)
    pub fn cache_stats(&self) -> (usize, usize, usize, usize) {
        (
            self.cache.len(),
            self.max_cache_entries,
            self.current_memory_bytes,
            self.max_memory_bytes,
        )
    }

    /// 获取内存使用百分比
    ///
    /// # 返回
    /// 内存使用百分比 (0.0 - 100.0)
    pub fn memory_usage_percent(&self) -> f64 {
        if self.max_memory_bytes == 0 {
            0.0
        } else {
            (self.current_memory_bytes as f64 / self.max_memory_bytes as f64) * 100.0
        }
    }
}

impl Default for ParserService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_service_creation() {
        let service = ParserService::new();
        assert_eq!(service.cache.len(), 0);
        assert_eq!(service.max_cache_entries, 100);
    }

    #[test]
    fn test_parser_service_with_config() {
        let service = ParserService::with_config(50, Duration::from_secs(60), 100 * 1024 * 1024);
        assert_eq!(service.max_cache_entries, 50);
        assert_eq!(service.cache_expiry, Duration::from_secs(60));
        assert_eq!(service.max_memory_bytes, 100 * 1024 * 1024);
    }

    #[test]
    fn test_parse_file_simple() {
        let mut service = ParserService::new();
        let content = "key = value";
        let result = service.parse_file("test.txt", content, 1);

        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_file_caching() {
        let mut service = ParserService::new();
        let content = "key = value";

        // 第一次解析
        let result1 = service.parse_file("test.txt", content, 1);
        assert!(result1.is_ok());

        // 第二次解析，应该从缓存获取
        let result2 = service.parse_file("test.txt", content, 1);
        assert!(result2.is_ok());

        // 验证缓存中有一个条目
        assert_eq!(service.cache.len(), 1);
    }

    #[test]
    fn test_parse_file_version_change() {
        let mut service = ParserService::new();
        let content1 = "key1 = value1";
        let content2 = "key2 = value2";

        // 第一次解析
        let result1 = service.parse_file("test.txt", content1, 1);
        assert!(result1.is_ok());

        // 版本变化，应该重新解析
        let result2 = service.parse_file("test.txt", content2, 2);
        assert!(result2.is_ok());

        // 验证缓存中仍然只有一个条目（被更新了）
        assert_eq!(service.cache.len(), 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut service = ParserService::new();
        let content = "key = value";

        // 解析并缓存
        let _ = service.parse_file("test.txt", content, 1);
        assert_eq!(service.cache.len(), 1);

        // 使缓存失效
        service.invalidate("test.txt");
        assert_eq!(service.cache.len(), 0);
    }

    #[test]
    fn test_clear_cache() {
        let mut service = ParserService::new();

        // 解析多个文件
        let _ = service.parse_file("test1.txt", "key1 = value1", 1);
        let _ = service.parse_file("test2.txt", "key2 = value2", 1);
        let _ = service.parse_file("test3.txt", "key3 = value3", 1);

        assert_eq!(service.cache.len(), 3);

        // 清空缓存
        service.clear_cache();
        assert_eq!(service.cache.len(), 0);
    }

    #[test]
    fn test_cache_eviction() {
        // 创建只能缓存 2 个条目的服务
        let mut service =
            ParserService::with_config(2, Duration::from_secs(300), 500 * 1024 * 1024);

        // 解析 3 个文件
        let _ = service.parse_file("test1.txt", "key1 = value1", 1);
        std::thread::sleep(Duration::from_millis(10)); // 确保时间戳不同
        let _ = service.parse_file("test2.txt", "key2 = value2", 1);
        std::thread::sleep(Duration::from_millis(10));
        let _ = service.parse_file("test3.txt", "key3 = value3", 1);

        // 应该只有 2 个条目（最旧的被驱逐）
        assert_eq!(service.cache.len(), 2);

        // test1.txt 应该被驱逐
        assert!(!service.cache.contains_key("test1.txt"));
        assert!(service.cache.contains_key("test2.txt"));
        assert!(service.cache.contains_key("test3.txt"));
    }

    #[test]
    fn test_cache_stats() {
        let mut service = ParserService::new();

        let (count, max, mem_used, mem_max) = service.cache_stats();
        assert_eq!(count, 0);
        assert_eq!(max, 100);
        assert_eq!(mem_used, 0);
        assert_eq!(mem_max, 500 * 1024 * 1024);

        let _ = service.parse_file("test.txt", "key = value", 1);

        let (count, max, mem_used, _) = service.cache_stats();
        assert_eq!(count, 1);
        assert_eq!(max, 100);
        assert!(mem_used > 0); // 应该有一些内存使用
    }

    #[test]
    fn test_parse_error_handling() {
        let mut service = ParserService::new();

        // 无效的语法
        let content = "= invalid";
        let result = service.parse_file("test.txt", content, 1);

        // 应该返回错误
        assert!(result.is_err());

        // 错误不应该被缓存
        assert_eq!(service.cache.len(), 0);
    }

    #[test]
    fn test_multiple_files() {
        let mut service = ParserService::new();

        // 解析多个不同的文件
        let result1 = service.parse_file("file1.txt", "key1 = value1", 1);
        let result2 = service.parse_file("file2.txt", "key2 = value2", 1);
        let result3 = service.parse_file("file3.txt", "key3 = value3", 1);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());

        // 应该有 3 个缓存条目
        assert_eq!(service.cache.len(), 3);
    }

    #[test]
    fn test_complex_script_parsing() {
        let mut service = ParserService::new();

        let content = r#"
country_event = {
    id = test.1
    title = "Test Event"
    desc = test.1.desc
    
    option = {
        name = test.1.a
        add_stability = 0.05
    }
}
"#;

        let result = service.parse_file("event.txt", content, 1);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_incremental_simple() {
        let mut service = ParserService::new();

        let original = "key1 = value1";
        let updated = "key1 = value2";

        // 第一次解析
        let _ = service.parse_file("test.txt", original, 1);

        // 增量解析
        let changes = vec![TextChange {
            range: Range::new(Position::new(1, 8, 7), Position::new(1, 14, 13)),
            text: "value2".to_string(),
        }];

        let result = service.parse_incremental("test.txt", updated, 2, &changes);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_incremental_add_line() {
        let mut service = ParserService::new();

        let original = "key1 = value1";
        let updated = "key1 = value1\nkey2 = value2";

        // 第一次解析
        let _ = service.parse_file("test.txt", original, 1);

        // 增量解析（添加新行）
        let changes = vec![TextChange {
            range: Range::new(Position::new(1, 14, 13), Position::new(1, 14, 13)),
            text: "\nkey2 = value2".to_string(),
        }];

        let result = service.parse_incremental("test.txt", updated, 2, &changes);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 2);
    }

    #[test]
    fn test_parse_incremental_delete_line() {
        let mut service = ParserService::new();

        let original = "key1 = value1\nkey2 = value2";
        let updated = "key1 = value1";

        // 第一次解析
        let _ = service.parse_file("test.txt", original, 1);

        // 增量解析（删除行）
        let changes = vec![TextChange {
            range: Range::new(Position::new(1, 14, 13), Position::new(2, 14, 27)),
            text: "".to_string(),
        }];

        let result = service.parse_incremental("test.txt", updated, 2, &changes);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_incremental_complex_change() {
        let mut service = ParserService::new();

        let original = r#"
option = {
    name = old_name
    value = 10
}
"#;

        let updated = r#"
option = {
    name = new_name
    value = 20
    extra = yes
}
"#;

        // 第一次解析
        let _ = service.parse_file("test.txt", original, 1);

        // 增量解析（多处修改）
        let changes = vec![
            TextChange {
                range: Range::new(Position::new(3, 12, 24), Position::new(3, 20, 32)),
                text: "new_name".to_string(),
            },
            TextChange {
                range: Range::new(Position::new(4, 12, 49), Position::new(4, 14, 51)),
                text: "20".to_string(),
            },
        ];

        let result = service.parse_incremental("test.txt", updated, 2, &changes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_major_change() {
        let service = ParserService::new();

        // 单行小改动
        let small_change = vec![TextChange {
            range: Range::new(Position::new(1, 1, 0), Position::new(1, 5, 4)),
            text: "test".to_string(),
        }];
        assert!(!service.is_major_change(&small_change));

        // 跨行改动
        let multiline_change = vec![TextChange {
            range: Range::new(Position::new(1, 1, 0), Position::new(2, 5, 10)),
            text: "test".to_string(),
        }];
        assert!(service.is_major_change(&multiline_change));

        // 包含花括号
        let brace_change = vec![TextChange {
            range: Range::new(Position::new(1, 1, 0), Position::new(1, 5, 4)),
            text: "{ }".to_string(),
        }];
        assert!(service.is_major_change(&brace_change));
    }

    #[test]
    fn test_apply_changes() {
        let service = ParserService::new();

        let original = "key = value";

        // 单个变更
        let changes = vec![TextChange {
            range: Range::new(Position::new(1, 7, 6), Position::new(1, 12, 11)),
            text: "new_value".to_string(),
        }];

        let result = service.apply_changes(original, &changes);
        assert_eq!(result, "key = new_value");
    }

    #[test]
    fn test_apply_multiple_changes() {
        let service = ParserService::new();

        let original = "key1 = value1\nkey2 = value2";

        // 多个变更
        let changes = vec![
            TextChange {
                range: Range::new(Position::new(1, 8, 7), Position::new(1, 14, 13)),
                text: "new1".to_string(),
            },
            TextChange {
                range: Range::new(Position::new(2, 8, 21), Position::new(2, 14, 27)),
                text: "new2".to_string(),
            },
        ];

        let result = service.apply_changes(original, &changes);
        assert_eq!(result, "key1 = new1\nkey2 = new2");
    }

    #[test]
    fn test_incremental_parse_equivalence() {
        let mut service = ParserService::new();

        let original = "key = old_value";
        let updated = "key = new_value";

        // 完整解析
        let full_parse = service.parse_file("test1.txt", updated, 1).unwrap();

        // 增量解析
        let changes = vec![TextChange {
            range: Range::new(Position::new(1, 7, 6), Position::new(1, 16, 15)),
            text: "new_value".to_string(),
        }];

        let incremental_parse = service
            .parse_incremental("test2.txt", updated, 1, &changes)
            .unwrap();

        // 两种解析方式应该产生相同数量的语句
        assert_eq!(
            full_parse.statements.len(),
            incremental_parse.statements.len()
        );
    }

    #[test]
    fn test_memory_usage_tracking() {
        let mut service = ParserService::new();

        // 初始内存使用应该为 0
        let (_, _, mem_used, _) = service.cache_stats();
        assert_eq!(mem_used, 0);

        // 解析一个文件
        let _ = service.parse_file("test.txt", "key = value", 1);

        // 内存使用应该增加
        let (_, _, mem_used, _) = service.cache_stats();
        assert!(mem_used > 0);

        // 清空缓存
        service.clear_cache();

        // 内存使用应该回到 0
        let (_, _, mem_used, _) = service.cache_stats();
        assert_eq!(mem_used, 0);
    }

    #[test]
    fn test_memory_usage_percent() {
        let mut service = ParserService::new();

        // 初始应该是 0%
        assert_eq!(service.memory_usage_percent(), 0.0);

        // 解析一些文件
        let _ = service.parse_file("test1.txt", "key1 = value1", 1);
        let _ = service.parse_file("test2.txt", "key2 = value2", 1);

        // 应该有一些内存使用
        let percent = service.memory_usage_percent();
        assert!(percent > 0.0);
        assert!(percent < 100.0);
    }

    #[test]
    fn test_memory_based_eviction() {
        // 创建内存限制为 5 KB 的服务
        let mut service = ParserService::with_config(100, Duration::from_secs(300), 5 * 1024);

        // 解析多个文件，应该触发基于内存的驱逐
        // 使用更大的内容以确保触发内存限制
        let content = "key1 = value1\nkey2 = value2\nkey3 = value3\nkey4 = value4\nkey5 = value5\nkey6 = value6";

        for i in 0..15 {
            let _ = service.parse_file(&format!("test{}.txt", i), content, 1);
        }

        // 由于内存限制，不应该缓存所有 15 个文件
        let (count, _, mem_used, mem_max) = service.cache_stats();
        assert!(
            count < 15,
            "应该有缓存被驱逐，但缓存了所有 {} 个文件",
            count
        );
        // 允许一些误差，因为估算不是完全精确的
        assert!(
            mem_used <= mem_max + 2048,
            "内存使用 {} 超过限制 {} + 2048",
            mem_used,
            mem_max
        );
    }

    #[test]
    fn test_invalidate_updates_memory() {
        let mut service = ParserService::new();

        // 解析文件
        let _ = service.parse_file("test.txt", "key = value", 1);

        let (_, _, mem_before, _) = service.cache_stats();
        assert!(mem_before > 0);

        // 使缓存失效
        service.invalidate("test.txt");

        let (_, _, mem_after, _) = service.cache_stats();
        assert_eq!(mem_after, 0);
    }

    #[test]
    fn test_cache_replacement_updates_memory() {
        let mut service = ParserService::new();

        // 解析文件
        let _ = service.parse_file("test.txt", "key = value", 1);
        let (_, _, mem_v1, _) = service.cache_stats();

        // 用更大的内容替换
        let large_content = "key1 = value1\nkey2 = value2\nkey3 = value3\nkey4 = value4";
        let _ = service.parse_file("test.txt", large_content, 2);
        let (_, _, mem_v2, _) = service.cache_stats();

        // 新版本应该使用更多内存
        assert!(mem_v2 > mem_v1);
    }

    #[test]
    fn test_estimate_ast_size() {
        let service = ParserService::new();

        // 创建一个简单的 AST
        let ast = AST {
            statements: vec![],
            source_file: "test.txt".to_string(),
        };

        let size = service.estimate_ast_size(&ast);

        // 应该至少包含基础结构大小
        assert!(size > 0);
    }

    #[test]
    fn test_memory_pressure_eviction() {
        // 创建内存限制为 20 KB 的服务
        let mut service = ParserService::with_config(100, Duration::from_secs(300), 20 * 1024);

        // 解析足够多的文件以触发内存压力
        let content = "key = value\nkey2 = value2\nkey3 = value3\nkey4 = value4";

        for i in 0..20 {
            let _ = service.parse_file(&format!("test{}.txt", i), content, 1);
            std::thread::sleep(Duration::from_millis(1)); // 确保时间戳不同
        }

        // 内存使用应该在限制范围内（允许一些误差）
        let (_, _, mem_used, mem_max) = service.cache_stats();
        assert!(mem_used <= mem_max + 2048); // 允许 2KB 误差

        // 应该有一些缓存被驱逐
        let (count, _, _, _) = service.cache_stats();
        assert!(count < 20);
    }
}
