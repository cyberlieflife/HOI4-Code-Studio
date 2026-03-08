//! 作用域管理模块
//!
//! 本模块实现 HOI4 脚本的作用域管理和验证功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HOI4 脚本作用域类型
///
/// 表示脚本执行的上下文环境
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    /// 国家作用域
    Country,
    /// 省份/州作用域
    State,
    /// 单位领导者作用域
    UnitLeader,
    /// 空军作用域
    Air,
    /// 任意作用域（通配符）
    Any,
}

impl Scope {
    /// 从字符串解析作用域
    ///
    /// # 参数
    /// * `s` - 作用域字符串
    ///
    /// # 返回
    /// 解析成功返回 Some(Scope)，失败返回 None
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "country" => Some(Scope::Country),
            "state" => Some(Scope::State),
            "unit_leader" | "unitleader" => Some(Scope::UnitLeader),
            "air" => Some(Scope::Air),
            "any" | "all" => Some(Scope::Any),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Country => "country",
            Scope::State => "state",
            Scope::UnitLeader => "unit_leader",
            Scope::Air => "air",
            Scope::Any => "any",
        }
    }

    /// 检查当前作用域是否匹配目标作用域
    ///
    /// Any 作用域可以匹配任何作用域
    ///
    /// # 参数
    /// * `target` - 目标作用域
    ///
    /// # 返回
    /// 如果匹配返回 true
    pub fn matches(&self, target: Scope) -> bool {
        *self == target || *self == Scope::Any || target == Scope::Any
    }
}

/// 作用域转换定义
///
/// 定义从一个作用域到另一个作用域的转换规则
#[derive(Debug, Clone)]
pub struct ScopeTransition {
    /// 源作用域列表（可以从这些作用域转换）
    pub from: Vec<Scope>,
    /// 目标作用域
    pub to: Scope,
    /// 触发转换的命令名称
    pub command: String,
}

impl ScopeTransition {
    /// 创建新的作用域转换
    pub fn new(from: Vec<Scope>, to: Scope, command: impl Into<String>) -> Self {
        Self {
            from,
            to,
            command: command.into(),
        }
    }

    /// 检查是否可以从指定作用域进行转换
    pub fn can_transition_from(&self, scope: Scope) -> bool {
        self.from.iter().any(|s| s.matches(scope))
    }
}

/// 作用域错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeError {
    /// 无效的作用域转换
    InvalidTransition {
        from: Scope,
        to: Scope,
        command: String,
    },
    /// 未知的转换命令
    UnknownCommand(String),
    /// 作用域栈为空
    EmptyStack,
    /// 作用域不匹配
    ScopeMismatch { expected: Vec<Scope>, actual: Scope },
}

impl std::fmt::Display for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeError::InvalidTransition { from, to, command } => {
                write!(
                    f,
                    "无效的作用域转换: 无法从 {} 通过命令 '{}' 转换到 {}",
                    from.as_str(),
                    command,
                    to.as_str()
                )
            }
            ScopeError::UnknownCommand(cmd) => {
                write!(f, "未知的作用域转换命令: {}", cmd)
            }
            ScopeError::EmptyStack => {
                write!(f, "作用域栈为空")
            }
            ScopeError::ScopeMismatch { expected, actual } => {
                let expected_str: Vec<_> = expected.iter().map(|s| s.as_str()).collect();
                write!(
                    f,
                    "作用域不匹配: 期望 {:?}, 实际 {}",
                    expected_str,
                    actual.as_str()
                )
            }
        }
    }
}

impl std::error::Error for ScopeError {}

/// 作用域管理器
///
/// 管理脚本验证过程中的作用域栈和作用域转换
pub struct ScopeManager {
    /// 作用域栈，栈顶是当前作用域
    scope_stack: Vec<Scope>,
    /// 作用域转换规则映射（命令名 -> 转换规则）
    scope_transitions: HashMap<String, ScopeTransition>,
}

