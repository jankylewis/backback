//! The BBEOD27 Rust fixture (feature `fixtures`, for tests): a Cargo
//! workspace of two members. `alpha` calls `math::add` across files and has
//! three tests (pass, fail, ignored); `beta` uses `alpha` and, when
//! [`Shape::Broken`], holds one compile error.

use std::path::Path;

/// Whether `beta` compiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Everything compiles.
    Clean,
    /// `beta/src/lib.rs` line [`BROKEN_LINE`] returns a string as `i32`.
    Broken,
}

/// The 1-based line of the compile error in `beta/src/lib.rs`.
pub const BROKEN_LINE: u32 = 4;
/// The 1-based line where `alpha/src/math.rs` defines `add`.
pub const ADD_LINE: u32 = 2;
/// The 1-based line where `alpha/src/lib.rs` calls `add`.
pub const CALL_LINE: u32 = 5;
/// The tests `cargo test -p alpha -- --list` names, sorted.
pub const TESTS: [&str; 3] = ["tests::fails", "tests::ignored", "tests::passes"];

const ROOT: &str = "[workspace]\nmembers = [\"alpha\", \"beta\"]\nresolver = \"2\"\n";
const ALPHA: &str = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const BETA: &str = "[package]\nname = \"beta\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
[dependencies]\nalpha = { path = \"../alpha\" }\n";
const MATH: &str = "/// Adds two numbers.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
const ALPHA_LIB: &str =
    "pub mod math;\n\n/// One plus two, through `math`.\npub fn total() -> i32 {\n    \
math::add(1, 2)\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn passes() {\n        \
assert_eq!(super::total(), 3);\n    }\n\n    #[test]\n    fn fails() {\n        \
assert_eq!(super::total(), 4, \"total is not four\");\n    }\n\n    #[test]\n    #[ignore]\n    \
fn ignored() {}\n}\n";
const BETA_CLEAN: &str = "/// Twice alpha's total.\npub fn twice() -> i32 {\n    let total: i32 = \
alpha::total();\n    total * 2\n}\n";
const BETA_BROKEN: &str =
    "/// Twice alpha's total.\npub fn twice() -> i32 {\n    let total: i32 = \
alpha::total();\n    \"not a number\"\n}\n";

/// Write the workspace into `root` (an empty folder).
///
/// # Errors
/// A folder or file cannot be written.
pub fn write(root: &Path, shape: Shape) -> std::io::Result<()> {
    let beta: &str = match shape {
        Shape::Clean => BETA_CLEAN,
        Shape::Broken => BETA_BROKEN,
    };
    let files: [(&str, &str); 6] = [
        ("Cargo.toml", ROOT),
        ("alpha/Cargo.toml", ALPHA),
        ("alpha/src/lib.rs", ALPHA_LIB),
        ("alpha/src/math.rs", MATH),
        ("beta/Cargo.toml", BETA),
        ("beta/src/lib.rs", beta),
    ];
    for (file, text) in files {
        let path: std::path::PathBuf = root.join(file);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, text)?;
    }
    Ok(())
}
