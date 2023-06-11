use std::collections::HashMap;

use chumsky::{error::Rich, IterParser as _, ParseResult, Parser as _};

pub struct Parser<'a, 'b>
where
    'a: 'b,
{
    inner: chumsky::Boxed<'a, 'b, &'a str, AST, chumsky::extra::Err<Rich<'a, char>>>,
}

#[derive(Debug, PartialEq)]
pub struct AST {
    module: HashMap<String, ModuleItem>,
}

#[derive(Debug, PartialEq)]
enum Ty {
    Html,
}

#[derive(Debug, PartialEq)]
struct FunctionCallExpr {
    name: String,
    args: Vec<Expr>,
    children: Vec<Expr>,
}

#[derive(Debug, PartialEq)]
enum Expr {
    StringLiteral(String),
    FunctionCall(FunctionCallExpr),
}

#[derive(Debug, PartialEq)]
struct Function {
    name: String,
    inputs: (),
    output: Ty,
    body: Vec<Expr>,
}

#[derive(Debug, PartialEq)]
enum ModuleItem {
    Function(Function),
}

impl<'a, 'b> Parser<'a, 'b> {
    pub fn new() -> Self {
        use chumsky::{
            primitive::{choice, end, just, none_of},
            recursive::{Indirect, Recursive},
            text::ident,
        };

        let mut expr_parser =
            Recursive::<Indirect<&str, Expr, chumsky::extra::Err<Rich<'a, char>>>>::declare();

        let vec_expr_parser = expr_parser.clone().repeated().collect::<Vec<Expr>>();

        let string_literal_expr = none_of("\\\"")
            .ignored()
            .repeated()
            .slice()
            .map(ToString::to_string)
            .map(Expr::StringLiteral)
            .delimited_by(just('"'), just('"'));

        let function_call_parser = ident()
            .padded()
            .then(
                expr_parser
                    .clone()
                    .delimited_by(just("("), just(")"))
                    .separated_by(just(","))
                    .collect::<Vec<Expr>>()
                    .padded(),
            )
            .then(
                expr_parser
                    .clone()
                    .separated_by(just(","))
                    .collect::<Vec<Expr>>()
                    .delimited_by(just("{").padded(), just("}").padded())
                    .or_not(),
            )
            .map(|((name, args), children)| {
                Expr::FunctionCall(FunctionCallExpr {
                    name: name.to_string(),
                    args,
                    children: children.unwrap_or_default(),
                })
            });

        expr_parser.define(choice((function_call_parser, string_literal_expr)));

        let fn_parser = just("fn")
            .then_ignore(just(" "))
            .padded()
            .ignore_then(ident())
            .then_ignore(just("()"))
            .then_ignore(just("->").padded())
            .then(ident().padded())
            .then(vec_expr_parser.delimited_by(just("{").padded(), just("}").padded()))
            .map(|((name, output), body)| {
                ModuleItem::Function(Function {
                    name: name.to_string(),
                    inputs: (),
                    output: if output == "Html" { Ty::Html } else { todo!() },
                    body,
                })
            })
            .padded()
            .then_ignore(end())
            .map(|module_item| AST {
                module: [("main".to_string(), module_item)].into_iter().collect(),
            })
            .boxed();

        Parser { inner: fn_parser }
    }

    pub fn parse(&self, file: &'a str) -> ParseResult<AST, Rich<char>> {
        let ret = self.inner.parse(file);

        // TODO: For initial debugging purposes only
        ret.errors().for_each(|error| {
            let span = error.span();
            let start = span.start.saturating_sub(5);
            let end = span.end.max(file.len());

            eprintln!("Error at: {}", &file[start..end]);
        });

        ret
    }
}

impl<'a, 'b> Default for Parser<'a, 'b> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Expr, FunctionCallExpr, ModuleItem, Parser, Ty, AST};

    #[test]
    fn empty_main() {
        let parser = Parser::new();
        assert_eq!(
            parser
                .parse(
                    r#"
fn main() -> Html {}
"#
                )
                .into_result(),
            Ok(AST {
                module: [(
                    "main",
                    ModuleItem::Function(crate::Function {
                        name: "main".to_string(),
                        inputs: (),
                        output: Ty::Html,
                        body: Vec::new()
                    })
                )]
                .map(|(name, val)| (name.to_string(), val))
                .into_iter()
                .collect()
            })
        )
    }

    #[test]
    fn empty_html() {
        let parser = Parser::new();
        assert_eq!(
            parser
                .parse(
                    r#"
fn main() -> Html {
    Html {
    }
}
"#
                )
                .into_result(),
            Ok(AST {
                module: [(
                    "main",
                    ModuleItem::Function(crate::Function {
                        name: "main".to_string(),
                        inputs: (),
                        output: Ty::Html,
                        body: vec![Expr::FunctionCall(FunctionCallExpr {
                            name: "Html".to_string(),
                            children: Vec::new(),
                            args: Vec::new(),
                        })]
                    })
                )]
                .map(|(name, val)| (name.to_string(), val))
                .into_iter()
                .collect()
            })
        )
    }

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
                    ModuleItem::Function(crate::Function {
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
}
