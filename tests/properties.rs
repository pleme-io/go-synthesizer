use proptest::prelude::*;
use go_synthesizer::*;

fn arb_go_type() -> impl Strategy<Value = GoType> {
    prop_oneof![
        Just(GoType::Named("string".into())),
        Just(GoType::Named("int".into())),
        Just(GoType::Named("bool".into())),
        Just(GoType::Named("float64".into())),
        Just(GoType::Any),
    ]
}

fn arb_go_expr() -> impl Strategy<Value = GoExpr> {
    prop_oneof![
        any::<i64>().prop_map(GoExpr::Int),
        any::<bool>().prop_map(GoExpr::Bool),
        Just(GoExpr::Nil),
        "[a-z]{1,10}".prop_map(|s| GoExpr::Str(s)),
        "[a-z]{1,8}".prop_map(|s| GoExpr::Ident(s)),
    ]
}

fn arb_go_stmt() -> impl Strategy<Value = GoStmt> {
    prop_oneof![
        Just(GoStmt::Blank),
        Just(GoStmt::Return(vec![])),
        ("[a-z]{1,8}", arb_go_expr()).prop_map(|(n, v)| GoStmt::ShortAssign { name: n, value: v }),
    ]
}

proptest! {
    #[test]
    fn go_expr_no_panic(expr in arb_go_expr()) {
        let _ = expr.emit();
    }

    #[test]
    fn go_expr_deterministic(expr in arb_go_expr()) {
        prop_assert_eq!(expr.emit(), expr.emit());
    }

    #[test]
    fn go_stmt_no_panic(stmt in arb_go_stmt()) {
        let _ = stmt.emit(0);
    }

    #[test]
    fn go_stmt_deterministic(stmt in arb_go_stmt()) {
        prop_assert_eq!(stmt.emit(0), stmt.emit(0));
    }

    #[test]
    fn go_type_no_panic(t in arb_go_type()) {
        let out = t.emit();
        prop_assert!(!out.is_empty());
    }

    #[test]
    fn go_tab_indentation(stmt in arb_go_stmt()) {
        let out = stmt.emit(1);
        if !out.is_empty() {
            prop_assert!(out.starts_with('\t'), "Go uses tab indentation: {out}");
        }
    }

    #[test]
    fn go_file_trailing_newline(expr in arb_go_expr()) {
        let node = GoNode::VarDecl {
            name: "x".into(),
            var_type: Some(GoType::Named("int".into())),
            value: Some(expr),
        };
        let out = emit_file(&[node]);
        prop_assert!(out.ends_with('\n'));
    }

    #[test]
    fn go_struct_balanced_braces(
        name in "[A-Z][a-z]{1,8}",
        field_name in "[A-Z][a-z]{1,8}",
    ) {
        let node = GoNode::Struct {
            name,
            doc: None,
            fields: vec![GoField::new(&field_name, GoType::Named("string".into()))],
        };
        let out = node.emit(0);
        let opens = out.chars().filter(|c| *c == '{').count();
        let closes = out.chars().filter(|c| *c == '}').count();
        prop_assert_eq!(opens, closes);
    }

    #[test]
    fn go_no_control_chars(expr in arb_go_expr()) {
        let out = expr.emit();
        for ch in out.chars() {
            if ch.is_control() {
                prop_assert!(ch == '\n' || ch == '\t');
            }
        }
    }
}
