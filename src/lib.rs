use generator::Generator;
use parser::Parser;

pub fn compile(input: &str) -> String {
    let parser = Parser::new();
    // TODO: Handle parser errors properly
    let ast = parser.parse(input).unwrap();
    let generator = Generator::new();

    generator.generate(&ast)
}
