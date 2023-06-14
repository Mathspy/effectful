#[test]
fn hello_world() {
    assert_eq!(
        effectful::compile(
            r#"
    fn main() -> Html {
        Html {
            Body {
                Paragraph("Hello, world!")
            }
        }
    }
    "#,
        ),
        "<html><body><p>Hello, world!</p></body></html>"
    );
}
