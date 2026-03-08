//! 规则引擎数据类型的单元测试

#[cfg(test)]
mod tests {
    use crate::cwtools::diagnostic::Severity;
    use crate::cwtools::rules::types::*;
    use crate::cwtools::validator::scope::Scope;

    #[test]
    fn test_ruleset_creation() {
        let ruleset = RuleSet::new();
        assert_eq!(ruleset.types.len(), 0);
        assert_eq!(ruleset.enums.len(), 0);
        assert_eq!(ruleset.aliases.len(), 0);
        assert_eq!(ruleset.modifiers.len(), 0);
    }

    #[test]
    fn test_ruleset_add_type() {
        let mut ruleset = RuleSet::new();
        let type_def = TypeDefinition::new("test_type".to_string());
        ruleset.add_type("test_type".to_string(), type_def);

        assert_eq!(ruleset.types.len(), 1);
        assert!(ruleset.get_type("test_type").is_some());
        assert!(ruleset.get_type("nonexistent").is_none());
    }

    #[test]
    fn test_ruleset_add_enum() {
        let mut ruleset = RuleSet::new();
        let enum_def = EnumDefinition::new("test_enum".to_string(), "Test enum".to_string());
        ruleset.add_enum("test_enum".to_string(), enum_def);

        assert_eq!(ruleset.enums.len(), 1);
        assert!(ruleset.get_enum("test_enum").is_some());
        assert!(ruleset.get_enum("nonexistent").is_none());
    }

    #[test]
    fn test_ruleset_add_alias() {
        let mut ruleset = RuleSet::new();
        let rule = Rule::leaf_rule(FieldType::Scalar, FieldType::Scalar);
        let alias = AliasRule::new("test_alias".to_string(), rule);
        ruleset.add_alias("test_alias".to_string(), alias);

        assert_eq!(ruleset.aliases.len(), 1);
        assert!(ruleset.get_alias("test_alias").is_some());
        assert!(ruleset.get_alias("nonexistent").is_none());
    }

    #[test]
    fn test_ruleset_add_modifier() {
        let mut ruleset = RuleSet::new();
        let modifier = ModifierDefinition::new(
            "test_modifier".to_string(),
            ModifierCategory::Country,
            vec![Scope::Country],
        );
        ruleset.add_modifier(modifier);

        assert_eq!(ruleset.modifiers.len(), 1);
    }

    #[test]
    fn test_type_definition_creation() {
        let type_def = TypeDefinition::new("test_type".to_string());
        assert_eq!(type_def.name, "test_type");
        assert_eq!(type_def.rules.len(), 0);
        assert_eq!(type_def.subtypes.len(), 0);
    }

    #[test]
    fn test_type_definition_add_rule() {
        let mut type_def = TypeDefinition::new("test_type".to_string());
        let rule = Rule::leaf_rule(FieldType::Scalar, FieldType::Scalar);
        type_def.add_rule(rule);

        assert_eq!(type_def.rules.len(), 1);
    }

    #[test]
    fn test_type_definition_add_subtype() {
        let mut type_def = TypeDefinition::new("test_type".to_string());
        let subtype = SubTypeDefinition::new("subtype".to_string());
        type_def.add_subtype(subtype);

        assert_eq!(type_def.subtypes.len(), 1);
    }

    #[test]
    fn test_subtype_definition_creation() {
        let subtype = SubTypeDefinition::new("subtype".to_string());
        assert_eq!(subtype.name, "subtype");
        assert_eq!(subtype.rules.len(), 0);
    }

    #[test]
    fn test_subtype_definition_add_rule() {
        let mut subtype = SubTypeDefinition::new("subtype".to_string());
        let rule = Rule::leaf_rule(FieldType::Scalar, FieldType::Scalar);
        subtype.add_rule(rule);

        assert_eq!(subtype.rules.len(), 1);
    }

    #[test]
    fn test_rule_creation() {
        let rule = Rule::new(
            RuleType::LeafRule {
                left: FieldType::Scalar,
                right: FieldType::Scalar,
            },
            RuleOptions::default(),
        );

        match rule.rule_type {
            RuleType::LeafRule { .. } => {}
            _ => panic!("Expected LeafRule"),
        }
    }

    #[test]
    fn test_rule_node_rule() {
        let rule = Rule::node_rule(FieldType::Scalar, vec![]);

        match rule.rule_type {
            RuleType::NodeRule {
                ref left,
                ref children,
            } => {
                assert_eq!(*left, FieldType::Scalar);
                assert_eq!(children.len(), 0);
            }
            _ => panic!("Expected NodeRule"),
        }
    }

