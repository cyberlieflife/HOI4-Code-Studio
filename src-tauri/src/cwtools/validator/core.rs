//! 验证器核心模块
//!
//! 实现基于规则的 AST 验证功能

use crate::cwtools::diagnostic::{Diagnostic, DiagnosticManager, Severity};
use crate::cwtools::models::{KeyValue, Position, Statement, Value, AST};
use crate::cwtools::rules::{FieldType, Rule, RuleOptions, RuleSet, RuleType, ValueType};
use crate::cwtools::validator::reference::ReferenceChecker;
use crate::cwtools::validator::scope::{Scope, ScopeManager};
use std::collections::HashMap;

/// 验证上下文
///
/// 保存验证过程中的上下文信息
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// 当前作用域
    pub current_scope: Scope,
    /// 当前类型名称
    pub current_type: Option<String>,
    /// 父键路径（用于嵌套验证）
    pub parent_keys: Vec<String>,
}

impl ValidationContext {
    /// 创建新的验证上下文
    pub fn new(scope: Scope) -> Self {
        Self {
            current_scope: scope,
            current_type: None,
            parent_keys: Vec::new(),
        }
    }

    /// 进入子上下文
    pub fn enter_child(&self, key: String, scope: Option<Scope>) -> Self {
        let mut parent_keys = self.parent_keys.clone();
        parent_keys.push(key);

        Self {
            current_scope: scope.unwrap_or(self.current_scope),
            current_type: self.current_type.clone(),
            parent_keys,
        }
    }
}

/// 验证结果
#[derive(Debug)]
pub struct ValidationResult {
    /// 验证是否成功（无错误）
    pub success: bool,
    /// 诊断信息列表
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationResult {
    /// 创建成功的验证结果
    pub fn success() -> Self {
        Self {
            success: true,
            diagnostics: Vec::new(),
        }
    }

    /// 创建失败的验证结果
    pub fn failure(diagnostics: Vec<Diagnostic>) -> Self {
        let has_errors = diagnostics.iter().any(|d| d.severity == Severity::Error);
        Self {
            success: !has_errors,
            diagnostics,
        }
    }
}

/// 验证器
///
/// 基于规则验证 AST 的正确性
pub struct Validator {
    /// 规则引擎
    rule_set: RuleSet,
    /// 作用域管理器
    scope_manager: ScopeManager,
    /// 引用检查器
    reference_checker: ReferenceChecker,
    /// 诊断管理器
    diagnostic_manager: DiagnosticManager,
    /// 字段计数器（用于检查字段出现次数）
    field_counts: HashMap<String, usize>,
}

impl Validator {
    /// 创建新的验证器
    ///
    /// # 参数
    /// * `rule_set` - 规则集合
    pub fn new(rule_set: RuleSet) -> Self {
        Self {
            rule_set,
            scope_manager: ScopeManager::new(),
            reference_checker: ReferenceChecker::new(),
            diagnostic_manager: DiagnosticManager::new(),
            field_counts: HashMap::new(),
        }
    }

    /// 创建带有引用检查器的验证器
    pub fn with_reference_checker(rule_set: RuleSet, reference_checker: ReferenceChecker) -> Self {
        Self {
            rule_set,
            scope_manager: ScopeManager::new(),
            reference_checker,
            diagnostic_manager: DiagnosticManager::new(),
            field_counts: HashMap::new(),
        }
    }

    /// 验证 AST
    ///
    /// # 参数
    /// * `ast` - 要验证的抽象语法树
    ///
    /// # 返回
    /// 验证结果，包含所有诊断信息
    pub fn validate(&mut self, ast: &AST) -> ValidationResult {
        // 清空之前的诊断信息
        self.diagnostic_manager.clear();
        self.field_counts.clear();

        // 重置作用域管理器
        self.scope_manager.reset();

        // 创建初始验证上下文
        let context = ValidationContext::new(self.scope_manager.current_scope());

        // 验证所有顶层语句
        for statement in &ast.statements {
            self.validate_statement(statement, &context);
        }

        // 返回验证结果
        let diagnostics = self.diagnostic_manager.get_all().to_vec();
        ValidationResult::failure(diagnostics)
    }

    /// 验证单个语句
    ///
    /// # 参数
    /// * `statement` - 要验证的语句
    /// * `context` - 验证上下文
    pub fn validate_statement(&mut self, statement: &Statement, context: &ValidationContext) {
        match statement {
            Statement::KeyValue(kv) => {
                self.validate_key_value(kv, context);
            }
            Statement::ValueOnly(value, position) => {
                // 验证单独的值（通常在列表中）
                self.validate_value_only(value, *position, context);
            }
            Statement::Comment(_, _) => {
                // 注释不需要验证
            }
        }
    }

