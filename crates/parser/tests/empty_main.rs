mod setup;

#[rustfmt::skip::macros(setup)]

setup!(empty_main; r#"

fn main() -> Html {}

"#);

