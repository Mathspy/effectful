use itertools::{Itertools, Position};
use std::io;

use super::{
    ArrowFunctionExpression, BinaryExpression, BinaryOperator, BlockStatement, BooleanLiteral,
    BreakStatement, CallExpression, ComputedMemberExpression, Declaration, Expression,
    ExpressionStatement, Identifier, IfStatement, LiteralExpression, MemberExpression,
    NumberLiteral, ObjectPattern, ObjectPatternProperty, Pattern, Program, Statement,
    StatementOrDeclaration, StaticMemberExpression, StringLiteral, VariableDeclaration,
    VariableDeclarationKind, VariableDeclarator, WhileStatement,
};

pub struct EcmaWriter<W> {
    writer: W,
}

impl<W> EcmaWriter<W>
where
    W: io::Write,
{
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_program(&mut self, program: &Program) -> io::Result<usize> {
        program
            .body
            .iter()
            .map(|statement_or_declaration| {
                self.write_statement_or_declaration(statement_or_declaration)
            })
            .sum::<io::Result<usize>>()
    }

    fn write_statement_or_declaration(
        &mut self,
        statement_or_declaration: &StatementOrDeclaration,
    ) -> io::Result<usize> {
        match statement_or_declaration {
            StatementOrDeclaration::Statement(statement) => self.write_statement(statement),
            StatementOrDeclaration::Declaration(declaration) => self.write_declaration(declaration),
        }
    }

    fn write_statement(&mut self, statement: &Statement) -> io::Result<usize> {
        match statement {
            Statement::Block(block_statement) => self.write_block_statement(block_statement),
            Statement::While(while_statement) => self.write_while_statement(while_statement),
            Statement::If(if_statement) => self.write_if_statement(if_statement),
            Statement::Break(break_statement) => self.write_break_statement(break_statement),
            Statement::Expression(expression_statement) => {
                self.write_expression_statement(expression_statement)
            }
        }
    }

    fn write_block_statement(&mut self, block_statement: &BlockStatement) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.writer.write(b"{")?;
        bytes_written += block_statement
            .body
            .iter()
            .map(|statement_or_declaration| {
                self.write_statement_or_declaration(statement_or_declaration)
            })
            .sum::<io::Result<usize>>()?;
        bytes_written += self.writer.write(b"}")?;

        Ok(bytes_written)
    }

    fn write_while_statement(&mut self, while_statement: &WhileStatement) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.writer.write(b"while(")?;
        bytes_written += self.write_expression(&while_statement.test)?;
        bytes_written += self.writer.write(b")")?;
        bytes_written += self.write_block_statement(&while_statement.body)?;

        Ok(bytes_written)
    }

    fn write_if_statement(&mut self, if_statement: &IfStatement) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.writer.write(b"if(")?;
        bytes_written += self.write_expression(&if_statement.test)?;
        bytes_written += self.writer.write(b")")?;
        bytes_written += self.write_block_statement(&if_statement.consequent)?;
        if let Some(alternate) = &if_statement.alternate {
            bytes_written += self.writer.write(b"else")?;
            bytes_written += self.write_block_statement(alternate)?;
        }

        Ok(bytes_written)
    }

    fn write_break_statement(&mut self, _break_statement: &BreakStatement) -> io::Result<usize> {
        self.writer.write(b"break;")
    }

    fn write_expression_statement(
        &mut self,
        expression_statement: &ExpressionStatement,
    ) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.write_expression(&expression_statement.0)?;
        bytes_written += self.writer.write(b";")?;

        Ok(bytes_written)
    }

    fn write_declaration(&mut self, declaration: &Declaration) -> io::Result<usize> {
        match declaration {
            Declaration::Variable(variable_declaration) => {
                self.write_variable_declaration(variable_declaration)
            }
        }
    }

    fn write_variable_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
    ) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += match variable_declaration.kind {
            VariableDeclarationKind::Var => self.writer.write(b"var ")?,
            VariableDeclarationKind::Let => self.writer.write(b"let ")?,
            VariableDeclarationKind::Const => self.writer.write(b"const ")?,
        };

        bytes_written += variable_declaration
            .declarations
            .iter()
            .with_position()
            .map(|(position, variable_declarator)| {
                let mut bytes_written = 0;

                bytes_written += self.write_variable_declarator(variable_declarator)?;

                if let Position::Last = position {
                    bytes_written += self.writer.write(b",")?;
                }

                Ok(bytes_written)
            })
            .sum::<io::Result<usize>>()?;

        bytes_written += self.writer.write(b";")?;

        Ok(bytes_written)
    }

    fn write_variable_declarator(
        &mut self,
        variable_declarator: &VariableDeclarator,
    ) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += match &variable_declarator.id {
            Pattern::Ident(identifier) => self.write_identifier(identifier)?,
            Pattern::ObjectPattern(object_pattern) => self.write_object_pattern(object_pattern)?,
        };
        bytes_written += self.writer.write(b"=")?;
        bytes_written += self.write_expression(&variable_declarator.init)?;

        Ok(bytes_written)
    }

    fn write_identifier(&mut self, identifier: &Identifier) -> io::Result<usize> {
        self.writer.write(identifier.0.as_bytes())
    }

    fn write_object_pattern(&mut self, object_pattern: &ObjectPattern) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.writer.write(b"{")?;
        bytes_written += object_pattern
            .properties
            .iter()
            .map(|property| self.write_object_pattern_property(property))
            .sum::<io::Result<usize>>()?;
        bytes_written += self.writer.write(b"}")?;

        Ok(bytes_written)
    }

    fn write_object_pattern_property(
        &mut self,
        property: &ObjectPatternProperty,
    ) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.write_identifier(&property.key)?;
        if let Some(value) = &property.value {
            bytes_written += self.writer.write(b":")?;
            bytes_written += self.write_identifier(value)?;
        }
        bytes_written += self.writer.write(b",")?;

        Ok(bytes_written)
    }

    fn write_expression(&mut self, expression: &Expression) -> io::Result<usize> {
        match expression {
            Expression::Ident(identifier) => self.write_identifier(identifier),
            Expression::Call(call_expression) => self.write_call_expression(call_expression),
            Expression::ArrowFunction(arrow_function_expression) => {
                self.write_arrow_function_expression(arrow_function_expression)
            }
            Expression::Literal(literal_expression) => {
                self.write_literal_expression(literal_expression)
            }
            Expression::Member(member_expression) => {
                self.write_member_expression(member_expression)
            }
            Expression::Binary(binary_expression) => {
                self.write_binary_expression(binary_expression)
            }
        }
    }

    fn write_call_expression(&mut self, call_expression: &CallExpression) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.write_expression(&call_expression.callee)?;
        bytes_written += self.writer.write(b"(")?;
        bytes_written += call_expression
            .arguments
            .iter()
            .map(|argument| {
                let mut bytes_written = 0;

                bytes_written += self.write_expression(argument)?;
                bytes_written += self.writer.write(b",")?;

                Ok(bytes_written)
            })
            .sum::<io::Result<usize>>()?;
        bytes_written += self.writer.write(b")")?;

        Ok(bytes_written)
    }

    fn write_arrow_function_expression(
        &mut self,
        arrow_function_expression: &ArrowFunctionExpression,
    ) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.writer.write(b"(")?;
        // TODO: parameters
        bytes_written += self.writer.write(b")")?;
        bytes_written += self.write_block_statement(&arrow_function_expression.body)?;

        Ok(bytes_written)
    }

    fn write_literal_expression(
        &mut self,
        literal_expression: &LiteralExpression,
    ) -> io::Result<usize> {
        match literal_expression {
            LiteralExpression::Boolean(boolean_literal) => {
                self.write_boolean_literal(boolean_literal)
            }
            LiteralExpression::String(string_literal) => self.write_string_literal(string_literal),
            LiteralExpression::Number(number_literal) => self.write_number_literal(number_literal),
        }
    }

    fn write_boolean_literal(&mut self, boolean_literal: &BooleanLiteral) -> io::Result<usize> {
        match boolean_literal {
            BooleanLiteral::False => self.writer.write(b"false"),
            BooleanLiteral::True => self.writer.write(b"true"),
        }
    }

    fn write_string_literal(&mut self, string_literal: &StringLiteral) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.writer.write(br#"""#)?;
        bytes_written += self.writer.write(string_literal.0.as_bytes())?;
        bytes_written += self.writer.write(br#"""#)?;

        Ok(bytes_written)
    }

    fn write_number_literal(&mut self, number_literal: &NumberLiteral) -> io::Result<usize> {
        match number_literal {
            NumberLiteral::Integer(int) => self.writer.write(int.to_string().as_bytes()),
            NumberLiteral::Float(float) => self.writer.write(float.to_string().as_bytes()),
        }
    }

    fn write_member_expression(
        &mut self,
        member_expression: &MemberExpression,
    ) -> io::Result<usize> {
        match member_expression {
            MemberExpression::StaticMemberExpression(static_member_expression) => {
                self.write_static_member_expression(static_member_expression)
            }
            MemberExpression::ComputedMemberExpression(computed_member_expression) => {
                self.write_computed_member_expression(computed_member_expression)
            }
        }
    }

    fn write_static_member_expression(
        &mut self,
        static_member_expression: &StaticMemberExpression,
    ) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.write_expression(&static_member_expression.object)?;
        bytes_written += self.writer.write(b".")?;
        bytes_written += self.write_identifier(&static_member_expression.property)?;

        Ok(bytes_written)
    }

    fn write_computed_member_expression(
        &mut self,
        computed_member_expression: &ComputedMemberExpression,
    ) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.write_expression(&computed_member_expression.object)?;
        bytes_written += self.writer.write(b"[")?;
        bytes_written += self.write_expression(&computed_member_expression.property)?;
        bytes_written += self.writer.write(b"]")?;

        Ok(bytes_written)
    }

    fn write_binary_expression(
        &mut self,
        binary_expression: &BinaryExpression,
    ) -> io::Result<usize> {
        let mut bytes_written = 0;

        bytes_written += self.write_expression(&binary_expression.left)?;
        bytes_written += match binary_expression.operator {
            BinaryOperator::StrictEqual => self.writer.write(b"===")?,
        };
        bytes_written += self.write_expression(&binary_expression.right)?;

        Ok(bytes_written)
    }
}