    /// 验证键值对
    ///
    /// # 参数
    /// * `kv` - 键值对
    /// * `context` - 验证上下文
    pub fn validate_key_value(&mut self, kv: &KeyValue, context: &ValidationContext) {
        let key = &kv.key;

        // 记录字段出现次数
        let field_key = format!("{}:{}", context.parent_keys.join("/"), key);
        *self.field_counts.entry(field_key.clone()).or_insert(0) += 1;

        // 查找适用的规则
        if let Some(rules) = self.find_applicable_rules(key, context) {
            for rule in rules {
                self.validate_with_rule(kv, &rule, context);
            }
        } else {
            // 未找到规则，报告未知键错误
            self.add_diagnostic(
                "V001".to_string(),
                Severity::Warning,
                format!("未知的键: '{}'", key),
                kv.position,
            );
        }
    }

    /// 验证单独的值
    fn validate_value_only(
        &mut self,
        value: &Value,
        position: Position,
        context: &ValidationContext,
    ) {
        // 对于单独的值，尝试根据上下文类型验证
        if let Some(type_name) = &context.current_type {
            // 先获取规则的克隆，避免借用冲突
            let rules = self
                .rule_set
                .get_type(type_name)
                .map(|type_def| type_def.rules.clone());

            if let Some(rules) = rules {
                // 查找 LeafValueRule
                for rule in &rules {
                    if let RuleType::LeafValueRule { right } = &rule.rule_type {
                        self.validate_value(value, right, position, context);
                        return;
                    }
                }
            }
        }
    }

    /// 使用规则验证键值对
    fn validate_with_rule(&mut self, kv: &KeyValue, rule: &Rule, context: &ValidationContext) {
        match &rule.rule_type {
            RuleType::NodeRule { left: _, children } => {
                // 节点规则：值应该是子句
                if let Value::Clause(statements) = &kv.value {
                    // 检查作用域
                    if let Some(push_scope) = rule.options.push_scope {
                        self.scope_manager.push_scope(push_scope);
                    }

                    // 创建子上下文
                    let child_context =
                        context.enter_child(kv.key.clone(), rule.options.push_scope);

                    // 验证子句内容
                    self.validate_clause(statements, children, &child_context);

                    // 恢复作用域
                    if rule.options.push_scope.is_some() {
                        self.scope_manager.pop_scope();
                    }
                } else {
                    self.add_diagnostic(
                        "V002".to_string(),
                        Severity::Error,
                        format!("键 '{}' 的值应该是子句（花括号包围）", kv.key),
                        kv.position,
                    );
                }
            }
            RuleType::LeafRule { left: _, right } => {
                // 叶子规则：值应该是简单类型
                if kv.value.is_clause() {
                    self.add_diagnostic(
                        "V003".to_string(),
                        Severity::Error,
                        format!("键 '{}' 的值应该是简单值，而不是子句", kv.key),
                        kv.position,
                    );
                } else {
                    self.validate_value(&kv.value, right, kv.position, context);
                }
            }
            RuleType::LeafValueRule { .. } => {
                // LeafValueRule 不应该用于键值对
            }
            RuleType::ValueClauseRule { .. } => {
                // ValueClauseRule 不应该用于键值对
            }
        }
    }

