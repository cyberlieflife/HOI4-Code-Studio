//! CWT 规则文件加载器
//!
//! 负责解析 .cwt 规则文件并构建规则集

use super::types::{
    AliasRule, EnumDefinition, FieldType, ModifierCategory, ModifierDefinition, Rule, RuleOptions,
    RuleSet, RuleType, SubTypeDefinition, TypeDefinition, ValueType,
};
use crate::cwtools::diagnostic::Severity;
use crate::cwtools::models::{Operator, Position, Value as AstValue};
use crate::cwtools::parser::Parser;
use crate::cwtools::validator::scope::Scope;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 规则加载错误
#[derive(Debug, Clone)]
pub struct RuleError {
    /// 错误消息
    pub message: String,
    /// 错误位置
    pub position: Option<Position>,
    /// 源文件路径
    pub file: Option<String>,
}

impl RuleError {
    /// 创建新的规则错误
    pub fn new(message: String) -> Self {
        Self {
            message,
            position: None,
            file: None,
        }
    }

    /// 创建带位置信息的规则错误
    pub fn with_position(message: String, position: Position) -> Self {
        Self {
            message,
            position: Some(position),
            file: None,
        }
    }

    /// 设置文件路径
    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(file) = &self.file {
            write!(f, "{}:", file)?;
        }
        if let Some(pos) = &self.position {
            write!(f, "{}:{}: ", pos.line, pos.column)?;
        }
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuleError {}

/// 规则加载器
///
/// 负责加载和解析 CWT 规则文件
pub struct RuleLoader {
    /// 规则文件路径列表
    rule_files: Vec<PathBuf>,
    /// 缓存的规则集
    cache: HashMap<String, RuleSet>,
}

impl RuleLoader {
    /// 创建新的规则加载器
    pub fn new() -> Self {
        Self {
            rule_files: Vec::new(),
            cache: HashMap::new(),
        }
    }

    /// 添加规则文件路径
    pub fn add_rule_file(&mut self, path: PathBuf) {
        self.rule_files.push(path);
    }

