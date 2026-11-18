//! The BBEOD28 Python fixture (feature `fixtures`, for tests): a pytest
//! root holding the package `shop` (`cart.total` calls `math_ops.add`
//! across modules), three tests (pass, fail, skip), a folder `with_venv`
//! with a fake `.venv` marker and a folder `plain` with none. When
//! [`Shape::Broken`], `shop/broken.py` holds one syntax error.

use std::path::Path;

/// Whether every file compiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Everything compiles.
    Clean,
    /// `shop/broken.py` line [`BROKEN_LINE`] has a syntax error.
    Broken,
}

/// The broken file, relative to the root.
pub const BROKEN_FILE: &str = "shop/broken.py";
/// The 1-based line of the syntax error.
pub const BROKEN_LINE: u32 = 2;
/// The file that defines `add`.
pub const MATH_FILE: &str = "shop/math_ops.py";
/// The 1-based line where `shop/math_ops.py` defines `add`.
pub const ADD_LINE: u32 = 1;
/// The file that calls `add`.
pub const CART_FILE: &str = "shop/cart.py";
/// The 1-based line where `shop/cart.py` calls `add`.
pub const CALL_LINE: u32 = 5;
/// The 0-based column of `add` in that call.
pub const CALL_COLUMN: u32 = 11;
/// The test file.
pub const TEST_FILE: &str = "tests/test_cart.py";
/// The node ids `--collect-only -q` lists, in file order.
pub const TESTS: [&str; 3] = [
    "tests/test_cart.py::test_passes",
    "tests/test_cart.py::test_fails",
    "tests/test_cart.py::test_skipped",
];
/// The folder with a fake `.venv` (its `pyvenv.cfg` marker only).
pub const VENV_FOLDER: &str = "with_venv";
/// The folder with no venv.
pub const PLAIN_FOLDER: &str = "plain";

const PYPROJECT: &str = "[project]\nname = \"shop\"\nversion = \"0.1.0\"\n";
const MATH: &str = "def add(a: int, b: int) -> int:\n    return a + b\n";
const CART: &str = "from shop.math_ops import add\n\n\ndef total() -> int:\n    return add(1, 2)\n";
const TEST: &str = "import pytest\n\nfrom shop.cart import total\n\n\n\
def test_passes() -> None:\n    assert total() == 3\n\n\n\
def test_fails() -> None:\n    assert total() == 4, \"total is not four\"\n\n\n\
@pytest.mark.skip(reason=\"not today\")\ndef test_skipped() -> None:\n    pass\n";
const BROKEN: &str = "def fine() -> int:\n    return = 1\n";
const VENV_CFG: &str = "home = /usr/bin\nversion = 3.9.6\n";

/// Write the fixture into `root` (an empty folder).
///
/// # Errors
/// A folder or file cannot be written.
pub fn write(root: &Path, shape: Shape) -> std::io::Result<()> {
    let mut files: Vec<(&str, &str)> = vec![
        ("pyproject.toml", PYPROJECT),
        ("shop/__init__.py", ""),
        (MATH_FILE, MATH),
        (CART_FILE, CART),
        (TEST_FILE, TEST),
        ("with_venv/.venv/pyvenv.cfg", VENV_CFG),
        ("with_venv/app.py", "print('venv folder')\n"),
        ("plain/tool.py", "print('plain folder')\n"),
    ];
    match shape {
        Shape::Clean => {}
        Shape::Broken => files.push((BROKEN_FILE, BROKEN)),
    }
    for (file, text) in files {
        let path: std::path::PathBuf = root.join(file);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, text)?;
    }
    Ok(())
}