    /// 验证值
    ///
    /// # 参数
    /// * `value` - 要验证的值
    /// * `field_type` - 期望的字段类型
    /// * `position` - 值的位置
    /// * `context` - 验证上下文
    pub fn validate_value(
        &mut self,
        value: &Value,
        field_type: &FieldType,
        position: Position,
        context: &ValidationContext,
    ) {
        match field_type {
            FieldType::Value(value_type) => {
                self.validate_value_type(value, value_type, position);
            }
            FieldType::Specific(expected) => {
                if let Some(actual) = value.as_string() {
                    if actual != expected {
                        self.add_diagnostic(
                            "V004".to_string(),
                            Severity::Error,
                            format!("期望值为 '{}', 实际为 '{}'", expected, actual),
                            position,
                        );
                    }
                } else {
                    self.add_diagnostic(
                        "V005".to_string(),
                        Severity::Error,
                        format!("期望字符串值 '{}'", expected),
                        position,
                    );
                }
            }
            FieldType::Scalar => {
                // 标量类型接受任何简单值
                if value.is_clause() {
                    self.add_diagnostic(
                        "V006".to_string(),
                        Severity::Error,
                        "期望标量值，而不是子句".to_string(),
                        position,
                    );
                }
            }
            FieldType::Type(type_name) => {
                // 引用其他类型定义
                let type_name_clone = type_name.clone();
                if self.rule_set.get_type(&type_name_clone).is_some() {
                    if let Value::Clause(statements) = value {
                        let child_context = ValidationContext {
                            current_scope: context.current_scope,
                            current_type: Some(type_name_clone),
                            parent_keys: context.parent_keys.clone(),
                        };
                        for statement in statements {
                            self.validate_statement(statement, &child_context);
                        }
                    }
                } else {
                    self.add_diagnostic(
                        "V007".to_string(),
                        Severity::Warning,
                        format!("未找到类型定义: '{}'", type_name),
                        position,
                    );
                }
            }
            FieldType::Scope(scopes) => {
                // 验证作用域
                if let Err(err) = self.scope_manager.validate_scope(scopes) {
                    self.add_diagnostic(
                        "V008".to_string(),
                        Severity::Error,
                        format!("作用域错误: {}", err),
                        position,
                    );
                }
            }
            FieldType::Localisation { .. } => {
                // 验证本地化键
                if let Some(key) = value.as_string() {
                    if !self.reference_checker.check_localisation(key) {
                        self.add_diagnostic(
                            "V009".to_string(),
                            Severity::Warning,
                            format!("未找到本地化键: '{}'", key),
                            position,
                        );
                    }
                }
            }
            FieldType::Filepath { prefix, extension } => {
                // 验证文件路径
                if let Some(path_str) = value.as_string() {
                    let mut full_path = String::new();
                    if let Some(ref p) = prefix {
                        full_path.push_str(p);
                        full_path.push('/');
                    }
                    full_path.push_str(path_str);
                    if let Some(ref ext) = extension {
                        if !path_str.ends_with(ext.as_str()) {
                            full_path.push('.');
                            full_path.push_str(ext);
                        }
                    }

                    let path = std::path::Path::new(&full_path);
                    if !self.reference_checker.check_file_path(path) {
                        self.add_diagnostic(
                            "V010".to_string(),
                            Severity::Warning,
                            format!("未找到文件: '{}'", full_path),
                            position,
                        );
                    }
                }
            }
            FieldType::Enum(enum_name) => {
                // 验证枚举值
                if let Some(enum_def) = self.rule_set.get_enum(enum_name) {
                    if let Some(value_str) = value.as_string() {
                        if !enum_def.contains(value_str) {
                            self.add_diagnostic(
                                "V011".to_string(),
                                Severity::Error,
                                format!(
                                    "无效的枚举值 '{}', 期望: {:?}",
                                    value_str, enum_def.values
                                ),
                                position,
                            );
                        }
                    }
                } else {
                    self.add_diagnostic(
                        "V012".to_string(),
                        Severity::Warning,
                        format!("未找到枚举定义: '{}'", enum_name),
                        position,
                    );
                }
            }
            FieldType::Alias(alias_name) => {
                // 验证别名
                if let Some(alias) = self.rule_set.get_alias(alias_name) {
                    // 递归验证别名规则
                    self.validate_value(
                        value,
                        &self.get_field_type_from_rule(&alias.rule),
                        position,
                        context,
                    );
                }
            }
            FieldType::Variable { is_int, min, max } => {
                // 验证变量
                if *is_int {
                    if let Some(int_val) = value.as_integer() {
                        let int_val_f64 = int_val as f64;
                        if int_val_f64 < *min || int_val_f64 > *max {
                            self.add_diagnostic(
                                "V013".to_string(),
                                Severity::Error,
                                format!("整数值 {} 超出范围 [{}, {}]", int_val, min, max),
                                position,
                            );
                        }
                    }
                } else if let Some(float_val) = value.as_float() {
                    if float_val < *min || float_val > *max {
                        self.add_diagnostic(
                            "V014".to_string(),
                            Severity::Error,
                            format!("浮点数值 {} 超出范围 [{}, {}]", float_val, min, max),
                            position,
                        );
                    }
                }
            }
        }
    }