    /// 加载单个规则文件
    ///
    /// # 参数
    /// * `path` - 规则文件路径
    ///
    /// # 返回
    /// 解析后的规则集或错误列表
    pub fn load_rules(&mut self, path: &Path) -> Result<RuleSet, Vec<RuleError>> {
        // 读取文件内容
        let content = fs::read_to_string(path).map_err(|e| {
            vec![RuleError::new(format!("Failed to read file: {}", e))
                .with_file(path.display().to_string())]
        })?;

        // 解析文件
        let mut parser = Parser::new(&content, path.display().to_string()).map_err(|e| {
            vec![
                RuleError::new(format!("Failed to create parser: {}", e.message))
                    .with_file(path.display().to_string()),
            ]
        })?;
        let ast = parser.parse().map_err(|errors| {
            errors
                .into_iter()
                .map(|e| {
                    RuleError::new(format!("Parse error: {}", e.message))
                        .with_file(path.display().to_string())
                })
                .collect::<Vec<_>>()
        })?;

        // 构建规则集
        let mut rule_set = RuleSet::new();
        let mut errors = Vec::new();

        for statement in &ast.statements {
            if let Err(e) = self.process_statement(statement, &mut rule_set) {
                errors.push(e.with_file(path.display().to_string()));
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(rule_set)
    }

    /// 加载所有规则文件并合并
    ///
    /// # 参数
    /// * `paths` - 规则文件路径列表
    ///
    /// # 返回
    /// 合并后的规则集或错误列表
    pub fn load_all_rules(&mut self, paths: &[PathBuf]) -> Result<RuleSet, Vec<RuleError>> {
        let mut merged_rule_set = RuleSet::new();
        let mut all_errors = Vec::new();

        for path in paths {
            match self.load_rules(path) {
                Ok(rule_set) => {
                    self.merge_rule_sets(&mut merged_rule_set, rule_set);
                }
                Err(errors) => {
                    all_errors.extend(errors);
                }
            }
        }

        if !all_errors.is_empty() {
            return Err(all_errors);
        }

        Ok(merged_rule_set)
    }

    /// 重新加载所有规则文件
    pub fn reload(&mut self) -> Result<(), Vec<RuleError>> {
        self.cache.clear();
        let paths = self.rule_files.clone();
        self.load_all_rules(&paths)?;
        Ok(())
    }

    /// 处理单个语句
    fn process_statement(
        &self,
        statement: &crate::cwtools::models::Statement,
        rule_set: &mut RuleSet,
    ) -> Result<(), RuleError> {
        use crate::cwtools::models::Statement;

        match statement {
            Statement::KeyValue(kv) => {
                match kv.key.as_str() {
                    "types" => self.process_types(&kv.value, rule_set)?,
                    "enums" => self.process_enums(&kv.value, rule_set)?,
                    "alias" => self.process_alias(&kv.value, rule_set, kv.position)?,
                    _ => {
                        // 忽略未知的顶层键
                    }
                }
            }
            Statement::Comment(_, _) => {
                // 忽略注释
            }
            Statement::ValueOnly(_, pos) => {
                return Err(RuleError::with_position(
                    "Unexpected value-only statement at top level".to_string(),
                    *pos,
                ));
            }
        }

        Ok(())
    }

    /// 处理 types 定义
    fn process_types(&self, value: &AstValue, rule_set: &mut RuleSet) -> Result<(), RuleError> {
        let statements = match value {
            AstValue::Clause(stmts) => stmts,
            _ => {
                return Err(RuleError::new(
                    "Expected clause for 'types' definition".to_string(),
                ))
            }
        };

        for statement in statements {
            if let crate::cwtools::models::Statement::KeyValue(kv) = statement {
                let type_name = kv.key.clone();
                let type_def = self.parse_type_definition(&type_name, &kv.value)?;
                rule_set.add_type(type_name, type_def);
            }
        }

        Ok(())
    }

    /// 处理 enums 定义
    fn process_enums(&self, value: &AstValue, rule_set: &mut RuleSet) -> Result<(), RuleError> {
        let statements = match value {
            AstValue::Clause(stmts) => stmts,
            _ => {
                return Err(RuleError::new(
                    "Expected clause for 'enums' definition".to_string(),
                ))
            }
        };

        for statement in statements {
            if let crate::cwtools::models::Statement::KeyValue(kv) = statement {
                let enum_name = kv.key.clone();
                let enum_def = self.parse_enum_definition(&enum_name, &kv.value)?;
                rule_set.add_enum(enum_name, enum_def);
            }
        }

        Ok(())
    }

    /// 处理 alias 定义
    fn process_alias(
        &self,
        value: &AstValue,
        rule_set: &mut RuleSet,
        position: Position,
    ) -> Result<(), RuleError> {
        // alias 可以是单个定义或多个定义的子句
        match value {
            AstValue::Clause(statements) => {
                for statement in statements {
                    if let crate::cwtools::models::Statement::KeyValue(kv) = statement {
                        let alias_name = kv.key.clone();
                        let alias_rule = self.parse_alias_rule(&alias_name, &kv.value)?;
                        rule_set.add_alias(alias_name, alias_rule);
                    }
                }
            }
            _ => {
                return Err(RuleError::with_position(
                    "Expected clause for 'alias' definition".to_string(),
                    position,
                ))
            }
        }

        Ok(())
    }

    /// 解析类型定义
    fn parse_type_definition(
        &self,
        name: &str,
        value: &AstValue,
    ) -> Result<TypeDefinition, RuleError> {
        let mut type_def = TypeDefinition::new(name.to_string());

        let statements = match value {
            AstValue::Clause(stmts) => stmts,
            _ => {
                return Err(RuleError::new(format!(
                    "Expected clause for type definition '{}'",
                    name
                )))
            }
        };

        for statement in statements {
            if let crate::cwtools::models::Statement::KeyValue(kv) = statement {
                // 解析规则
                let rule = self.parse_rule(&kv.key, &kv.value, kv.operator)?;
                type_def.add_rule(rule);
            }
        }

        Ok(type_def)
    }

    /// 解析枚举定义
    fn parse_enum_definition(
        &self,
        name: &str,
        value: &AstValue,
    ) -> Result<EnumDefinition, RuleError> {
        let mut enum_def = EnumDefinition::new(name.to_string(), String::new());

        let statements = match value {
            AstValue::Clause(stmts) => stmts,
            _ => {
                return Err(RuleError::new(format!(
                    "Expected clause for enum definition '{}'",
                    name
                )))
            }
        };

        for statement in statements {
            match statement {
                crate::cwtools::models::Statement::ValueOnly(val, _) => {
                    if let Some(s) = val.as_string() {
                        enum_def.add_value(s.to_string());
                    }
                }
                crate::cwtools::models::Statement::KeyValue(kv) => {
                    // 枚举值也可以是键值对形式
                    enum_def.add_value(kv.key.clone());
                }
                _ => {}
            }
        }

        Ok(enum_def)
    }

    /// 解析别名规则
    fn parse_alias_rule(&self, name: &str, value: &AstValue) -> Result<AliasRule, RuleError> {
        // 别名规则通常是一个简单的规则定义
        let rule = match value {
            AstValue::Clause(statements) => {
                // 如果是子句，解析为节点规则
                let mut children = Vec::new();
                for statement in statements {
                    if let crate::cwtools::models::Statement::KeyValue(kv) = statement {
                        let child_rule = self.parse_rule(&kv.key, &kv.value, kv.operator)?;
                        children.push(child_rule);
                    }
                }
                Rule::new(
                    RuleType::NodeRule {
                        left: FieldType::Specific(name.to_string()),
                        children,
                    },
                    RuleOptions::default(),
                )
            }
            _ => {
                // 如果是简单值，解析为叶子规则
                let field_type = self.parse_field_type(value)?;
                Rule::new(
                    RuleType::LeafRule {
                        left: FieldType::Specific(name.to_string()),
                        right: field_type,
                    },
                    RuleOptions::default(),
                )
            }
        };

        Ok(AliasRule::new(name.to_string(), rule))
    }

    /// 解析规则
    fn parse_rule(
        &self,
        key: &str,
        value: &AstValue,
        operator: Operator,
    ) -> Result<Rule, RuleError> {
        let left = self.parse_field_type_from_key(key)?;

        let rule_type = match value {
            AstValue::Clause(statements) => {
                // 节点规则：键值对，值为子句
                let mut children = Vec::new();
                let mut options = self.parse_rule_options(operator);

                for statement in statements {
                    match statement {
                        crate::cwtools::models::Statement::KeyValue(kv) => {
                            // 检查是否是选项定义
                            if self.is_option_key(&kv.key) {
                                self.parse_option_into(&kv.key, &kv.value, &mut options)?;
                            } else {
                                let child_rule =
                                    self.parse_rule(&kv.key, &kv.value, kv.operator)?;
                                children.push(child_rule);
                            }
                        }
                        _ => {}
                    }
                }
                return Ok(Rule::new(RuleType::NodeRule { left, children }, options));
            }
            _ => {
                // 叶子规则：键值对，值为简单类型
                let right = self.parse_field_type(value)?;
                RuleType::LeafRule { left, right }
            }
        };

        // 解析规则选项（从操作符推断）
        let options = self.parse_rule_options(operator);

        Ok(Rule::new(rule_type, options))
    }

    /// 从键解析字段类型
    fn parse_field_type_from_key(&self, key: &str) -> Result<FieldType, RuleError> {
        // 检查是否是特殊语法
        if key.starts_with('<') && key.ends_with('>') {
            // 类型引用，如 <country>
            let type_name = &key[1..key.len() - 1];
            return Ok(FieldType::Type(type_name.to_string()));
        }

        if key.starts_with("enum[") && key.ends_with(']') {
            // 枚举引用，如 enum[country_tags]
            let enum_name = &key[5..key.len() - 1];
            return Ok(FieldType::Enum(enum_name.to_string()));
        }

        // 默认为特定字符串
        Ok(FieldType::Specific(key.to_string()))
    }

    /// 从值解析字段类型
    fn parse_field_type(&self, value: &AstValue) -> Result<FieldType, RuleError> {
        match value {
            AstValue::String(s) | AstValue::QuotedString(s) => self.parse_field_type_from_string(s),
            AstValue::Integer(_) => Ok(FieldType::Value(ValueType::int())),
            AstValue::Float(_) => Ok(FieldType::Value(ValueType::float())),
            AstValue::Boolean(_) => Ok(FieldType::Value(ValueType::Boolean)),
            AstValue::Clause(_) => Err(RuleError::new(
                "Unexpected clause in field type".to_string(),
            )),
        }
    }

    /// 从字符串解析字段类型
    fn parse_field_type_from_string(&self, s: &str) -> Result<FieldType, RuleError> {
        match s {
            "int" => Ok(FieldType::Value(ValueType::int())),
            "float" => Ok(FieldType::Value(ValueType::float())),
            "bool" => Ok(FieldType::Value(ValueType::Boolean)),
            "percent" => Ok(FieldType::Value(ValueType::Percent)),
            "date" => Ok(FieldType::Value(ValueType::Date)),
            "scalar" => Ok(FieldType::Scalar),
            s if s.starts_with('<') && s.ends_with('>') => {
                let type_name = &s[1..s.len() - 1];
                Ok(FieldType::Type(type_name.to_string()))
            }
            s if s.starts_with("enum[") && s.ends_with(']') => {
                let enum_name = &s[5..s.len() - 1];
                Ok(FieldType::Enum(enum_name.to_string()))
            }
            _ => Ok(FieldType::Specific(s.to_string())),
        }
    }

    /// 解析规则选项
    fn parse_rule_options(&self, operator: Operator) -> RuleOptions {
        let mut options = RuleOptions::default();

        // 根据操作符设置选项
        match operator {
            Operator::Equals => {
                // = 表示可选字段
                options.min = 0;
                options.max = Some(1);
            }
            Operator::GreaterThan => {
                // > 表示至少一个
                options.min = 1;
                options.max = None;
            }
            Operator::LessThan => {
                // < 表示最多一个
                options.min = 0;
                options.max = Some(1);
            }
            _ => {
                // 其他操作符使用默认值
            }
        }

        options
    }

    /// 检查键是否是选项键
    fn is_option_key(&self, key: &str) -> bool {
        matches!(
            key,
            "min"
                | "max"
                | "severity"
                | "push_scope"
                | "required_scopes"
                | "scope"
                | "description"
                | "warning_only"
        )
    }

    /// 将选项值解析到 RuleOptions 中
    fn parse_option_into(
        &self,
        key: &str,
        value: &AstValue,
        options: &mut RuleOptions,
    ) -> Result<(), RuleError> {
        match key {
            "min" => {
                if let Some(i) = value.as_integer() {
                    options.min = i as usize;
                } else {
                    return Err(RuleError::new(format!(
                        "Expected integer for 'min' option, got {:?}",
                        value
                    )));
                }
            }
            "max" => {
                if let Some(i) = value.as_integer() {
                    options.max = Some(i as usize);
                } else {
                    return Err(RuleError::new(format!(
                        "Expected integer for 'max' option, got {:?}",
                        value
                    )));
                }
            }
            "severity" => {
                if let Some(s) = value.as_string() {
                    options.severity = Some(self.parse_severity(s)?);
                } else {
                    return Err(RuleError::new(format!(
                        "Expected string for 'severity' option, got {:?}",
                        value
                    )));
                }
            }
            "push_scope" => {
                if let Some(s) = value.as_string() {
                    options.push_scope = Some(self.parse_scope(s)?);
                } else {
                    return Err(RuleError::new(format!(
                        "Expected string for 'push_scope' option, got {:?}",
                        value
                    )));
                }
            }
            "required_scopes" | "scope" => {
                let scopes = self.parse_scopes(value)?;
                options.required_scopes = scopes;
            }
            "description" => {
                if let Some(s) = value.as_string() {
                    options.description = Some(s.to_string());
                } else {
                    return Err(RuleError::new(format!(
                        "Expected string for 'description' option, got {:?}",
                        value
                    )));
                }
            }
            "warning_only" => {
                if let Some(b) = value.as_boolean() {
                    options.warning_only = b;
                } else {
                    return Err(RuleError::new(format!(
                        "Expected boolean for 'warning_only' option, got {:?}",
                        value
                    )));
                }
            }
            _ => {
                // 忽略未知选项
            }
        }

        Ok(())
    }

    /// 解析严重程度
    fn parse_severity(&self, s: &str) -> Result<Severity, RuleError> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Severity::Error),
            "warning" => Ok(Severity::Warning),
            "information" | "info" => Ok(Severity::Information),
            "hint" => Ok(Severity::Hint),
            _ => Err(RuleError::new(format!("Unknown severity: {}", s))),
        }
    }