    #[test]
    fn test_rule_leaf_rule() {
        let rule = Rule::leaf_rule(FieldType::Scalar, FieldType::Value(ValueType::int()));

        match rule.rule_type {
            RuleType::LeafRule {
                ref left,
                ref right,
            } => {
                assert_eq!(*left, FieldType::Scalar);
                match right {
                    FieldType::Value(ValueType::Int { .. }) => {}
                    _ => panic!("Expected Value(Int)"),
                }
            }
            _ => panic!("Expected LeafRule"),
        }
    }

    #[test]
    fn test_rule_type_variants() {
        let node_rule = RuleType::NodeRule {
            left: FieldType::Scalar,
            children: vec![],
        };

        let leaf_rule = RuleType::LeafRule {
            left: FieldType::Scalar,
            right: FieldType::Scalar,
        };

        let leaf_value_rule = RuleType::LeafValueRule {
            right: FieldType::Scalar,
        };

        let value_clause_rule = RuleType::ValueClauseRule { children: vec![] };

        match node_rule {
            RuleType::NodeRule { .. } => {}
            _ => panic!("Expected NodeRule"),
        }

        match leaf_rule {
            RuleType::LeafRule { .. } => {}
            _ => panic!("Expected LeafRule"),
        }

        match leaf_value_rule {
            RuleType::LeafValueRule { .. } => {}
            _ => panic!("Expected LeafValueRule"),
        }

        match value_clause_rule {
            RuleType::ValueClauseRule { .. } => {}
            _ => panic!("Expected ValueClauseRule"),
        }
    }

    #[test]
    fn test_field_type_variants() {
        let value_type = FieldType::Value(ValueType::int());
        let specific = FieldType::Specific("test".to_string());
        let scalar = FieldType::Scalar;
        let type_ref = FieldType::Type("MyType".to_string());
        let scope = FieldType::Scope(vec![Scope::Country]);
        let localisation = FieldType::Localisation {
            synced: true,
            inline: false,
        };
        let filepath = FieldType::Filepath {
            prefix: Some("gfx/".to_string()),
            extension: Some(".dds".to_string()),
        };
        let enum_type = FieldType::Enum("MyEnum".to_string());
        let alias = FieldType::Alias("MyAlias".to_string());
        let variable = FieldType::Variable {
            is_int: true,
            min: 0.0,
            max: 100.0,
        };

        assert!(matches!(value_type, FieldType::Value(_)));
        assert!(matches!(specific, FieldType::Specific(_)));
        assert!(matches!(scalar, FieldType::Scalar));
        assert!(matches!(type_ref, FieldType::Type(_)));
        assert!(matches!(scope, FieldType::Scope(_)));
        assert!(matches!(localisation, FieldType::Localisation { .. }));
        assert!(matches!(filepath, FieldType::Filepath { .. }));
        assert!(matches!(enum_type, FieldType::Enum(_)));
        assert!(matches!(alias, FieldType::Alias(_)));
        assert!(matches!(variable, FieldType::Variable { .. }));
    }