    /// 验证值类型
    fn validate_value_type(&mut self, value: &Value, value_type: &ValueType, position: Position) {
        match value_type {
            ValueType::Int { min, max } => {
                if let Some(int_val) = value.as_integer() {
                    if int_val < *min || int_val > *max {
                        self.add_diagnostic(
                            "V015".to_string(),
                            Severity::Error,
                            format!("整数值 {} 超出范围 [{}, {}]", int_val, min, max),
                            position,
                        );
                    }
                } else {
                    self.add_diagnostic(
                        "V016".to_string(),
                        Severity::Error,
                        "期望整数值".to_string(),
                        position,
                    );
                }
            }
            ValueType::Float { min, max } => match value {
                Value::Float(f) => {
                    if f < min || f > max {
                        self.add_diagnostic(
                            "V017".to_string(),
                            Severity::Error,
                            format!("浮点数值 {} 超出范围 [{}, {}]", f, min, max),
                            position,
                        );
                    }
                }
                Value::Integer(i) => {
                    let f = *i as f64;
                    if f < *min || f > *max {
                        self.add_diagnostic(
                            "V018".to_string(),
                            Severity::Error,
                            format!("数值 {} 超出范围 [{}, {}]", f, min, max),
                            position,
                        );
                    }
                }
                _ => {
                    self.add_diagnostic(
                        "V019".to_string(),
                        Severity::Error,
                        "期望浮点数值".to_string(),
                        position,
                    );
                }
            },
            ValueType::Boolean => {
                if value.as_boolean().is_none() {
                    self.add_diagnostic(
                        "V020".to_string(),
                        Severity::Error,
                        "期望布尔值 (yes/no)".to_string(),
                        position,
                    );
                }
            }
            ValueType::Percent => {
                // 百分比可以是浮点数或整数
                if value.as_float().is_none() && value.as_integer().is_none() {
                    self.add_diagnostic(
                        "V021".to_string(),
                        Severity::Error,
                        "期望百分比值（数值）".to_string(),
                        position,
                    );
                }
            }
            ValueType::Date => {
                // 日期格式验证（简化版）
                if let Some(date_str) = value.as_string() {
                    if !self.is_valid_date(date_str) {
                        self.add_diagnostic(
                            "V022".to_string(),
                            Severity::Error,
                            format!("无效的日期格式: '{}'", date_str),
                            position,
                        );
                    }
                } else {
                    self.add_diagnostic(
                        "V023".to_string(),
                        Severity::Error,
                        "期望日期值".to_string(),
                        position,
                    );
                }
            }
        }
    }

    /// 验证子句
    ///
    /// # 参数
    /// * `statements` - 子句中的语句列表
    /// * `rules` - 适用的规则列表
    /// * `context` - 验证上下文
    pub fn validate_clause(
        &mut self,
        statements: &[Statement],
        rules: &[Rule],
        context: &ValidationContext,
    ) {
        // 验证每个语句
        for statement in statements {
            self.validate_statement(statement, context);
        }

        // 检查必需字段
        self.check_required_fields(statements, rules, context);
    }

    /// 查找适用的规则
    fn find_applicable_rules(&self, key: &str, context: &ValidationContext) -> Option<Vec<Rule>> {
        // 如果有当前类型，从类型定义中查找规则
        if let Some(type_name) = &context.current_type {
            if let Some(type_def) = self.rule_set.get_type(type_name) {
                let mut applicable_rules = Vec::new();
                for rule in &type_def.rules {
                    if self.rule_matches_key(rule, key) {
                        applicable_rules.push(rule.clone());
                    }
                }
                if !applicable_rules.is_empty() {
                    return Some(applicable_rules);
                }
            }
        }

        // 否则尝试从全局类型中查找
        for type_def in self.rule_set.types.values() {
            let mut applicable_rules = Vec::new();
            for rule in &type_def.rules {
                if self.rule_matches_key(rule, key) {
                    applicable_rules.push(rule.clone());
                }
            }
            if !applicable_rules.is_empty() {
                return Some(applicable_rules);
            }
        }

        None
    }

    /// 检查规则是否匹配键
    fn rule_matches_key(&self, rule: &Rule, key: &str) -> bool {
        match &rule.rule_type {
            RuleType::NodeRule { left, .. } | RuleType::LeafRule { left, .. } => {
                self.field_type_matches_key(left, key)
            }
            _ => false,
        }
    }

    /// 检查字段类型是否匹配键
    fn field_type_matches_key(&self, field_type: &FieldType, key: &str) -> bool {
        match field_type {
            FieldType::Specific(expected) => expected == key,
            FieldType::Scalar => true,
            _ => false,
        }
    }

    /// 从规则中提取字段类型
    fn get_field_type_from_rule(&self, rule: &Rule) -> FieldType {
        match &rule.rule_type {
            RuleType::LeafRule { right, .. } => right.clone(),
            RuleType::LeafValueRule { right } => right.clone(),
            _ => FieldType::Scalar,
        }
    }

    /// 验证日期格式
    fn is_valid_date(&self, date_str: &str) -> bool {
        // 简化的日期验证：YYYY.MM.DD 或 YYYY.M.D
        let parts: Vec<&str> = date_str.split('.').collect();
        if parts.len() != 3 {
            return false;
        }

        // 验证年份
        if parts[0].parse::<i32>().is_err() {
            return false;
        }

        // 验证月份
        if let Ok(month) = parts[1].parse::<u32>() {
            if !(1..=12).contains(&month) {
                return false;
            }
        } else {
            return false;
        }

        // 验证日期
        if let Ok(day) = parts[2].parse::<u32>() {
            if !(1..=31).contains(&day) {
                return false;
            }
        } else {
            return false;
        }

        true
    }