    /// 解析作用域
    fn parse_scope(&self, s: &str) -> Result<Scope, RuleError> {
        Scope::from_str(s).ok_or_else(|| RuleError::new(format!("Unknown scope: {}", s)))
    }

    /// 解析作用域列表
    fn parse_scopes(&self, value: &AstValue) -> Result<Vec<Scope>, RuleError> {
        match value {
            AstValue::String(s) | AstValue::QuotedString(s) => {
                // 单个作用域
                Ok(vec![self.parse_scope(s)?])
            }
            AstValue::Clause(statements) => {
                // 多个作用域
                let mut scopes = Vec::new();
                for statement in statements {
                    if let crate::cwtools::models::Statement::ValueOnly(val, _) = statement {
                        if let Some(s) = val.as_string() {
                            scopes.push(self.parse_scope(s)?);
                        }
                    }
                }
                Ok(scopes)
            }
            _ => Err(RuleError::new(format!(
                "Expected string or clause for scopes, got {:?}",
                value
            ))),
        }
    }

    /// 合并规则集
    ///
    /// 将源规则集合并到目标规则集中。
    /// 对于相同名称的类型定义，会合并其规则列表。
    /// 对于枚举和别名，后加载的会覆盖先加载的。
    fn merge_rule_sets(&self, target: &mut RuleSet, source: RuleSet) {
        // 合并类型定义
        for (name, source_type) in source.types {
            if let Some(target_type) = target.types.get_mut(&name) {
                // 如果类型已存在，合并规则
                target_type.rules.extend(source_type.rules);
                target_type.subtypes.extend(source_type.subtypes);

                // 合并选项（后加载的覆盖先加载的）
                if source_type.options.skip_root_key {
                    target_type.options.skip_root_key = true;
                }
                if source_type.options.path_prefix.is_some() {
                    target_type.options.path_prefix = source_type.options.path_prefix;
                }
                if source_type.options.path_extension.is_some() {
                    target_type.options.path_extension = source_type.options.path_extension;
                }
            } else {
                // 如果类型不存在，直接插入
                target.types.insert(name, source_type);
            }
        }

        // 合并枚举定义（后加载的覆盖先加载的）
        for (name, source_enum) in source.enums {
            if let Some(target_enum) = target.enums.get_mut(&name) {
                // 合并枚举值，去重
                for value in source_enum.values {
                    if !target_enum.values.contains(&value) {
                        target_enum.values.push(value);
                    }
                }
                // 更新描述
                if !source_enum.description.is_empty() {
                    target_enum.description = source_enum.description;
                }
            } else {
                target.enums.insert(name, source_enum);
            }
        }

        // 合并别名规则（后加载的覆盖先加载的）
        for (name, alias) in source.aliases {
            target.aliases.insert(name, alias);
        }

        // 合并修饰符定义（去重）
        for modifier in source.modifiers {
            // 检查是否已存在同名修饰符
            if !target.modifiers.iter().any(|m| m.name == modifier.name) {
                target.modifiers.push(modifier);
            }
        }
    }

