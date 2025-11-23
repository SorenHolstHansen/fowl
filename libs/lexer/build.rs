fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=src/lexing.re");

    let mut version_cmd = std::process::Command::new("re2rust");
    version_cmd.arg("--version");
    match version_cmd.output() {
        Ok(v) => {
            let version_res =
                String::from_utf8(v.stdout).expect("Expected re2rust --version to print a version");
            let version = version_res
                .trim()
                .split(' ')
                .next_back()
                .expect("Expected re2rust --version to print a version");
            if !version.starts_with("4") {
                println!(
                    "cargo::error=re2rust version 4 required, found '{}'",
                    version
                );
                panic!()
            }
        }
        Err(e) => panic!("{e:?}"),
    }

    let mut cmd = std::process::Command::new("re2rust");
    cmd.args([
        "src/lexing.re",
        "--output",
        "src/lexing.rs",
        "--no-unsafe",
        "--start-conditions",
    ]);
    match cmd.output() {
        Ok(_) => {}
        Err(e) => {
            println!("cargo::error={e:?}");
            panic!()
        }
    }
}