    /// 添加诊断信息
    fn add_diagnostic(
        &mut self,
        code: String,
        severity: Severity,
        message: String,
        position: Position,
    ) {
        use crate::cwtools::models::Range;

        let diagnostic = Diagnostic::new(
            code,
            severity,
            message,
            Range::point(position),
            "validator".to_string(),
        );
        self.diagnostic_manager.add(diagnostic);
    }

    /// 获取诊断管理器的引用
    pub fn diagnostics(&self) -> &DiagnosticManager {
        &self.diagnostic_manager
    }
}

// 字段验证逻辑实现
impl Validator {
    /// 检查必需字段
    ///
    /// 验证子句中是否包含所有必需的字段（min > 0）
    ///
    /// # 参数
    /// * `statements` - 子句中的语句列表
    /// * `rules` - 适用的规则列表
    /// * `context` - 验证上下文
    pub fn check_required_fields(
        &mut self,
        statements: &[Statement],
        rules: &[Rule],
        _context: &ValidationContext,
    ) {
        // 收集子句中出现的所有键
        let mut present_keys: HashMap<String, usize> = HashMap::new();
        for statement in statements {
            if let Statement::KeyValue(kv) = statement {
                *present_keys.entry(kv.key.clone()).or_insert(0) += 1;
            }
        }

        // 检查每个规则的必需字段
        for rule in rules {
            let (key_pattern, min_count) = match &rule.rule_type {
                RuleType::NodeRule { left, .. } | RuleType::LeafRule { left, .. } => {
                    if let FieldType::Specific(key) = left {
                        (Some(key.as_str()), rule.options.min)
                    } else {
                        (None, rule.options.min)
                    }
                }
                _ => (None, 0),
            };

            if let Some(key) = key_pattern {
                let count = present_keys.get(key).copied().unwrap_or(0);

                // 检查最小出现次数
                if count < min_count {
                    let position = if let Some(Statement::KeyValue(kv)) = statements.first() {
                        kv.position
                    } else {
                        Position::start()
                    };

                    self.add_diagnostic(
                        "V024".to_string(),
                        Severity::Error,
                        format!(
                            "缺少必需字段 '{}' (需要至少 {} 次，实际 {} 次)",
                            key, min_count, count
                        ),
                        position,
                    );
                }

                // 检查最大出现次数
                if let Some(max_count) = rule.options.max {
                    if count > max_count {
                        self.check_field_count(key, count, &rule.options, Position::start());
                    }
                }
            }
        }
    }

    /// 检查字段数量
    ///
    /// 验证字段出现次数是否在允许的范围内
    ///
    /// # 参数
    /// * `key` - 字段键名
    /// * `count` - 实际出现次数
    /// * `options` - 规则选项（包含 min 和 max）
    /// * `position` - 位置信息
    pub fn check_field_count(
        &mut self,
        key: &str,
        count: usize,
        options: &RuleOptions,
        position: Position,
    ) {
        // 检查最小出现次数
        if count < options.min {
            let severity = options.severity.unwrap_or(Severity::Error);
            self.add_diagnostic(
                "V025".to_string(),
                severity,
                format!(
                    "字段 '{}' 出现次数不足 (需要至少 {} 次，实际 {} 次)",
                    key, options.min, count
                ),
                position,
            );
        }

        // 检查最大出现次数
        if let Some(max) = options.max {
            if count > max {
                let severity = options.severity.unwrap_or(Severity::Error);
                self.add_diagnostic(
                    "V026".to_string(),
                    severity,
                    format!(
                        "字段 '{}' 出现次数过多 (最多允许 {} 次，实际 {} 次)",
                        key, max, count
                    ),
                    position,
                );
            }
        }
    }

    /// 验证类型匹配
    ///
    /// 检查值的类型是否与期望的字段类型匹配
    ///
    /// # 参数
    /// * `value` - 要验证的值
    /// * `field_type` - 期望的字段类型
    /// * `position` - 位置信息
    ///
    /// # 返回
    /// 如果类型匹配返回 true
    pub fn check_type_match(
        &mut self,
        value: &Value,
        field_type: &FieldType,
        position: Position,
    ) -> bool {
        match field_type {
            FieldType::Value(value_type) => {
                self.check_value_type_match(value, value_type, position)
            }
            FieldType::Specific(expected) => {
                if let Some(actual) = value.as_string() {
                    actual == expected
                } else {
                    self.add_diagnostic(
                        "V027".to_string(),
                        Severity::Error,
                        format!("类型不匹配: 期望字符串 '{}'", expected),
                        position,
                    );
                    false
                }
            }
            FieldType::Scalar => {
                // 标量接受任何非子句值
                !value.is_clause()
            }
            FieldType::Type(_) => {
                // 类型引用需要是子句
                value.is_clause()
            }
            _ => true, // 其他类型暂时接受
        }
    }

