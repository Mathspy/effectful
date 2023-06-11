use parser::{Expr, Function, FunctionCallExpr, ModuleItem, Parser, Ty, AST};

#[test]
fn hello_world() {
    let parser = Parser::new();
    assert_eq!(
        parser
            .parse(
                r#"
fn main() -> Html {
    Html {
        Body {
            Paragraph("Hello, world!")
        }
    }
}
"#
            )
            .into_result(),
        Ok(AST {
            module: [(
                "main",
                ModuleItem::Function(Function {
                    name: "main".to_string(),
                    inputs: (),
                    output: Ty::Html,
                    body: vec![Expr::FunctionCall(FunctionCallExpr {
                        name: "Html".to_string(),
                        children: vec![Expr::FunctionCall(FunctionCallExpr {
                            name: "Body".to_string(),
                            children: vec![Expr::FunctionCall(FunctionCallExpr {
                                name: "Paragraph".to_string(),
                                children: vec![],
                                args: vec![Expr::StringLiteral("Hello, world!".to_string())]
                            })],
                            args: vec![],
                        })],
                        args: vec![],
                    })]
                })
            )]
            .map(|(name, val)| (name.to_string(), val))
            .into_iter()
            .collect()
        })
    )
}
