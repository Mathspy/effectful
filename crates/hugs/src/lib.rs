use parser::{Expr, FunctionCallExpr, AST};
use petgraph::Graph;

fn get_function_calls(expr: &Expr) -> Vec<&FunctionCallExpr> {
    match expr {
        parser::Expr::StringLiteral(_) => Vec::new(),
        parser::Expr::FunctionCall(call) => {
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

pub fn generate_call_graph(ast: &AST) -> Graph<String, ()> {
    let mut graph = Graph::new();

    // TODO: Handle generating graphs for functions besides main
    let main = ast.module.get("main").expect("main is missing");

    let main_node = graph.add_node("main".to_string());

    let parser::ModuleItem::Function(main) = main;

    main.body
        .statements
        .iter()
        .flat_map(|statement| match statement {
            parser::Statement::ExprStatement(expr) => get_function_calls(expr),
        })
        .for_each(|call| {
            let node = graph.add_node(call.name.clone());
            graph.add_edge(main_node, node, ());
        });

    main.body
        .return_expression
        .iter()
        .flat_map(get_function_calls)
        .for_each(|call| {
            let node = graph.add_node(call.name.clone());
            graph.add_edge(main_node, node, ());
        });

    graph
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use parser::{Expr, Function, FunctionCallExpr, ModuleItem, Ty, AST};
    use petgraph::Direction;

    use super::generate_call_graph;

    #[test]
    fn hello_world() {
        let ast = AST {
            module: [(
                "main",
                ModuleItem::Function(Function {
                    name: "main".to_string(),
                    inputs: (),
                    output: Ty::Html,
                    body: parser::BlockExpr {
                        statements: vec![],
                        return_expression: Some(Expr::FunctionCall(FunctionCallExpr {
                            name: "Html".to_string(),
                            children: vec![Expr::FunctionCall(FunctionCallExpr {
                                name: "Body".to_string(),
                                children: vec![Expr::FunctionCall(FunctionCallExpr {
                                    name: "Paragraph".to_string(),
                                    children: vec![],
                                    args: vec![Expr::StringLiteral("Hello, world!".to_string())],
                                })],
                                args: vec![],
                            })],
                            args: vec![],
                        })),
                    },
                }),
            )]
            .map(|(name, val)| (name.to_string(), val))
            .into_iter()
            .collect(),
        };

        let graph = generate_call_graph(&ast);

        let main = graph
            .node_indices()
            .find(|index| graph[*index] == "main")
            .unwrap();

        let neighbors = graph
            .neighbors_directed(main, Direction::Outgoing)
            .map(|index| graph[index].clone())
            .collect::<BTreeSet<_>>();

        assert_eq!(
            neighbors,
            ["Html", "Body", "Paragraph"]
                .map(|name| name.to_string())
                .into_iter()
                .collect()
        );
    }
}
