//! cwtools HOI4 语法验证模块
//!
//! 本模块实现了 Paradox 脚本的解析、验证和诊断功能

pub mod commands;
pub mod config;
pub mod diagnostic;
pub mod error_logger;
pub mod fallback;
pub mod formatter;
pub mod models;
pub mod parser;
pub mod rules;
pub mod services;
pub mod validator;
