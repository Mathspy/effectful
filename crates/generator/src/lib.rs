use parser::{Expr, FunctionCallExpr, ModuleItem, AST};

pub struct Generator;

impl Generator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, ast: &AST) -> String {
        // TODO: better error handling
        let main = ast.module.get("main").expect("missing main function");

        let ModuleItem::Function(main) = main;

        // TODO: Handle main return type not being Html
        // TODO: Handle main not having a last expression
        let ret = main
            .body
            .return_expression
            .as_ref()
            .expect("main must have a return value of Html");

        self.expr_to_html(ret)
    }

    // TODO: This will eventually need to be rewriting in effectful itself and be completed
    fn html_std(call: &FunctionCallExpr) -> (&'static str, &[Expr]) {
        match &call.name[..] {
            "Html" => ("html", &call.children),
            "Body" => ("body", &call.children),
            "Paragraph" => ("p", &call.args[..1]),
            _ => todo!(),
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn expr_to_html(&self, expr: &Expr) -> String {
        match expr {
            Expr::StringLiteral(string_literal) => string_literal.clone(),
            Expr::FunctionCall(call) => {
                let (tag, children) = Self::html_std(call);
                let children = children
                    .iter()
                    .map(|expr| self.expr_to_html(expr))
                    .collect::<String>();

                format!("<{tag}>{children}</{tag}>")
            }
        }
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}
