use chumsky::{error::Rich, Parser as _};

pub struct Parser<'a, 'b>
where
    'a: 'b,
{
    inner: chumsky::Boxed<'a, 'b, &'a str, AST, chumsky::extra::Err<Rich<'a, char>>>,
}

#[derive(Debug, PartialEq)]
pub struct AST;

impl<'a, 'b> Parser<'a, 'b> {
    pub fn new() -> Self {
        Parser {
            inner: chumsky::primitive::todo().boxed(),
        }
    }

    pub fn parse(&self, _file: &str) -> AST {
        AST
    }
}

impl<'a, 'b> Default for Parser<'a, 'b> {
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
