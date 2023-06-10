pub struct Parser;

#[derive(Debug, PartialEq)]
pub struct AST;

impl Parser {
    pub fn new() -> Self {
        Parser
    }

    pub fn parse(&self, _file: &str) -> AST {
        AST
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Parser, AST};

    #[test]
    fn can_parse_a_simple_main() {
        let parser = Parser::new();
        assert_eq!(
            parser.parse(
                r#"
fn main() -> Html {
    <html>
        <body>

        </body>
    </html>
}
"#
            ),
            AST
        )
    }
}