    /// 验证规则集的一致性
    ///
    /// 检查规则集中的引用是否有效
    pub fn validate_rule_set(&self, rule_set: &RuleSet) -> Vec<RuleError> {
        let mut errors = Vec::new();

        // 检查类型引用
        for (type_name, type_def) in &rule_set.types {
            for rule in &type_def.rules {
                self.validate_rule_references(rule, rule_set, type_name, &mut errors);
            }
        }

        errors
    }

    /// 验证规则中的引用
    fn validate_rule_references(
        &self,
        rule: &Rule,
        rule_set: &RuleSet,
        context: &str,
        errors: &mut Vec<RuleError>,
    ) {
        match &rule.rule_type {
            RuleType::NodeRule { left, children } => {
                self.validate_field_type_references(left, rule_set, context, errors);
                for child in children {
                    self.validate_rule_references(child, rule_set, context, errors);
                }
            }
            RuleType::LeafRule { left, right } => {
                self.validate_field_type_references(left, rule_set, context, errors);
                self.validate_field_type_references(right, rule_set, context, errors);
            }
            RuleType::LeafValueRule { right } => {
                self.validate_field_type_references(right, rule_set, context, errors);
            }
            RuleType::ValueClauseRule { children } => {
                for child in children {
                    self.validate_rule_references(child, rule_set, context, errors);
                }
            }
        }
    }

