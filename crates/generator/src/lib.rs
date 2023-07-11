#![allow(dead_code)]

mod ecma;
mod html;
mod machination;

use html::{Child, Element, HtmlWriter};
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

        let element = match self.expr_to_html(ret) {
            Child::Element(element) => element,
            Child::Text(_) => unreachable!("We should verify the type is Html not a string"),
        };

        let mut buf = Vec::new();
        let mut writer = HtmlWriter::new(&mut buf);
        writer.write_element(&element).unwrap();
        String::from_utf8(buf).unwrap()
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
    fn expr_to_html(&self, expr: &Expr) -> Child {
        match expr {
            Expr::StringLiteral(string_literal) => Child::Text(string_literal.clone()),
            Expr::FunctionCall(call) => {
                let (tag, children) = Self::html_std(call);
                let children = children
                    .iter()
                    .map(|expr| self.expr_to_html(expr))
                    .collect::<Vec<Child>>();

                Child::Element(Element {
                    name: tag.to_owned(),
                    children,
                })
            }
        }
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}
