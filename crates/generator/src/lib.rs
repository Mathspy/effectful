mod ecma;
mod html;
mod machination;

use html::{Child, Element, HtmlWriter};
use parser::{Expr, FunctionCallExpr, ModuleItem, Statement, AST};

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

        let machination = if let Some(effect) = &main.output.eff {
            match effect {
                parser::Eff::Simple(effect) if effect == "Console" => {
                    Some(machination::gen_fns::machination())
                }
                _ => todo!(),
            }
        } else {
            None
        };

        let main_code = main
            .body
            .statements
            .iter()
            .map(|statement| self.stmt_to_js(statement))
            .map(|statement| statement.or_declaration())
            .collect::<Vec<_>>();

        let mut element = match self.expr_to_html(ret) {
            Child::Element(element) => element,
            Child::Text(_) | Child::Script(_) => {
                unreachable!("We should verify the type is Html not a string")
            }
        };

        if let Some(machination) = machination {
            assert!(element.name == "html");
            let body = element.children.iter_mut().find_map(|child| match child {
                Child::Element(element) if element.name == "body" => Some(element),
                _ => None,
            });

            let main_fn = ecma::declare::gen_func(ecma::ident("main"))
                .body(ecma::block(main_code))
                .into_declaration()
                .or_statement();

            match body {
                Some(body) => {
                    body.children.push(Child::Script(ecma::Program {
                        body: vec![main_fn],
                    }));
                    body.children.push(Child::Script(machination));
                }
                None => todo!(),
            }
        }

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

    // TODO: This will eventually need to be rewriting in effectful itself
    fn eff_std(call: &FunctionCallExpr) -> Option<(&'static str, &[Expr])> {
        match &call.name[..] {
            "log" => Some(("Console", &call.args)),
            _ => None,
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

    fn stmt_to_js(&self, statement: &Statement) -> ecma::Statement {
        match statement {
            Statement::ExprStatement(expr) => match expr {
                Expr::StringLiteral(_) => todo!(),
                Expr::FunctionCall(fn_call) => {
                    if let Some((eff, extra)) = Self::eff_std(fn_call) {
                        machination::gen_fns::effect(eff, extra)
                    } else {
                        todo!()
                    }
                }
            },
        }
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}
