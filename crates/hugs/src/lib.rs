mod hir;

use hir::Hir;
use petgraph::graphmap::DiGraphMap;

fn get_function_calls(expr: &hir::Expr) -> Vec<&hir::FunctionCallExpr> {
    match expr {
        hir::Expr::StringLiteral(_) => Vec::new(),
        hir::Expr::FunctionCall(call) => {
            let mut vec = vec![call];

            let from_args = call
                .args
                .iter()
                .flat_map(get_function_calls)
                .collect::<Vec<_>>();
            let from_children = call
                .children
                .iter()
                .flat_map(get_function_calls)
                .collect::<Vec<_>>();

            vec.extend(from_args);
            vec.extend(from_children);

            vec
        }
    }
}

pub fn generate_call_graph(ast: &Hir) -> DiGraphMap<hir::Id, ()> {
    let mut graph = DiGraphMap::new();

    ast.module.iter().for_each(|(name, item)| {
        let hir::ModuleItem::Function(function) = item;

        let function_node = graph.add_node(*name);

        function
            .body
            .statements
            .iter()
            .flat_map(|statement| match statement {
                hir::Statement::ExprStatement(expr) => get_function_calls(expr),
            })
            .for_each(|call| {
                let node = graph.add_node(call.name);
                graph.add_edge(function_node, node, ());
            });

        function
            .body
            .return_expression
            .iter()
            .flat_map(get_function_calls)
            .for_each(|call| {
                let node = graph.add_node(call.name);
                graph.add_edge(function_node, node, ());
            });
    });

    graph
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, sync::Arc};

    use parser::Parser;

    use super::{generate_call_graph, hir::Hir};

    #[test]
    fn hello_world() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                r#"

fn main() -> Html {
    Html {
        Body {
            Paragraph("Hello, world!")
        }
    }
}

"#,
            )
            .into_output()
            .unwrap();

        let hir = Hir::lower(&ast);
        let graph = generate_call_graph(&hir);

        let main = hir
            .id_map
            .iter()
            .find_map(|(id, name)| if &**name == "main" { Some(id) } else { None })
            .copied()
            .expect("there to be a main in HIR id map");

        let neighbors = graph
            .neighbors(main)
            .map(|neighbor| hir.id_map.get(&neighbor).unwrap().clone())
            .collect::<BTreeSet<_>>();

        assert_eq!(
            neighbors,
            ["Html", "Body", "Paragraph"]
                .map(Arc::from)
                .into_iter()
                .collect()
        );
    }
}
