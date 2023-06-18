use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use parser::AST;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct Id(u64);

struct Scopes {
    inner: Vec<BTreeMap<Arc<str>, Id>>,
    reverse_map: BTreeMap<Id, Arc<str>>,
}

impl Scopes {
    fn with_prelude() -> Self {
        // TODO: Whacky temporary hack until we implement a proper standard library
        let prelude = [
            (Arc::from("Html"), Id(0xa624256d78ea27e8)),
            (Arc::from("Body"), Id(0xf65ea75ed430d7aa)),
            (Arc::from("Paragraph"), Id(0x829569a9b2c10679)),
            (Arc::from("Console"), Id(0x6f21a62dd1571f6e)),
        ];

        Scopes {
            inner: vec![prelude.clone().into_iter().collect()],
            reverse_map: prelude.into_iter().map(|(ident, id)| (id, ident)).collect(),
        }
    }

    fn new_id(&mut self, ident: &str) -> Id {
        let id = Id(rand::random());

        let scope = self.inner.last_mut().expect("there to always be a scope");
        let ident = Arc::from(ident);
        scope.insert(Arc::clone(&ident), id);
        self.reverse_map.insert(id, ident);

        id
    }

    fn get_id(&self, ident: &str) -> Option<Id> {
        self.inner
            .iter()
            .rev()
            .find_map(|scope| scope.get(ident))
            .copied()
    }

    fn new_scope(&mut self) {
        self.inner.push(BTreeMap::new())
    }

    fn pop_scope(&mut self) {
        self.inner.pop();
    }

    fn into_id_map(self) -> BTreeMap<Id, Arc<str>> {
        self.reverse_map
    }
}

#[derive(Debug, PartialEq)]
pub struct Hir {
    pub module: HashMap<Id, ModuleItem>,
    pub id_map: BTreeMap<Id, Arc<str>>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionCallExpr {
    pub name: Id,
    pub args: Vec<Expr>,
    pub children: Vec<Expr>,
}

impl FunctionCallExpr {
    fn lower(scopes: &mut Scopes, call: &parser::FunctionCallExpr) -> Self {
        Self {
            name: scopes
                .get_id(&call.name)
                .expect("function calls to call something in scope"),
            args: call
                .args
                .iter()
                .map(|expr| Expr::lower(scopes, expr))
                .collect(),
            children: call
                .children
                .iter()
                .map(|expr| Expr::lower(scopes, expr))
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BlockExpr {
    pub statements: Vec<Statement>,
    pub return_expression: Option<Expr>,
}

impl BlockExpr {
    fn lower(scopes: &mut Scopes, block: &parser::BlockExpr) -> Self {
        scopes.new_scope();
        let ret = Self {
            statements: block
                .statements
                .iter()
                .map(|statement| Statement::lower(scopes, statement))
                .collect(),
            return_expression: block
                .return_expression
                .iter()
                .map(|expr| Expr::lower(scopes, expr))
                .next(),
        };
        scopes.pop_scope();

        ret
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    StringLiteral(String),
    FunctionCall(FunctionCallExpr),
    // TODO: Handle BlockExpr
}

impl Expr {
    fn lower(scopes: &mut Scopes, expr: &parser::Expr) -> Self {
        match expr {
            parser::Expr::StringLiteral(string) => Expr::StringLiteral(string.clone()),
            parser::Expr::FunctionCall(call) => {
                Expr::FunctionCall(FunctionCallExpr::lower(scopes, call))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    ExprStatement(Expr),
}

impl Statement {
    fn lower(scopes: &mut Scopes, statement: &parser::Statement) -> Self {
        match statement {
            parser::Statement::ExprStatement(expr) => {
                Statement::ExprStatement(Expr::lower(scopes, expr))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionOutput {
    pub ty: Id,
    pub eff: Option<Id>,
}

impl FunctionOutput {
    fn lower(scopes: &Scopes, output: &parser::FunctionOutput) -> Self {
        // TODO: lowering should return results
        let ty = match &output.ty {
            parser::Ty::Simple(ty) => scopes
                .get_id(ty)
                .expect("FunctionOutput to have a ty in scope"),
        };
        let eff = match &output.eff {
            None => None,
            Some(parser::Eff::Simple(eff)) => scopes
                .get_id(eff)
                .expect("FunctionOutput to have an effect in scope")
                .into(),
        };

        Self { ty, eff }
    }
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub inputs: (),
    pub output: FunctionOutput,
    pub body: BlockExpr,
}

#[derive(Debug, PartialEq)]
pub enum ModuleItem {
    Function(Function),
}

impl Hir {
    pub fn lower(ast: &AST) -> Self {
        let mut scopes = Scopes::with_prelude();

        let module = ast
            .module
            .iter()
            .map(|(name, item)| {
                (
                    scopes.new_id(name),
                    match item {
                        parser::ModuleItem::Function(function) => ModuleItem::Function(Function {
                            name: function.name.clone(),
                            inputs: function.inputs,
                            output: FunctionOutput::lower(&scopes, &function.output),
                            body: BlockExpr::lower(&mut scopes, &function.body),
                        }),
                    },
                )
            })
            .collect();

        Self {
            module,
            id_map: scopes.into_id_map(),
        }
    }
}