    /// 检查值类型匹配
    fn check_value_type_match(
        &mut self,
        value: &Value,
        value_type: &ValueType,
        position: Position,
    ) -> bool {
        match value_type {
            ValueType::Int { .. } => {
                if value.as_integer().is_none() {
                    self.add_diagnostic(
                        "V028".to_string(),
                        Severity::Error,
                        "类型不匹配: 期望整数".to_string(),
                        position,
                    );
                    false
                } else {
                    true
                }
            }
            ValueType::Float { .. } => {
                if value.as_float().is_none() && value.as_integer().is_none() {
                    self.add_diagnostic(
                        "V029".to_string(),
                        Severity::Error,
                        "类型不匹配: 期望浮点数".to_string(),
                        position,
                    );
                    false
                } else {
                    true
                }
            }
            ValueType::Boolean => {
                if value.as_boolean().is_none() {
                    self.add_diagnostic(
                        "V030".to_string(),
                        Severity::Error,
                        "类型不匹配: 期望布尔值".to_string(),
                        position,
                    );
                    false
                } else {
                    true
                }
            }
            ValueType::Percent => {
                if value.as_float().is_none() && value.as_integer().is_none() {
                    self.add_diagnostic(
                        "V031".to_string(),
                        Severity::Error,
                        "类型不匹配: 期望百分比（数值）".to_string(),
                        position,
                    );
                    false
                } else {
                    true
                }
            }
            ValueType::Date => {
                if value.as_string().is_none() {
                    self.add_diagnostic(
                        "V032".to_string(),
                        Severity::Error,
                        "类型不匹配: 期望日期字符串".to_string(),
                        position,
                    );
                    false
                } else {
                    true
                }
            }
        }
    }
}

// 数值和枚举验证实现
impl Validator {
    /// 检查数值范围
    ///
    /// 验证整数或浮点数值是否在指定范围内
    ///
    /// # 参数
    /// * `value` - 要验证的值
    /// * `min` - 最小值
    /// * `max` - 最大值
    /// * `position` - 位置信息
    ///
    /// # 返回
    /// 如果值在范围内返回 true
    pub fn check_number_range(
        &mut self,
        value: &Value,
        min: f64,
        max: f64,
        position: Position,
    ) -> bool {
        let num_value = match value {
            Value::Integer(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        };

        if let Some(num) = num_value {
            if num < min || num > max {
                self.add_diagnostic(
                    "V033".to_string(),
                    Severity::Error,
                    format!("数值 {} 超出范围 [{}, {}]", num, min, max),
                    position,
                );
                false
            } else {
                true
            }
        } else {
            self.add_diagnostic(
                "V034".to_string(),
                Severity::Error,
                "期望数值类型".to_string(),
                position,
            );
            false
        }
    }

    /// 检查整数范围
    ///
    /// 验证整数值是否在指定范围内
    ///
    /// # 参数
    /// * `value` - 要验证的值
    /// * `min` - 最小值
    /// * `max` - 最大值
    /// * `position` - 位置信息
    ///
    /// # 返回
    /// 如果值在范围内返回 true
    pub fn check_integer_range(
        &mut self,
        value: &Value,
        min: i64,
        max: i64,
        position: Position,
    ) -> bool {
        if let Some(int_val) = value.as_integer() {
            if int_val < min || int_val > max {
                self.add_diagnostic(
                    "V035".to_string(),
                    Severity::Error,
                    format!("整数值 {} 超出范围 [{}, {}]", int_val, min, max),
                    position,
                );
                false
            } else {
                true
            }
        } else {
            self.add_diagnostic(
                "V036".to_string(),
                Severity::Error,
                "期望整数类型".to_string(),
                position,
            );
            false
        }
    }

    /// 检查浮点数范围
    ///
    /// 验证浮点数值是否在指定范围内
    ///
    /// # 参数
    /// * `value` - 要验证的值
    /// * `min` - 最小值
    /// * `max` - 最大值
    /// * `position` - 位置信息
    ///
    /// # 返回
    /// 如果值在范围内返回 true
    pub fn check_float_range(
        &mut self,
        value: &Value,
        min: f64,
        max: f64,
        position: Position,
    ) -> bool {
        match value {
            Value::Float(f) => {
                if f < &min || f > &max {
                    self.add_diagnostic(
                        "V037".to_string(),
                        Severity::Error,
                        format!("浮点数值 {} 超出范围 [{}, {}]", f, min, max),
                        position,
                    );
                    false
                } else {
                    true
                }
            }
            Value::Integer(i) => {
                let f = *i as f64;
                if f < min || f > max {
                    self.add_diagnostic(
                        "V038".to_string(),
                        Severity::Error,
                        format!("数值 {} 超出范围 [{}, {}]", f, min, max),
                        position,
                    );
                    false
                } else {
                    true
                }
            }
            _ => {
                self.add_diagnostic(
                    "V039".to_string(),
                    Severity::Error,
                    "期望浮点数类型".to_string(),
                    position,
                );
                false
            }
        }
    }

    /// 验证枚举值
    ///
    /// 检查值是否在允许的枚举值列表中
    ///
    /// # 参数
    /// * `value` - 要验证的值
    /// * `enum_name` - 枚举名称
    /// * `position` - 位置信息
    ///
    /// # 返回
    /// 如果值是有效的枚举值返回 true
    pub fn check_enum_value(&mut self, value: &Value, enum_name: &str, position: Position) -> bool {
        if let Some(enum_def) = self.rule_set.get_enum(enum_name) {
            if let Some(value_str) = value.as_string() {
                if enum_def.contains(value_str) {
                    true
                } else {
                    self.add_diagnostic(
                        "V040".to_string(),
                        Severity::Error,
                        format!(
                            "无效的枚举值 '{}' (枚举: {}), 允许的值: {:?}",
                            value_str, enum_name, enum_def.values
                        ),
                        position,
                    );
                    false
                }
            } else {
                self.add_diagnostic(
                    "V041".to_string(),
                    Severity::Error,
                    format!("枚举 '{}' 期望字符串值", enum_name),
                    position,
                );
                false
            }
        } else {
            self.add_diagnostic(
                "V042".to_string(),
                Severity::Warning,
                format!("未找到枚举定义: '{}'", enum_name),
                position,
            );
            false
        }
    }
}

// 作用域验证实现
impl Validator {
    /// 检查作用域
    ///
    /// 验证当前作用域是否符合要求
    ///
    /// # 参数
    /// * `required_scopes` - 必需的作用域列表
    /// * `position` - 位置信息
    ///
    /// # 返回
    /// 如果作用域匹配返回 true
    pub fn check_scope(&mut self, required_scopes: &[Scope], position: Position) -> bool {
        if required_scopes.is_empty() {
            return true;
        }

        match self.scope_manager.validate_scope(required_scopes) {
            Ok(()) => true,
            Err(err) => {
                self.add_diagnostic(
                    "V043".to_string(),
                    Severity::Error,
                    format!("作用域验证失败: {}", err),
                    position,
                );
                false
            }
        }
    }

