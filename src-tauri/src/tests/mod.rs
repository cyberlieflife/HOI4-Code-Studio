//! 单元测试模块
//! 
//! 这个模块包含了项目中所有核心模块的单元测试。
//! 每个测试文件都应该以 `mod_tests_` 为前缀。

pub mod file_tree_tests;
pub mod tag_validator_tests;
pub mod dependency_tests;
pub mod country_tags_tests;
pub mod cwtools_compatibility_tests;
pub mod gfx_index_cache_tests;
pub mod dds_conversion_cache_tests;

#[cfg(test)]
mod helpers {
    use super::*;
    use tempfile::{tempdir, TempDir};
    use std::fs;
    use std::path::PathBuf;

    /// 创建测试用的临时目录
    pub fn create_test_dir() -> TempDir {
        tempdir().expect("Failed to create temporary directory")
    }

    /// 创建测试文件
    pub fn create_test_file(path: &PathBuf, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directory");
        }
        fs::write(path, content).expect("Failed to write test file");
    }

    /// 创建测试目录结构
    pub fn create_test_directory_structure(base_path: &PathBuf) {
        // 创建测试文件
        let test_file = base_path.join("test.txt");
        create_test_file(&test_file, "test content");

        // 创建子目录
        let sub_dir = base_path.join("subdir");
        fs::create_dir_all(&sub_dir).expect("Failed to create subdirectory");

        // 在子目录中创建文件
        let sub_file = sub_dir.join("sub_test.txt");
        create_test_file(&sub_file, "sub test content");
    }
}