mod setup;

#[rustfmt::skip::macros(setup)]

setup!(hello_world; r#"

fn main() -> String {
    "I am a string!";
    "Oh look at me, I am another string";

    "sadly neither of them will lead to change :<"
}

"#);
