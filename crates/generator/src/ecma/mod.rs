pub mod writer;

pub struct Program {
    pub body: Vec<StatementOrDeclaration>,
}

pub enum StatementOrDeclaration {
    Statement(Statement),
    Declaration(Declaration),
}

pub enum Statement {
    Block(BlockStatement),
    While(WhileStatement),
    If(IfStatement),
    Break(BreakStatement),
    Expression(ExpressionStatement),
}

impl Statement {
    pub fn or_declaration(self) -> StatementOrDeclaration {
        StatementOrDeclaration::Statement(self)
    }
}

pub mod statements {
    use super::{BlockStatement, BreakStatement, Expression, IfStatement, WhileStatement};

    pub struct IfStatementBuilder {
        test: Expression,
    }

    impl IfStatementBuilder {
        pub fn body(self, body: BlockStatement) -> IfStatement {
            IfStatement {
                test: self.test,
                consequent: body,
                alternate: None,
            }
        }
    }

    pub fn if_statement(test: Expression) -> IfStatementBuilder {
        IfStatementBuilder { test }
    }

    pub fn break_statement() -> BreakStatement {
        BreakStatement
    }

    pub struct WhileStatementBuilder {
        test: Expression,
    }

    impl WhileStatementBuilder {
        pub fn body(self, body: BlockStatement) -> WhileStatement {
            WhileStatement {
                test: self.test,
                body,
            }
        }
    }

    pub fn while_statement(test: Expression) -> WhileStatementBuilder {
        WhileStatementBuilder { test }
    }
}

pub use statements::{break_statement, if_statement, while_statement};

pub struct BlockStatement {
    pub body: Vec<StatementOrDeclaration>,
}

pub fn block(body: Vec<StatementOrDeclaration>) -> BlockStatement {
    BlockStatement { body }
}

pub struct WhileStatement {
    pub test: Expression,
    pub body: BlockStatement,
}

impl WhileStatement {
    pub fn into_statement(self) -> Statement {
        Statement::While(self)
    }
}

pub struct IfStatement {
    pub test: Expression,
    pub consequent: BlockStatement,
    pub alternate: Option<BlockStatement>,
}

impl IfStatement {
    pub fn into_statement(self) -> Statement {
        Statement::If(self)
    }
}

pub struct BreakStatement;

impl BreakStatement {
    pub fn into_statement(self) -> Statement {
        Statement::Break(self)
    }
}

pub struct ExpressionStatement(pub Expression);

pub enum Declaration {
    Variable(VariableDeclaration),
}

impl Declaration {
    pub fn or_statement(self) -> StatementOrDeclaration {
        StatementOrDeclaration::Declaration(self)
    }
}

pub struct VariableDeclaration {
    pub kind: VariableDeclarationKind,
    pub declarations: Vec<VariableDeclarator>,
}

impl VariableDeclaration {
    pub fn into_declaration(self) -> Declaration {
        Declaration::Variable(self)
    }
}

pub enum VariableDeclarationKind {
    Var,
    Let,
    Const,
}

pub struct VariableDeclarator {
    pub id: Pattern,
    pub init: Expression,
}

pub mod declare {
    use super::{
        Expression, Pattern, VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
    };

    pub struct VariableDeclarationBuilder {
        kind: VariableDeclarationKind,
    }

    impl VariableDeclarationBuilder {
        pub fn id(self, id: Pattern) -> VariableDeclarationBuilderWithId {
            VariableDeclarationBuilderWithId {
                kind: self.kind,
                id,
            }
        }
    }

    pub struct VariableDeclarationBuilderWithId {
        kind: VariableDeclarationKind,
        id: Pattern,
    }

    impl VariableDeclarationBuilderWithId {
        pub fn init(self, init: Expression) -> VariableDeclaration {
            VariableDeclaration {
                kind: self.kind,
                declarations: vec![VariableDeclarator { id: self.id, init }],
            }
        }
    }

    pub fn constant() -> VariableDeclarationBuilder {
        VariableDeclarationBuilder {
            kind: VariableDeclarationKind::Const,
        }
    }

    pub fn variable() -> VariableDeclarationBuilder {
        VariableDeclarationBuilder {
            kind: VariableDeclarationKind::Let,
        }
    }
}

pub enum Pattern {
    Ident(Identifier),
    ObjectPattern(ObjectPattern),
}

pub struct Identifier(pub String);

pub fn ident(id: &str) -> Identifier {
    Identifier(id.to_owned())
}

impl Identifier {
    pub fn member_access(self, member: &str) -> MemberExpression {
        MemberExpression::StaticMemberExpression(StaticMemberExpression {
            object: Box::new(Expression::Ident(self)),
            property: Identifier(member.to_owned()),
        })
    }

    pub fn dyn_member_access(self, member: Expression) -> MemberExpression {
        MemberExpression::ComputedMemberExpression(ComputedMemberExpression {
            object: Box::new(Expression::Ident(self)),
            property: Box::new(member),
        })
    }

    pub fn into_expression(self) -> Expression {
        Expression::Ident(self)
    }

    pub fn into_pattern(self) -> Pattern {
        Pattern::Ident(self)
    }