    /// 验证作用域转换
    ///
    /// 检查作用域转换命令是否有效
    ///
    /// # 参数
    /// * `command` - 作用域转换命令
    /// * `position` - 位置信息
    ///
    /// # 返回
    /// 如果转换成功返回新的作用域
    pub fn validate_scope_transition(
        &mut self,
        command: &str,
        position: Position,
    ) -> Option<Scope> {
        match self.scope_manager.apply_transition(command) {
            Ok(new_scope) => Some(new_scope),
            Err(err) => {
                self.add_diagnostic(
                    "V044".to_string(),
                    Severity::Error,
                    format!("作用域转换失败: {}", err),
                    position,
                );
                None
            }
        }
    }

    /// 获取当前作用域
    pub fn current_scope(&self) -> Scope {
        self.scope_manager.current_scope()
    }

    /// 推送新作用域
    pub fn push_scope(&mut self, scope: Scope) {
        self.scope_manager.push_scope(scope);
    }

    /// 弹出作用域
    pub fn pop_scope(&mut self) -> Option<Scope> {
        self.scope_manager.pop_scope()
    }

    /// 重置作用域到初始状态
    pub fn reset_scope(&mut self) {
        self.scope_manager.reset();
    }

    /// 检查作用域是否可以转换
    ///
    /// # 参数
    /// * `from` - 源作用域
    /// * `to` - 目标作用域
    ///
    /// # 返回
    /// 如果可以转换返回 true
    pub fn can_transition_scope(&self, from: Scope, to: Scope) -> bool {
        self.scope_manager.can_transition(from, to)
    }
}

// 修饰符验证实现
impl Validator {
    /// 加载修饰符定义
    ///
    /// 从规则集中加载修饰符定义
    pub fn load_modifiers(&mut self) {
        // 修饰符已经在 RuleSet 中加载
        // 这里可以进行额外的初始化工作
    }

