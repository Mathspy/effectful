mod setup;

#[rustfmt::skip::macros(setup)]

setup!(empty_html; r#"

fn main() -> Html {
    Html {
    }
}

"#);
