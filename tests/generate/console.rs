#[rustfmt::skip::macros(setup)]

crate::setup!(console; r#"

fn main() -> Html eff Console {
	log("Hello");
	log("World");
	
    Html {
        Body {
            Paragraph("Hello, world!")
        }
    }
}

"#);