    /// 验证修饰符
    ///
    /// 检查修饰符是否有效，包括类别和作用域匹配
    ///
    /// # 参数
    /// * `modifier_name` - 修饰符名称
    /// * `value` - 修饰符值
    /// * `position` - 位置信息
    ///
    /// # 返回
    /// 如果修饰符有效返回 true
    pub fn validate_modifier(
        &mut self,
        modifier_name: &str,
        value: &Value,
        position: Position,
    ) -> bool {
        // 查找修饰符定义并克隆必要的数据以避免借用冲突
        let modifier_info = self
            .rule_set
            .modifiers
            .iter()
            .find(|m| m.name == modifier_name)
            .map(|m| (m.scopes.clone(), m.value_type.clone()));

        if let Some((scopes, value_type)) = modifier_info {
            // 检查作用域匹配
            let current_scope = self.scope_manager.current_scope();
            if !scopes.iter().any(|s| s.matches(current_scope)) {
                self.add_diagnostic(
                    "V045".to_string(),
                    Severity::Error,
                    format!(
                        "修饰符 '{}' 不能在当前作用域 '{}' 中使用，允许的作用域: {:?}",
                        modifier_name,
                        current_scope.as_str(),
                        scopes.iter().map(|s| s.as_str()).collect::<Vec<_>>()
                    ),
                    position,
                );
                return false;
            }

            // 验证值类型
            self.validate_value_type(value, &value_type, position);

            true
        } else {
            // 未找到修饰符定义，发出警告
            self.add_diagnostic(
                "V046".to_string(),
                Severity::Warning,
                format!("未知的修饰符: '{}'", modifier_name),
                position,
            );
            false
        }
    }

    /// 检查修饰符类别
    ///
    /// 验证修饰符是否属于指定的类别
    ///
    /// # 参数
    /// * `modifier_name` - 修饰符名称
    /// * `expected_category` - 期望的类别
    /// * `position` - 位置信息
    ///
    /// # 返回
    /// 如果类别匹配返回 true
    pub fn check_modifier_category(
        &mut self,
        modifier_name: &str,
        expected_category: crate::cwtools::rules::ModifierCategory,
        position: Position,
    ) -> bool {
        let modifier_def = self
            .rule_set
            .modifiers
            .iter()
            .find(|m| m.name == modifier_name);

        if let Some(modifier) = modifier_def {
            if modifier.category != expected_category {
                self.add_diagnostic(
                    "V047".to_string(),
                    Severity::Error,
                    format!(
                        "修饰符 '{}' 的类别不匹配，期望 {:?}，实际 {:?}",
                        modifier_name, expected_category, modifier.category
                    ),
                    position,
                );
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    /// 检查修饰符作用域
    ///
    /// 验证修饰符是否可以在当前作用域中使用
    ///
    /// # 参数
    /// * `modifier_name` - 修饰符名称
    /// * `position` - 位置信息
    ///
    /// # 返回
    /// 如果作用域匹配返回 true
    pub fn check_modifier_scope(&mut self, modifier_name: &str, position: Position) -> bool {
        let modifier_def = self
            .rule_set
            .modifiers
            .iter()
            .find(|m| m.name == modifier_name);

        if let Some(modifier) = modifier_def {
            let current_scope = self.scope_manager.current_scope();
            let scope_matches = modifier.scopes.iter().any(|s| s.matches(current_scope));

            if !scope_matches {
                self.add_diagnostic(
                    "V048".to_string(),
                    Severity::Error,
                    format!(
                        "修饰符 '{}' 不能在作用域 '{}' 中使用",
                        modifier_name,
                        current_scope.as_str()
                    ),
                    position,
                );
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    /// 验证修饰符值范围
    ///
    /// 检查修饰符的值是否在合理范围内
    ///
    /// # 参数
    /// * `modifier_name` - 修饰符名称
    /// * `value` - 修饰符值
    /// * `position` - 位置信息
    pub fn validate_modifier_value_range(
        &mut self,
        modifier_name: &str,
        value: &Value,
        position: Position,
    ) {
        let modifier_def = self
            .rule_set
            .modifiers
            .iter()
            .find(|m| m.name == modifier_name);

        if let Some(modifier) = modifier_def {
            // 根据值类型验证范围
            match &modifier.value_type {
                ValueType::Int { min, max } => {
                    if let Some(int_val) = value.as_integer() {
                        if int_val < *min || int_val > *max {
                            self.add_diagnostic(
                                "V049".to_string(),
                                Severity::Warning,
                                format!(
                                    "修饰符 '{}' 的值 {} 可能超出合理范围 [{}, {}]",
                                    modifier_name, int_val, min, max
                                ),
                                position,
                            );
                        }
                    }
                }
                ValueType::Float { min, max } => {
                    let num_val = match value {
                        Value::Float(f) => Some(*f),
                        Value::Integer(i) => Some(*i as f64),
                        _ => None,
                    };

                    if let Some(num) = num_val {
                        if num < *min || num > *max {
                            self.add_diagnostic(
                                "V050".to_string(),
                                Severity::Warning,
                                format!(
                                    "修饰符 '{}' 的值 {} 可能超出合理范围 [{}, {}]",
                                    modifier_name, num, min, max
                                ),
                                position,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
