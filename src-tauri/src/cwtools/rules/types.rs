//! 规则引擎数据类型定义
//!
//! 定义 CWT 规则文件的数据结构，包括类型定义、规则、字段类型等

use crate::cwtools::diagnostic::Severity;
use crate::cwtools::validator::scope::Scope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 规则集合
///
/// 包含从 CWT 文件加载的所有规则定义
#[derive(Debug, Clone, Default)]
pub struct RuleSet {
    /// 类型定义映射，键为类型名称
    pub types: HashMap<String, TypeDefinition>,
    /// 枚举定义映射，键为枚举名称
    pub enums: HashMap<String, EnumDefinition>,
    /// 别名规则映射，键为别名名称
    pub aliases: HashMap<String, AliasRule>,
    /// 修饰符定义列表
    pub modifiers: Vec<ModifierDefinition>,
}

impl RuleSet {
    /// 创建新的空规则集
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加类型定义
    pub fn add_type(&mut self, name: String, type_def: TypeDefinition) {
        self.types.insert(name, type_def);
    }

    /// 添加枚举定义
    pub fn add_enum(&mut self, name: String, enum_def: EnumDefinition) {
        self.enums.insert(name, enum_def);
    }

    /// 添加别名规则
    pub fn add_alias(&mut self, name: String, alias: AliasRule) {
        self.aliases.insert(name, alias);
    }

    /// 添加修饰符定义
    pub fn add_modifier(&mut self, modifier: ModifierDefinition) {
        self.modifiers.push(modifier);
    }

    /// 获取类型定义
    pub fn get_type(&self, name: &str) -> Option<&TypeDefinition> {
        self.types.get(name)
    }

    /// 获取枚举定义
    pub fn get_enum(&self, name: &str) -> Option<&EnumDefinition> {
        self.enums.get(name)
    }

    /// 获取别名规则
    pub fn get_alias(&self, name: &str) -> Option<&AliasRule> {
        self.aliases.get(name)
    }

    /// 合并另一个规则集
    ///
    /// 将另一个规则集的内容合并到当前规则集中
    /// 如果有重复的键，后加载的规则会覆盖先加载的规则
    ///
    /// # 参数
    /// * `other` - 要合并的规则集
    pub fn merge(&mut self, other: RuleSet) {
        // 合并类型定义
        for (name, type_def) in other.types {
            self.types.insert(name, type_def);
        }

        // 合并枚举定义
        for (name, enum_def) in other.enums {
            self.enums.insert(name, enum_def);
        }

        // 合并别名规则
        for (name, alias) in other.aliases {
            self.aliases.insert(name, alias);
        }

        // 合并修饰符定义
        self.modifiers.extend(other.modifiers);
    }
}

/// 类型定义
///
/// 定义特定脚本类型的结构和验证规则
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// 类型名称
    pub name: String,
    /// 规则列表
    pub rules: Vec<Rule>,
    /// 子类型定义列表
    pub subtypes: Vec<SubTypeDefinition>,
    /// 类型选项
    pub options: TypeOptions,
}

