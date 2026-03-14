// 命令层模块
//
// 此模块包含所有 Tauri 命令接口，负责：
// - 接收前端请求
// - 参数验证
// - 调用服务层执行业务逻辑
// - 返回结果给前端

pub mod cwtools;
pub mod file;
pub mod game;
pub mod gfx;
pub mod project;
pub mod settings;
pub mod terminal;

// 重新导出所有命令函数，便于在 lib.rs 中注册
pub use cwtools::*;
pub use file::*;
pub use game::*;
pub use gfx::*;
pub use project::*;
pub use settings::*;
pub use terminal::*;
