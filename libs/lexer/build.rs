fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=src/lexing.re");
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
