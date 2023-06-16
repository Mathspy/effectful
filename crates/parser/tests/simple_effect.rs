mod setup;

#[rustfmt::skip::macros(setup)]

setup!(hello_world; r#"

fn main() -> Html eff Console {}

"#);
