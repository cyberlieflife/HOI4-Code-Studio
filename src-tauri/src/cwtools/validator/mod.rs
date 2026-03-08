//! 验证器模块
//!
//! 负责基于规则验证 AST 的正确性

pub mod core;
pub mod reference;
pub mod scope;

pub use core::{ValidationContext, ValidationResult, Validator};
pub use reference::ReferenceChecker;
pub use scope::{Scope, ScopeError, ScopeManager, ScopeTransition};