impl ScopeManager {
    /// 创建新的作用域管理器
    ///
    /// 默认初始化为 Country 作用域，并加载 HOI4 的标准作用域转换规则
    pub fn new() -> Self {
        let mut manager = Self {
            scope_stack: vec![Scope::Country], // 默认从国家作用域开始
            scope_transitions: HashMap::new(),
        };
        manager.load_default_transitions();
        manager
    }

    /// 创建带有指定初始作用域的管理器
    pub fn with_initial_scope(initial_scope: Scope) -> Self {
        let mut manager = Self {
            scope_stack: vec![initial_scope],
            scope_transitions: HashMap::new(),
        };
        manager.load_default_transitions();
        manager
    }

    /// 加载 HOI4 的默认作用域转换规则
    fn load_default_transitions(&mut self) {
        // ROOT - 返回到根作用域（通常是国家）
        self.add_transition(ScopeTransition::new(
            vec![Scope::Any],
            Scope::Country,
            "ROOT",
        ));

        // FROM - 返回到调用者作用域
        // 这个需要运行时上下文，这里简化为返回上一层
        self.add_transition(ScopeTransition::new(
            vec![Scope::Any],
            Scope::Country,
            "FROM",
        ));

        // PREV - 返回到前一个作用域
        self.add_transition(ScopeTransition::new(
            vec![Scope::Any],
            Scope::Country,
            "PREV",
        ));

        // 国家 -> 州
        self.add_transition(ScopeTransition::new(
            vec![Scope::Country],
            Scope::State,
            "random_owned_controlled_state",
        ));
        self.add_transition(ScopeTransition::new(
            vec![Scope::Country],
            Scope::State,
            "every_owned_state",
        ));
        self.add_transition(ScopeTransition::new(
            vec![Scope::Country],
            Scope::State,
            "any_owned_state",
        ));

        // 国家 -> 单位领导者
        self.add_transition(ScopeTransition::new(
            vec![Scope::Country],
            Scope::UnitLeader,
            "random_unit_leader",
        ));
        self.add_transition(ScopeTransition::new(
            vec![Scope::Country],
            Scope::UnitLeader,
            "every_unit_leader",
        ));

        // 州 -> 国家
        self.add_transition(ScopeTransition::new(
            vec![Scope::State],
            Scope::Country,
            "owner",
        ));
        self.add_transition(ScopeTransition::new(
            vec![Scope::State],
            Scope::Country,
            "controller",
        ));
    }

    /// 添加作用域转换规则
    pub fn add_transition(&mut self, transition: ScopeTransition) {
        self.scope_transitions
            .insert(transition.command.clone(), transition);
    }

    /// 压入新的作用域到栈顶
    ///
    /// # 参数
    /// * `scope` - 要压入的作用域
    pub fn push_scope(&mut self, scope: Scope) {
        self.scope_stack.push(scope);
    }

