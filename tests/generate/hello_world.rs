#[rustfmt::skip::macros(setup)]

crate::setup!(hello_world; r#"

fn main() -> Html {
    Html {
        Body {
            Paragraph("Hello, world!")
        }
    }
}

"#);