    pub fn call(self, arguments: Vec<Expression>) -> CallExpression {
        CallExpression {
            callee: self.into_expression().boxed(),
            arguments,
        }
    }
}

pub struct ObjectPattern {
    pub properties: Vec<ObjectPatternProperty>,
    // TODO:
    pub rest: Option<()>,
}

// TODO: Should we return an ObjectPattern instead and add a .into_pattern function to it?
pub fn obj_pat(props: Vec<(&str, Option<&str>)>) -> Pattern {
    Pattern::ObjectPattern(ObjectPattern {
        properties: props
            .into_iter()
            .map(|(key, value)| ObjectPatternProperty {
                key: ident(key),
                value: value.map(ident),
            })
            .collect(),
        rest: None,
    })
}

pub struct ObjectPatternProperty {
    pub key: Identifier,
    pub value: Option<Identifier>,
}

pub enum Expression {
    Ident(Identifier),
    Call(CallExpression),
    ArrowFunction(ArrowFunctionExpression),
    Literal(LiteralExpression),
    Member(MemberExpression),
    Binary(BinaryExpression),
    Yield(YieldExpression),
    Object(ObjectExpression),
}

impl Expression {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

    pub fn into_statement(self) -> Statement {
        Statement::Expression(ExpressionStatement(self))
    }

    pub fn call(self, arguments: Vec<Expression>) -> CallExpression {
        CallExpression {
            callee: self.boxed(),
            arguments,
        }
    }

    pub fn strict_eq(self, right: Expression) -> Expression {
        Expression::Binary(BinaryExpression {
            left: Box::new(self),
            operator: BinaryOperator::StrictEqual,
            right: Box::new(right),
        })
    }
}

pub struct CallExpression {
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
}

impl CallExpression {
    pub fn into_expression(self) -> Expression {
        Expression::Call(self)
    }

    pub fn into_statement(self) -> Statement {
        Statement::Expression(ExpressionStatement(self.into_expression()))
    }
}

pub struct ArrowFunctionExpression {
    // TODO:
    pub params: Vec<()>,
    pub body: BlockStatement,
}

pub enum LiteralExpression {
    Boolean(BooleanLiteral),
    String(StringLiteral),
    Number(NumberLiteral),
}

pub enum MemberExpression {
    StaticMemberExpression(StaticMemberExpression),
    ComputedMemberExpression(ComputedMemberExpression),
}

impl MemberExpression {
    pub fn into_expression(self) -> Expression {
        Expression::Member(self)
    }

    pub fn call(self, arguments: Vec<Expression>) -> CallExpression {
        CallExpression {
            callee: self.into_expression().boxed(),
            arguments,
        }
    }

    pub fn dyn_member_access(self, member: Expression) -> MemberExpression {
        MemberExpression::ComputedMemberExpression(ComputedMemberExpression {
            object: Box::new(Expression::Member(self)),
            property: Box::new(member),
        })
    }
}

pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

pub enum BinaryOperator {
    StrictEqual,
}

pub struct StaticMemberExpression {
    pub object: Box<Expression>,
    pub property: Identifier,
}

pub struct ComputedMemberExpression {
    pub object: Box<Expression>,
    pub property: Box<Expression>,
}

pub enum BooleanLiteral {
    False,
    True,
}

impl BooleanLiteral {
    pub fn into_expression(self) -> Expression {
        Expression::Literal(LiteralExpression::Boolean(self))
    }
}

pub fn boolean(b: bool) -> BooleanLiteral {
    if b {
        BooleanLiteral::True
    } else {
        BooleanLiteral::False
    }
}

pub struct StringLiteral(pub String);

pub fn string(s: &str) -> StringLiteral {
    StringLiteral(s.to_owned())
}

impl StringLiteral {
    pub fn into_expression(self) -> Expression {
        Expression::Literal(LiteralExpression::String(self))
    }
}

pub enum NumberLiteral {
    Integer(u32),
    Float(f64),
}

pub fn int(n: u32) -> NumberLiteral {
    NumberLiteral::Integer(n)
}

impl NumberLiteral {
    pub fn into_expression(self) -> Expression {
        Expression::Literal(LiteralExpression::Number(self))
    }
}

pub struct YieldExpression {
    pub argument: Box<Expression>,
}

pub fn yield_(argument: Expression) -> YieldExpression {
    YieldExpression {
        argument: argument.boxed(),
    }
}

impl YieldExpression {
    pub fn into_statement(self) -> Statement {
        Statement::Expression(ExpressionStatement(Expression::Yield(self)))
    }
}

pub struct ObjectExpression {
    pub properties: Vec<ObjectProperty>,
}

pub struct ObjectProperty {
    key: Identifier,
    value: Option<Expression>,
}

pub fn obj(props: Vec<(&str, Option<Expression>)>) -> ObjectExpression {
    ObjectExpression {
        properties: props
            .into_iter()
            .map(|(key, value)| ObjectProperty {
                key: ident(key),
                value,
            })
            .collect(),
    }
}

impl ObjectExpression {
    pub fn into_expression(self) -> Expression {
        Expression::Object(self)
    }
}