    /// 弹出栈顶作用域
    ///
    /// # 返回
    /// 返回弹出的作用域，如果栈为空返回 None
    pub fn pop_scope(&mut self) -> Option<Scope> {
        // 保持至少一个作用域在栈中
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop()
        } else {
            None
        }
    }

    /// 获取当前作用域
    ///
    /// # 返回
    /// 返回栈顶作用域
    pub fn current_scope(&self) -> Scope {
        *self.scope_stack.last().unwrap_or(&Scope::Any)
    }

    /// 获取作用域栈的深度
    pub fn depth(&self) -> usize {
        self.scope_stack.len()
    }

    /// 检查是否可以从一个作用域转换到另一个作用域
    ///
    /// # 参数
    /// * `from` - 源作用域
    /// * `to` - 目标作用域
    ///
    /// # 返回
    /// 如果存在有效的转换规则返回 true
    pub fn can_transition(&self, from: Scope, to: Scope) -> bool {
        // Any 作用域可以转换到任何作用域
        if from == Scope::Any || to == Scope::Any {
            return true;
        }

        // 相同作用域总是可以转换
        if from == to {
            return true;
        }

        // 检查是否存在转换规则
        self.scope_transitions
            .values()
            .any(|t| t.to == to && t.can_transition_from(from))
    }

    /// 应用作用域转换命令
    ///
    /// # 参数
    /// * `command` - 转换命令名称
    ///
    /// # 返回
    /// 成功返回转换后的作用域，失败返回错误
    pub fn apply_transition(&mut self, command: &str) -> Result<Scope, ScopeError> {
        let current = self.current_scope();

        // 查找转换规则并克隆必要的数据以避免借用冲突
        if let Some(transition) = self.scope_transitions.get(command) {
            let target_scope = transition.to;
            let can_transition = transition.can_transition_from(current);

            // 检查是否可以从当前作用域转换
            if can_transition {
                self.push_scope(target_scope);
                Ok(target_scope)
            } else {
                Err(ScopeError::InvalidTransition {
                    from: current,
                    to: target_scope,
                    command: command.to_string(),
                })
            }
        } else {
            Err(ScopeError::UnknownCommand(command.to_string()))
        }
    }

    /// 验证当前作用域是否匹配期望的作用域列表
    ///
    /// # 参数
    /// * `expected_scopes` - 期望的作用域列表
    ///
    /// # 返回
    /// 如果当前作用域匹配任一期望作用域返回 Ok，否则返回错误
    pub fn validate_scope(&self, expected_scopes: &[Scope]) -> Result<(), ScopeError> {
        let current = self.current_scope();

        // 检查是否匹配任一期望作用域
        if expected_scopes.iter().any(|s| s.matches(current)) {
            Ok(())
        } else {
            Err(ScopeError::ScopeMismatch {
                expected: expected_scopes.to_vec(),
                actual: current,
            })
        }
    }

    /// 重置作用域管理器到初始状态
    pub fn reset(&mut self) {
        self.scope_stack.clear();
        self.scope_stack.push(Scope::Country);
    }

    /// 重置到指定作用域
    pub fn reset_to(&mut self, scope: Scope) {
        self.scope_stack.clear();
        self.scope_stack.push(scope);
    }

    /// 获取作用域栈的副本（用于调试）
    pub fn get_stack(&self) -> Vec<Scope> {
        self.scope_stack.clone()
    }
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_from_str() {
        assert_eq!(Scope::from_str("country"), Some(Scope::Country));
        assert_eq!(Scope::from_str("Country"), Some(Scope::Country));
        assert_eq!(Scope::from_str("state"), Some(Scope::State));
        assert_eq!(Scope::from_str("unit_leader"), Some(Scope::UnitLeader));
        assert_eq!(Scope::from_str("unitleader"), Some(Scope::UnitLeader));
        assert_eq!(Scope::from_str("air"), Some(Scope::Air));
        assert_eq!(Scope::from_str("any"), Some(Scope::Any));
        assert_eq!(Scope::from_str("invalid"), None);
    }

    #[test]
    fn test_scope_matches() {
        assert!(Scope::Country.matches(Scope::Country));
        assert!(!Scope::Country.matches(Scope::State));
        assert!(Scope::Any.matches(Scope::Country));
        assert!(Scope::Country.matches(Scope::Any));
        assert!(Scope::Any.matches(Scope::Any));
    }

    #[test]
    fn test_scope_manager_new() {
        let manager = ScopeManager::new();
        assert_eq!(manager.current_scope(), Scope::Country);
        assert_eq!(manager.depth(), 1);
    }

    #[test]
    fn test_scope_manager_with_initial_scope() {
        let manager = ScopeManager::with_initial_scope(Scope::State);
        assert_eq!(manager.current_scope(), Scope::State);
    }

    #[test]
    fn test_push_pop_scope() {
        let mut manager = ScopeManager::new();
        assert_eq!(manager.current_scope(), Scope::Country);

        manager.push_scope(Scope::State);
        assert_eq!(manager.current_scope(), Scope::State);
        assert_eq!(manager.depth(), 2);

        manager.push_scope(Scope::UnitLeader);
        assert_eq!(manager.current_scope(), Scope::UnitLeader);
        assert_eq!(manager.depth(), 3);

        assert_eq!(manager.pop_scope(), Some(Scope::UnitLeader));
        assert_eq!(manager.current_scope(), Scope::State);

        assert_eq!(manager.pop_scope(), Some(Scope::State));
        assert_eq!(manager.current_scope(), Scope::Country);

        // 不能弹出最后一个作用域
        assert_eq!(manager.pop_scope(), None);
        assert_eq!(manager.current_scope(), Scope::Country);
    }

    #[test]
    fn test_can_transition() {
        let manager = ScopeManager::new();

        // 相同作用域可以转换
        assert!(manager.can_transition(Scope::Country, Scope::Country));

        // Any 作用域可以转换到任何作用域
        assert!(manager.can_transition(Scope::Any, Scope::State));
        assert!(manager.can_transition(Scope::Country, Scope::Any));

        // 检查预定义的转换规则
        assert!(manager.can_transition(Scope::Country, Scope::State));
        assert!(manager.can_transition(Scope::State, Scope::Country));
    }

    #[test]
    fn test_apply_transition() {
        let mut manager = ScopeManager::new();
        assert_eq!(manager.current_scope(), Scope::Country);

        // 应用有效的转换
        let result = manager.apply_transition("random_owned_controlled_state");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Scope::State);
        assert_eq!(manager.current_scope(), Scope::State);

        // 应用返回国家的转换
        let result = manager.apply_transition("owner");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Scope::Country);
        assert_eq!(manager.current_scope(), Scope::Country);

        // 应用未知命令
        let result = manager.apply_transition("unknown_command");
        assert!(result.is_err());
        match result {
            Err(ScopeError::UnknownCommand(cmd)) => {
                assert_eq!(cmd, "unknown_command");
            }
            _ => panic!("Expected UnknownCommand error"),
        }
    }

    #[test]
    fn test_validate_scope() {
        let mut manager = ScopeManager::new();

        // 当前是 Country 作用域
        assert!(manager.validate_scope(&[Scope::Country]).is_ok());
        assert!(manager
            .validate_scope(&[Scope::Country, Scope::State])
            .is_ok());
        assert!(manager.validate_scope(&[Scope::Any]).is_ok());

        // 不匹配的作用域
        let result = manager.validate_scope(&[Scope::State, Scope::UnitLeader]);
        assert!(result.is_err());
        match result {
            Err(ScopeError::ScopeMismatch { expected, actual }) => {
                assert_eq!(expected, vec![Scope::State, Scope::UnitLeader]);
                assert_eq!(actual, Scope::Country);
            }
            _ => panic!("Expected ScopeMismatch error"),
        }

        // 切换到 State 作用域
        manager.push_scope(Scope::State);
        assert!(manager.validate_scope(&[Scope::State]).is_ok());
    }

    #[test]
    fn test_reset() {
        let mut manager = ScopeManager::new();
        manager.push_scope(Scope::State);
        manager.push_scope(Scope::UnitLeader);
        assert_eq!(manager.depth(), 3);

        manager.reset();
        assert_eq!(manager.depth(), 1);
        assert_eq!(manager.current_scope(), Scope::Country);
    }

    #[test]
    fn test_reset_to() {
        let mut manager = ScopeManager::new();
        manager.push_scope(Scope::State);

        manager.reset_to(Scope::Air);
        assert_eq!(manager.depth(), 1);
        assert_eq!(manager.current_scope(), Scope::Air);
    }

    #[test]
    fn test_scope_transition() {
        let transition = ScopeTransition::new(
            vec![Scope::Country, Scope::State],
            Scope::UnitLeader,
            "test_command",
        );

        assert!(transition.can_transition_from(Scope::Country));
        assert!(transition.can_transition_from(Scope::State));
        assert!(!transition.can_transition_from(Scope::Air));
        assert!(transition.can_transition_from(Scope::Any));
    }

    #[test]
    fn test_add_custom_transition() {
        let mut manager = ScopeManager::new();

        // 添加自定义转换规则
        manager.add_transition(ScopeTransition::new(
            vec![Scope::Country],
            Scope::Air,
            "custom_air_command",
        ));

        // 验证自定义转换
        let result = manager.apply_transition("custom_air_command");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Scope::Air);
        assert_eq!(manager.current_scope(), Scope::Air);
    }
}
