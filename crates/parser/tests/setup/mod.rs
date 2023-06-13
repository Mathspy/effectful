#[macro_export]
macro_rules! setup {
    ($code:expr) => {
        use std::path::Path;

        use parser::{Parser, AST};

        use pretty_assertions::Comparison;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        pub enum RichPattern {
            Token(char),
            Label(String),
            EndOfInput,
        }

        impl RichPattern {
            fn from_chumsky<'a>(chumsky: chumsky::error::RichPattern<'a, char>) -> Self {
                use chumsky::{error::RichPattern, util::Maybe};

                match chumsky {
                    RichPattern::Token(Maybe::Ref(&c)) => Self::Token(c),
                    RichPattern::Token(Maybe::Val(c)) => Self::Token(c),
                    RichPattern::Label(label) => Self::Label(label.to_owned()),
                    RichPattern::EndOfInput => Self::EndOfInput,
                }
            }
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        pub enum RichReason {
            ExpectedFound {
                expected: Vec<RichPattern>,
                found: Option<char>,
            },
            Custom(String),
            Many(Vec<Self>),
        }

        impl RichReason {
            fn from_chumsky<'a>(chumsky: chumsky::error::RichReason<'a, char>) -> Self {
                use chumsky::{error::RichReason, util::Maybe};

                match chumsky {
                    RichReason::ExpectedFound { expected, found } => Self::ExpectedFound {
                        expected: expected
                            .into_iter()
                            .map(RichPattern::from_chumsky)
                            .collect(),
                        found: found.map(|c| match c {
                            Maybe::Ref(&c) => c,
                            Maybe::Val(c) => c,
                        }),
                    },
                    RichReason::Custom(string) => Self::Custom(string),
                    RichReason::Many(many) => {
                        Self::Many(many.into_iter().map(Self::from_chumsky).collect())
                    }
                }
            }
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct SimpleSpan {
            start: usize,
            end: usize,
        }

        impl SimpleSpan {
            fn from_chumsky(chumsky: chumsky::span::SimpleSpan<usize>) -> Self {
                Self {
                    start: chumsky.start,
                    end: chumsky.end,
                }
            }
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Rich {
            span: SimpleSpan,
            reason: RichReason,
        }

        impl Rich {
            fn from_chumsky<'a>(chumsky: chumsky::error::Rich<'a, char>) -> Self {
                Self {
                    span: SimpleSpan::from_chumsky(*chumsky.span()),
                    reason: RichReason::from_chumsky(chumsky.into_reason()),
                }
            }
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct ParseResult {
            output: Option<AST>,
            errors: Vec<Rich>,
        }

        #[test]
        fn hello_world() {
            // TODO: We get the test path via a hacky solution that relies on this test being in
            // crates/crate_name/tests/folder because file!() does not interact very nicely with
            // env::current_dir()
            //
            // current_dir() == "~/Coding/effectful/crates/parser"
            // file!() == "crates/parser/tests/hello_world.rs"
            //
            // So if this was in crate_root/tests we'd need to not go up two directories because the
            // values will be more sane
            //
            // current_dir() == "~/Coding/effectful"
            // file!() == "tests/hello_world.rs"
            //
            // There's an open issue for this on rust-lang/cargo
            // See: https://github.com/rust-lang/cargo/issues/3946
            let snapshot_file_path = match std::env::current_dir() {
                Ok(current_dir) => current_dir
                    .join("../../")
                    .join(Path::new(file!()).with_extension("ast")),
                Err(error) => panic!("Error while detecting current dir: {error}"),
            };
            let snapshot_file_name = snapshot_file_path
                .file_name()
                .expect("file is derived from file!() macro and has a valid filename")
                .to_str()
                .expect("test files names to be valid utf8");

            let snapshot_file = std::fs::File::options()
                .read(true)
                .open(&snapshot_file_path);

            let snapshot = match snapshot_file {
                Ok(snapshot) => match serde_json::from_reader::<_, ParseResult>(snapshot) {
                    Ok(snapshot) => Some(snapshot),
                    Err(error) => panic!("Error while parsing snapshot file {snapshot_file_name}: {error}"),
                },
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => panic!("Error while opening snapshot file {snapshot_file_name}: {error}"),
            };

            let parser = Parser::new();
            let ast = parser.parse($code);

            let (output, errors) = ast.into_output_errors();

            let result = ParseResult {
                output,
                errors: errors.into_iter().map(Rich::from_chumsky).collect(),
            };

            // If the EFF_OVERRIDE_SNAPSHOTS  env variable is at all set then we will override
            let override_snapshots = std::env::var("EFF_OVERRIDE_SNAPSHOTS").ok().is_some();
            let snapshot_file = match (snapshot, override_snapshots) {
                // If the snapshot matches we are done!
                (Some(snapshot), _) if snapshot.eq(&result) => {
                    return;
                }
                // If they don't match, display the pretty diff and crash and we don't wanna override
                (Some(snapshot), false) => {
                    eprintln!("{}", Comparison::new(&snapshot, &result));
                    panic!("Snapshot mismatched, to override set env var EFF_OVERRIDE_SNAPSHOTS to 1");
                }
                // If they don't match and we are allowed to override, get write access to file
                (Some(_), true) => std::fs::File::options()
                    .write(true)
                    .truncate(true)
                    .open(&snapshot_file_path),
                // If it doesn't exist, then write it
                (None, _) => std::fs::File::options()
                    .write(true)
                    .create_new(true)
                    .open(&snapshot_file_path),
            };

            let snapshot_file = match snapshot_file {
                Ok(snapshot_file) => snapshot_file,
                Err(error) => {
                    panic!("Error while creating snapshot file {snapshot_file_name}: {error}")
                }
            };

            match serde_json::to_writer_pretty(snapshot_file, &result) {
                Ok(_) => {}
                Err(error) => {
                    panic!("Error while writing to snapshot file {snapshot_file_name}: {error}")
                }
            }
        }
    }
}
