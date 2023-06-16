use std::collections::HashMap;

use chumsky::{error::Rich, IterParser as _, ParseResult, Parser as _};
use serde::{Deserialize, Serialize};

pub struct Parser<'a, 'b>
where
    'a: 'b,
{
    inner: chumsky::Boxed<'a, 'b, &'a str, AST, chumsky::extra::Err<Rich<'a, char>>>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AST {
    pub module: HashMap<String, ModuleItem>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Ty {
    // TODO: This can be upgraded later to Path
    // ref: https://doc.rust-lang.org/stable/reference/paths.html#paths-in-types
    Simple(String),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FunctionCallExpr {
    pub name: String,
    pub args: Vec<Expr>,
    pub children: Vec<Expr>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct BlockExpr {
    pub statements: Vec<Statement>,
    pub return_expression: Option<Expr>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Expr {
    StringLiteral(String),
    FunctionCall(FunctionCallExpr),
    // TODO: Handle BlockExpr
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Statement {
    ExprStatement(Expr),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Function {
    pub name: String,
    pub inputs: (),
    pub output: Ty,
    pub body: BlockExpr,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum ModuleItem {
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
        let mut statement_parser =
            Recursive::<Indirect<&str, Statement, chumsky::extra::Err<Rich<'a, char>>>>::declare();

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

        let block_parser = just("{")
            .ignore_then(
                statement_parser
                    .clone()
                    .padded()
                    .repeated()
                    .collect::<Vec<Statement>>(),
            )
            .then(expr_parser.clone().padded().or_not())
            .then_ignore(just("}"))
            .map(|(statements, return_expression)| BlockExpr {
                statements,
                return_expression,
            });

        expr_parser.define(choice((function_call_parser, string_literal_expr)));
        statement_parser.define(
            expr_parser
                .then_ignore(just(";"))
                .map(Statement::ExprStatement),
        );

        let fn_parser = just("fn")
            .then_ignore(just(" "))
            .padded()
            .ignore_then(ident())
            .then_ignore(just("()"))
            .then_ignore(just("->").padded())
            .then(ident().padded())
            .then(block_parser)
            .map(|((name, output), body)| {
                ModuleItem::Function(Function {
                    name: name.to_string(),
                    inputs: (),
                    output: Ty::Simple(output.to_string()),
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