impl TypeDefinition {
    /// 创建新的类型定义
    pub fn new(name: String) -> Self {
        Self {
            name,
            rules: Vec::new(),
            subtypes: Vec::new(),
            options: TypeOptions::default(),
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// 添加子类型
    pub fn add_subtype(&mut self, subtype: SubTypeDefinition) {
        self.subtypes.push(subtype);
    }
}

/// 子类型定义
///
/// 定义类型的子类型变体
#[derive(Debug, Clone)]
pub struct SubTypeDefinition {
    /// 子类型名称
    pub name: String,
    /// 子类型规则
    pub rules: Vec<Rule>,
}

impl SubTypeDefinition {
    /// 创建新的子类型定义
    pub fn new(name: String) -> Self {
        Self {
            name,
            rules: Vec::new(),
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }
}

/// 类型选项
///
/// 类型定义的配置选项
#[derive(Debug, Clone, Default)]
pub struct TypeOptions {
    /// 是否跳过根键检查
    pub skip_root_key: bool,
    /// 路径前缀
    pub path_prefix: Option<String>,
    /// 路径扩展名
    pub path_extension: Option<String>,
}

/// 规则
///
/// 定义验证规则的结构
#[derive(Debug, Clone)]
pub struct Rule {
    /// 规则类型
    pub rule_type: RuleType,
    /// 规则选项
    pub options: RuleOptions,
}

impl Rule {
    /// 创建新的规则
    pub fn new(rule_type: RuleType, options: RuleOptions) -> Self {
        Self { rule_type, options }
    }

    /// 创建简单的节点规则
    pub fn node_rule(left: FieldType, children: Vec<Rule>) -> Self {
        Self {
            rule_type: RuleType::NodeRule { left, children },
            options: RuleOptions::default(),
        }
    }

    /// 创建简单的叶子规则
    pub fn leaf_rule(left: FieldType, right: FieldType) -> Self {
        Self {
            rule_type: RuleType::LeafRule { left, right },
            options: RuleOptions::default(),
        }
    }
}

/// 规则类型
///
/// 定义不同类型的验证规则
#[derive(Debug, Clone)]
pub enum RuleType {
    /// 节点规则：键值对，值为子句
    ///
    /// 例如：`country = { ... }`
    NodeRule {
        /// 左侧字段类型（键）
        left: FieldType,
        /// 子规则列表（子句内容）
        children: Vec<Rule>,
    },

    /// 叶子规则：键值对，值为简单类型
    ///
    /// 例如：`tag = GER`
    LeafRule {
        /// 左侧字段类型（键）
        left: FieldType,
        /// 右侧字段类型（值）
        right: FieldType,
    },

    /// 叶子值规则：仅包含值，无键
    ///
    /// 例如：列表中的单个值
    LeafValueRule {
        /// 右侧字段类型（值）
        right: FieldType,
    },

    /// 值子句规则：值为子句，无键
    ///
    /// 例如：`{ ... }` 作为列表项
    ValueClauseRule {
        /// 子规则列表（子句内容）
        children: Vec<Rule>,
    },
}

/// 字段类型
///
/// 定义字段可以接受的值类型
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// 值类型（整数、浮点数、布尔值等）
    Value(ValueType),

    /// 特定字符串值
    ///
    /// 例如：必须是 "yes" 或 "no"
    Specific(String),

    /// 标量类型（任意字符串或数值）
    Scalar,

    /// 引用其他类型定义
    Type(String),

    /// 作用域类型
    Scope(Vec<Scope>),

    /// 本地化键
    Localisation {
        /// 是否同步本地化
        synced: bool,
        /// 是否内联本地化
        inline: bool,
    },

    /// 文件路径
    Filepath {
        /// 路径前缀
        prefix: Option<String>,
        /// 文件扩展名
        extension: Option<String>,
    },

    /// 枚举类型
    Enum(String),

    /// 别名类型
    Alias(String),

    /// 变量类型
    Variable {
        /// 是否为整数变量
        is_int: bool,
        /// 最小值
        min: f64,
        /// 最大值
        max: f64,
    },
}

/// 值类型
///
/// 定义基本的值类型及其约束
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    /// 整数类型
    Int {
        /// 最小值
        min: i64,
        /// 最大值
        max: i64,
    },

    /// 浮点数类型
    Float {
        /// 最小值
        min: f64,
        /// 最大值
        max: f64,
    },

    /// 布尔类型
    Boolean,

    /// 百分比类型
    Percent,

    /// 日期类型
    Date,
}

impl ValueType {
    /// 创建无限制的整数类型
    pub fn int() -> Self {
        Self::Int {
            min: i64::MIN,
            max: i64::MAX,
        }
    }

    /// 创建有范围限制的整数类型
    pub fn int_range(min: i64, max: i64) -> Self {
        Self::Int { min, max }
    }

    /// 创建无限制的浮点数类型
    pub fn float() -> Self {
        Self::Float {
            min: f64::MIN,
            max: f64::MAX,
        }
    }