    /// 验证字段类型引用
    fn validate_field_type_references(
        &self,
        field_type: &FieldType,
        rule_set: &RuleSet,
        context: &str,
        errors: &mut Vec<RuleError>,
    ) {
        match field_type {
            FieldType::Type(type_name) => {
                if !rule_set.types.contains_key(type_name) {
                    errors.push(RuleError::new(format!(
                        "Type '{}' referenced in '{}' is not defined",
                        type_name, context
                    )));
                }
            }
            FieldType::Enum(enum_name) => {
                if !rule_set.enums.contains_key(enum_name) {
                    errors.push(RuleError::new(format!(
                        "Enum '{}' referenced in '{}' is not defined",
                        enum_name, context
                    )));
                }
            }
            FieldType::Alias(alias_name) => {
                if !rule_set.aliases.contains_key(alias_name) {
                    errors.push(RuleError::new(format!(
                        "Alias '{}' referenced in '{}' is not defined",
                        alias_name, context
                    )));
                }
            }
            _ => {}
        }
    }
}

impl Default for RuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_loader_creation() {
        let loader = RuleLoader::new();
        assert_eq!(loader.rule_files.len(), 0);
        assert_eq!(loader.cache.len(), 0);
    }

    #[test]
    fn test_add_rule_file() {
        let mut loader = RuleLoader::new();
        loader.add_rule_file(PathBuf::from("test.cwt"));
        assert_eq!(loader.rule_files.len(), 1);
    }

    #[test]
    fn test_parse_field_type_from_string() {
        let loader = RuleLoader::new();

        assert!(matches!(
            loader.parse_field_type_from_string("int"),
            Ok(FieldType::Value(ValueType::Int { .. }))
        ));

        assert!(matches!(
            loader.parse_field_type_from_string("float"),
            Ok(FieldType::Value(ValueType::Float { .. }))
        ));

        assert!(matches!(
            loader.parse_field_type_from_string("bool"),
            Ok(FieldType::Value(ValueType::Boolean))
        ));

        assert!(matches!(
            loader.parse_field_type_from_string("scalar"),
            Ok(FieldType::Scalar)
        ));

        assert!(matches!(
            loader.parse_field_type_from_string("<country>"),
            Ok(FieldType::Type(s)) if s == "country"
        ));

        assert!(matches!(
            loader.parse_field_type_from_string("enum[tags]"),
            Ok(FieldType::Enum(s)) if s == "tags"
        ));
    }

    #[test]
    fn test_parse_rule_options() {
        let loader = RuleLoader::new();

        let opts = loader.parse_rule_options(Operator::Equals);
        assert_eq!(opts.min, 0);
        assert_eq!(opts.max, Some(1));

        let opts = loader.parse_rule_options(Operator::GreaterThan);
        assert_eq!(opts.min, 1);
        assert_eq!(opts.max, None);

        let opts = loader.parse_rule_options(Operator::LessThan);
        assert_eq!(opts.min, 0);
        assert_eq!(opts.max, Some(1));
    }

    #[test]
    fn test_rule_error_display() {
        let error = RuleError::new("Test error".to_string());
        assert_eq!(error.to_string(), "Test error");

        let error = RuleError::with_position("Test error".to_string(), Position::new(10, 5, 100));
        assert_eq!(error.to_string(), "10:5: Test error");

        let error = RuleError::new("Test error".to_string()).with_file("test.cwt".to_string());
        assert_eq!(error.to_string(), "test.cwt:Test error");
    }

    #[test]
    fn test_is_option_key() {
        let loader = RuleLoader::new();
        assert!(loader.is_option_key("min"));
        assert!(loader.is_option_key("max"));
        assert!(loader.is_option_key("severity"));
        assert!(loader.is_option_key("push_scope"));
        assert!(loader.is_option_key("required_scopes"));
        assert!(loader.is_option_key("scope"));
        assert!(loader.is_option_key("description"));
        assert!(loader.is_option_key("warning_only"));
        assert!(!loader.is_option_key("unknown"));
    }

    #[test]
    fn test_parse_severity() {
        let loader = RuleLoader::new();
        assert!(matches!(
            loader.parse_severity("error"),
            Ok(Severity::Error)
        ));
        assert!(matches!(
            loader.parse_severity("warning"),
            Ok(Severity::Warning)
        ));
        assert!(matches!(
            loader.parse_severity("info"),
            Ok(Severity::Information)
        ));
        assert!(matches!(loader.parse_severity("hint"), Ok(Severity::Hint)));
        assert!(loader.parse_severity("unknown").is_err());
    }

    #[test]
    fn test_parse_scope() {
        let loader = RuleLoader::new();
        assert!(matches!(loader.parse_scope("country"), Ok(Scope::Country)));
        assert!(matches!(loader.parse_scope("state"), Ok(Scope::State)));
        assert!(matches!(
            loader.parse_scope("unit_leader"),
            Ok(Scope::UnitLeader)
        ));
        assert!(matches!(loader.parse_scope("air"), Ok(Scope::Air)));
        assert!(matches!(loader.parse_scope("any"), Ok(Scope::Any)));
        assert!(loader.parse_scope("unknown").is_err());
    }

    #[test]
    fn test_parse_option_into() {
        let loader = RuleLoader::new();
        let mut options = RuleOptions::default();

        // 测试 min
        loader
            .parse_option_into("min", &AstValue::Integer(5), &mut options)
            .unwrap();
        assert_eq!(options.min, 5);

        // 测试 max
        loader
            .parse_option_into("max", &AstValue::Integer(10), &mut options)
            .unwrap();
        assert_eq!(options.max, Some(10));

        // 测试 severity
        loader
            .parse_option_into(
                "severity",
                &AstValue::String("warning".to_string()),
                &mut options,
            )
            .unwrap();
        assert_eq!(options.severity, Some(Severity::Warning));

        // 测试 push_scope
        loader
            .parse_option_into(
                "push_scope",
                &AstValue::String("country".to_string()),
                &mut options,
            )
            .unwrap();
        assert_eq!(options.push_scope, Some(Scope::Country));

        // 测试 description
        loader
            .parse_option_into(
                "description",
                &AstValue::String("Test description".to_string()),
                &mut options,
            )
            .unwrap();
        assert_eq!(options.description, Some("Test description".to_string()));

        // 测试 warning_only
        loader
            .parse_option_into("warning_only", &AstValue::Boolean(true), &mut options)
            .unwrap();
        assert!(options.warning_only);
    }

    #[test]
    fn test_merge_rule_sets() {
        let loader = RuleLoader::new();
        let mut target = RuleSet::new();
        let mut source = RuleSet::new();

        // 添加类型到 target
        let mut type1 = TypeDefinition::new("type1".to_string());
        type1.add_rule(Rule::leaf_rule(
            FieldType::Specific("key1".to_string()),
            FieldType::Scalar,
        ));
        target.add_type("type1".to_string(), type1);

        // 添加相同类型到 source，但有不同的规则
        let mut type1_source = TypeDefinition::new("type1".to_string());
        type1_source.add_rule(Rule::leaf_rule(
            FieldType::Specific("key2".to_string()),
            FieldType::Scalar,
        ));
        source.add_type("type1".to_string(), type1_source);

        // 添加新类型到 source
        let type2 = TypeDefinition::new("type2".to_string());
        source.add_type("type2".to_string(), type2);

        // 合并
        loader.merge_rule_sets(&mut target, source);

        // 验证合并结果
        assert_eq!(target.types.len(), 2);
        assert!(target.types.contains_key("type1"));
        assert!(target.types.contains_key("type2"));

        // type1 应该有两个规则
        let merged_type1 = target.types.get("type1").unwrap();
        assert_eq!(merged_type1.rules.len(), 2);
    }

    #[test]
    fn test_merge_enums() {
        let loader = RuleLoader::new();
        let mut target = RuleSet::new();
        let mut source = RuleSet::new();

        // 添加枚举到 target
        let mut enum1 = EnumDefinition::new("enum1".to_string(), "Test enum".to_string());
        enum1.add_value("value1".to_string());
        enum1.add_value("value2".to_string());
        target.add_enum("enum1".to_string(), enum1);

        // 添加相同枚举到 source，但有不同的值
        let mut enum1_source = EnumDefinition::new("enum1".to_string(), "Updated enum".to_string());
        enum1_source.add_value("value2".to_string()); // 重复值
        enum1_source.add_value("value3".to_string()); // 新值
        source.add_enum("enum1".to_string(), enum1_source);

        // 合并
        loader.merge_rule_sets(&mut target, source);

        // 验证合并结果
        let merged_enum = target.enums.get("enum1").unwrap();
        assert_eq!(merged_enum.values.len(), 3); // value1, value2, value3
        assert_eq!(merged_enum.description, "Updated enum");
    }

    #[test]
    fn test_validate_rule_set() {
        let loader = RuleLoader::new();
        let mut rule_set = RuleSet::new();

        // 添加一个引用不存在类型的规则
        let mut type1 = TypeDefinition::new("type1".to_string());
        type1.add_rule(Rule::leaf_rule(
            FieldType::Specific("key1".to_string()),
            FieldType::Type("nonexistent_type".to_string()),
        ));
        rule_set.add_type("type1".to_string(), type1);

        // 验证
        let errors = loader.validate_rule_set(&rule_set);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("nonexistent_type"));
    }

    #[test]
    fn test_validate_enum_reference() {
        let loader = RuleLoader::new();
        let mut rule_set = RuleSet::new();

        // 添加一个引用不存在枚举的规则
        let mut type1 = TypeDefinition::new("type1".to_string());
        type1.add_rule(Rule::leaf_rule(
            FieldType::Specific("key1".to_string()),
            FieldType::Enum("nonexistent_enum".to_string()),
        ));
        rule_set.add_type("type1".to_string(), type1);

        // 验证
        let errors = loader.validate_rule_set(&rule_set);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("nonexistent_enum"));
    }
}
