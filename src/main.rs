use std::io::Read;

fn main() {
    let mut buffer = Vec::new();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    handle
        .read_to_end(&mut buffer)
        .expect("unable to read stdin");

    let input = std::str::from_utf8(&buffer).expect("valid utf8 input");

    let output = effectful::compile(input);
    println!("{output}");
}