    /// 创建有范围限制的浮点数类型
    pub fn float_range(min: f64, max: f64) -> Self {
        Self::Float { min, max }
    }
}

/// 规则选项
///
/// 规则的配置选项，控制验证行为
#[derive(Debug, Clone)]
pub struct RuleOptions {
    /// 最小出现次数
    pub min: usize,
    /// 最大出现次数（None 表示无限制）
    pub max: Option<usize>,
    /// 必需的作用域列表
    pub required_scopes: Vec<Scope>,
    /// 进入子句时切换到的作用域
    pub push_scope: Option<Scope>,
    /// 错误严重程度
    pub severity: Option<Severity>,
    /// 规则描述
    pub description: Option<String>,
    /// 是否仅警告
    pub warning_only: bool,
}

impl Default for RuleOptions {
    fn default() -> Self {
        Self {
            min: 0,
            max: None,
            required_scopes: Vec::new(),
            push_scope: None,
            severity: None,
            description: None,
            warning_only: false,
        }
    }
}

impl RuleOptions {
    /// 创建新的规则选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最小出现次数
    pub fn with_min(mut self, min: usize) -> Self {
        self.min = min;
        self
    }

    /// 设置最大出现次数
    pub fn with_max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    /// 设置必需的作用域
    pub fn with_required_scopes(mut self, scopes: Vec<Scope>) -> Self {
        self.required_scopes = scopes;
        self
    }

    /// 设置推送作用域
    pub fn with_push_scope(mut self, scope: Scope) -> Self {
        self.push_scope = Some(scope);
        self
    }

    /// 设置严重程度
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置仅警告标志
    pub fn with_warning_only(mut self, warning_only: bool) -> Self {
        self.warning_only = warning_only;
        self
    }
}

/// 枚举定义
///
/// 定义允许的枚举值列表
#[derive(Debug, Clone)]
pub struct EnumDefinition {
    /// 枚举键（名称）
    pub key: String,
    /// 枚举描述
    pub description: String,
    /// 允许的值列表
    pub values: Vec<String>,
}

impl EnumDefinition {
    /// 创建新的枚举定义
    pub fn new(key: String, description: String) -> Self {
        Self {
            key,
            description,
            values: Vec::new(),
        }
    }

    /// 添加枚举值
    pub fn add_value(&mut self, value: String) {
        self.values.push(value);
    }

    /// 检查值是否在枚举中
    pub fn contains(&self, value: &str) -> bool {
        self.values.iter().any(|v| v == value)
    }
}

/// 别名规则
///
/// 定义可复用的规则别名
#[derive(Debug, Clone)]
pub struct AliasRule {
    /// 别名名称
    pub name: String,
    /// 别名对应的规则
    pub rule: Rule,
}

impl AliasRule {
    /// 创建新的别名规则
    pub fn new(name: String, rule: Rule) -> Self {
        Self { name, rule }
    }
}

/// 修饰符定义
///
/// 定义游戏修饰符的属性
#[derive(Debug, Clone)]
pub struct ModifierDefinition {
    /// 修饰符名称
    pub name: String,
    /// 修饰符类别
    pub category: ModifierCategory,
    /// 允许的作用域列表
    pub scopes: Vec<Scope>,
    /// 值类型
    pub value_type: ValueType,
}

impl ModifierDefinition {
    /// 创建新的修饰符定义
    pub fn new(name: String, category: ModifierCategory, scopes: Vec<Scope>) -> Self {
        Self {
            name,
            category,
            scopes,
            value_type: ValueType::float(),
        }
    }

    /// 设置值类型
    pub fn with_value_type(mut self, value_type: ValueType) -> Self {
        self.value_type = value_type;
        self
    }
}

/// 修饰符类别
///
/// 定义修饰符的应用类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierCategory {
    /// 国家修饰符
    Country,
    /// 州修饰符
    State,
    /// 单位修饰符
    Unit,
    /// 单位领导者修饰符
    UnitLeader,
    /// 空军修饰符
    Air,
}
