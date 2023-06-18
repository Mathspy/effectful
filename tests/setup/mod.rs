#[macro_export]
macro_rules! setup {
    ($test_name:ident; $code:expr) => {
        use std::path::Path;

        use pretty_assertions::Comparison;

        #[test]
        fn $test_name() {
            // If the EFF_OVERRIDE_SNAPSHOTS env variable is at all set then we will override even
            // if the .ast files contain invalid ASTs, which happens often during parser development
            let override_snapshots = std::env::var("EFF_OVERRIDE_SNAPSHOTS").ok().is_some();

            let snapshot_file_path = match std::env::current_dir() {
                Ok(current_dir) => current_dir.join(Path::new(file!()).with_extension("html")),
                Err(error) => panic!("Error while detecting current dir: {error}"),
            };
            let snapshot_file_name = snapshot_file_path
                .file_name()
                .expect("file is derived from file!() macro and has a valid filename")
                .to_str()
                .expect("test files names to be valid utf8");

            let snapshot = match std::fs::read_to_string(&snapshot_file_path) {
                Ok(snapshot) => Some(snapshot),
                Err(_) if override_snapshots => None,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => {
                    panic!("Error while opening snapshot file {snapshot_file_name}: {error}")
                }
            };

            let output = effectful::compile($code);

            match (snapshot, override_snapshots) {
                // If the snapshot matches we are done!
                (Some(snapshot), _) if snapshot.eq(&output) => {
                    return;
                }
                // If they don't match, display the pretty diff and crash and we don't wanna override
                (Some(snapshot), false) => {
                    eprintln!("{}", Comparison::new(&snapshot, &output));
                    panic!(
                        "Snapshot mismatched, to override set env var EFF_OVERRIDE_SNAPSHOTS to 1"
                    );
                }
                // If they don't match and we are allowed to override, get write access to file
                (_, true) | (None, _) => {}
            };

            dbg!(&snapshot_file_path);

            match std::fs::write(&snapshot_file_path, &output) {
                Ok(_) => {}
                Err(error) => {
                    panic!("Error while writing to snapshot file {snapshot_file_name}: {error}")
                }
            }
        }
    };
}
