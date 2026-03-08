//! 规则引擎模块
//!
//! 负责加载和管理 CWT 规则文件，提供规则数据结构和验证逻辑

pub mod loader;
pub mod types;

#[cfg(test)]
mod types_test;

// 重新导出常用类型
pub use loader::{RuleError, RuleLoader};
pub use types::{
    AliasRule, EnumDefinition, FieldType, ModifierCategory, ModifierDefinition, Rule, RuleOptions,
    RuleSet, RuleType, SubTypeDefinition, TypeDefinition, TypeOptions, ValueType,
};
