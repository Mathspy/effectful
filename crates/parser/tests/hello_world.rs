mod setup;

#[rustfmt::skip::macros(setup)]

setup!(r#"

fn main() -> Html {
    Html {
        Body {
            Paragraph("Hello, world!")
        }
    }
}

"#);