    #[test]
    fn test_value_type_int() {
        let int_type = ValueType::int();
        match int_type {
            ValueType::Int { min, max } => {
                assert_eq!(min, i64::MIN);
                assert_eq!(max, i64::MAX);
            }
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_value_type_int_range() {
        let int_type = ValueType::int_range(0, 100);
        match int_type {
            ValueType::Int { min, max } => {
                assert_eq!(min, 0);
                assert_eq!(max, 100);
            }
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_value_type_float() {
        let float_type = ValueType::float();
        match float_type {
            ValueType::Float { min, max } => {
                assert_eq!(min, f64::MIN);
                assert_eq!(max, f64::MAX);
            }
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_value_type_float_range() {
        let float_type = ValueType::float_range(0.0, 1.0);
        match float_type {
            ValueType::Float { min, max } => {
                assert_eq!(min, 0.0);
                assert_eq!(max, 1.0);
            }
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_value_type_variants() {
        let boolean = ValueType::Boolean;
        let percent = ValueType::Percent;
        let date = ValueType::Date;

        assert!(matches!(boolean, ValueType::Boolean));
        assert!(matches!(percent, ValueType::Percent));
        assert!(matches!(date, ValueType::Date));
    }

    #[test]
    fn test_rule_options_default() {
        let options = RuleOptions::default();
        assert_eq!(options.min, 0);
        assert_eq!(options.max, None);
        assert_eq!(options.required_scopes.len(), 0);
        assert_eq!(options.push_scope, None);
        assert_eq!(options.severity, None);
        assert_eq!(options.description, None);
        assert!(!options.warning_only);
    }

    #[test]
    fn test_rule_options_builder() {
        let options = RuleOptions::new()
            .with_min(1)
            .with_max(5)
            .with_required_scopes(vec![Scope::Country])
            .with_push_scope(Scope::State)
            .with_severity(Severity::Warning)
            .with_description("Test description".to_string())
            .with_warning_only(true);

        assert_eq!(options.min, 1);
        assert_eq!(options.max, Some(5));
        assert_eq!(options.required_scopes.len(), 1);
        assert_eq!(options.push_scope, Some(Scope::State));
        assert_eq!(options.severity, Some(Severity::Warning));
        assert_eq!(options.description, Some("Test description".to_string()));
        assert!(options.warning_only);
    }

    #[test]
    fn test_enum_definition_creation() {
        let enum_def = EnumDefinition::new("test_enum".to_string(), "Test enum".to_string());
        assert_eq!(enum_def.key, "test_enum");
        assert_eq!(enum_def.description, "Test enum");
        assert_eq!(enum_def.values.len(), 0);
    }

    #[test]
    fn test_enum_definition_add_value() {
        let mut enum_def = EnumDefinition::new("test_enum".to_string(), "Test enum".to_string());
        enum_def.add_value("value1".to_string());
        enum_def.add_value("value2".to_string());

        assert_eq!(enum_def.values.len(), 2);
        assert_eq!(enum_def.values[0], "value1");
        assert_eq!(enum_def.values[1], "value2");
    }

    #[test]
    fn test_enum_definition_contains() {
        let mut enum_def = EnumDefinition::new("test_enum".to_string(), "Test enum".to_string());
        enum_def.add_value("value1".to_string());
        enum_def.add_value("value2".to_string());

        assert!(enum_def.contains("value1"));
        assert!(enum_def.contains("value2"));
        assert!(!enum_def.contains("value3"));
    }

    #[test]
    fn test_alias_rule_creation() {
        let rule = Rule::leaf_rule(FieldType::Scalar, FieldType::Scalar);
        let alias = AliasRule::new("test_alias".to_string(), rule);

        assert_eq!(alias.name, "test_alias");
    }

    #[test]
    fn test_modifier_definition_creation() {
        let modifier = ModifierDefinition::new(
            "test_modifier".to_string(),
            ModifierCategory::Country,
            vec![Scope::Country, Scope::State],
        );

        assert_eq!(modifier.name, "test_modifier");
        assert_eq!(modifier.category, ModifierCategory::Country);
        assert_eq!(modifier.scopes.len(), 2);
    }

    #[test]
    fn test_modifier_definition_with_value_type() {
        let modifier = ModifierDefinition::new(
            "test_modifier".to_string(),
            ModifierCategory::Country,
            vec![Scope::Country],
        )
        .with_value_type(ValueType::int_range(0, 100));

        match modifier.value_type {
            ValueType::Int { min, max } => {
                assert_eq!(min, 0);
                assert_eq!(max, 100);
            }
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_modifier_category_variants() {
        let categories = vec![
            ModifierCategory::Country,
            ModifierCategory::State,
            ModifierCategory::Unit,
            ModifierCategory::UnitLeader,
            ModifierCategory::Air,
        ];

        assert_eq!(categories.len(), 5);
    }

    #[test]
    fn test_scope_from_str() {
        assert_eq!(Scope::from_str("country"), Some(Scope::Country));
        assert_eq!(Scope::from_str("Country"), Some(Scope::Country));
        assert_eq!(Scope::from_str("COUNTRY"), Some(Scope::Country));
        assert_eq!(Scope::from_str("state"), Some(Scope::State));
        assert_eq!(Scope::from_str("unit_leader"), Some(Scope::UnitLeader));
        assert_eq!(Scope::from_str("air"), Some(Scope::Air));
        assert_eq!(Scope::from_str("any"), Some(Scope::Any));
        assert_eq!(Scope::from_str("invalid"), None);
    }

    #[test]
    fn test_scope_as_str() {
        assert_eq!(Scope::Country.as_str(), "country");
        assert_eq!(Scope::State.as_str(), "state");
        assert_eq!(Scope::UnitLeader.as_str(), "unit_leader");
        assert_eq!(Scope::Air.as_str(), "air");
        assert_eq!(Scope::Any.as_str(), "any");
    }

    #[test]
    fn test_scope_variants() {
        let scopes = vec![
            Scope::Country,
            Scope::State,
            Scope::UnitLeader,
            Scope::Air,
            Scope::Any,
        ];

        assert_eq!(scopes.len(), 5);
    }

    #[test]
    fn test_severity_ordering() {
        // Severity 按照枚举定义顺序排序
        // Error < Warning < Information < Hint
        assert!(Severity::Error < Severity::Warning);
        assert!(Severity::Warning < Severity::Information);
        assert!(Severity::Information < Severity::Hint);
    }

    #[test]
    fn test_severity_as_str() {
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Information.as_str(), "information");
        assert_eq!(Severity::Hint.as_str(), "hint");
    }

    #[test]
    fn test_severity_variants() {
        let severities = vec![
            Severity::Error,
            Severity::Warning,
            Severity::Information,
            Severity::Hint,
        ];

        assert_eq!(severities.len(), 4);
    }
}
