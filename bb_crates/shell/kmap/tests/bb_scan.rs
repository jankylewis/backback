//! One file's map: purpose, items, signatures, docs, owners, variants, tests.

use bb_kmap::{scan, FileMap, Item, Kind, Vis};

const SRC: &str = r#"//! Parse widgets and keep
//! them small.

use std::fmt;
pub use crate::inner::{A, B};

/// Longest name.
pub const MAX: usize = 12;

/// A widget the user
/// sees. Second sentence.
#[derive(Debug)]
pub struct Widget {
    name: String,
}

/// Colours.
pub enum Colour {
    /// Red.
    Red,
    Green(u8),
    Blue { deep: bool },
}

impl Widget {
    /// Make one.
    pub fn new(name: &str,
               size: [u8; 3]) -> Self {
        let text: &str = "pub fn fake() {";
        fn inner() {}
        Self { name: name.to_owned() }
    }

    fn private_helper(&self) {}
}

impl fmt::Display for Widget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Ok(()) }
}

pub(crate) trait Paint {
    /// Paint it.
    fn paint(&self);
}

#[cfg(test)]
mod tests {
    #[test]
    fn hidden() {}
    #[tokio::test]
    async fn also() {}
}
"#;

#[cfg(test)]
mod helpers {
    use bb_kmap::{FileMap, Item};

    pub fn named<'a>(map: &'a FileMap, name: &str) -> &'a Item {
        map.items.iter().find(|i: &&Item| i.name == name).unwrap()
    }
}

use helpers::named;

#[test]
fn the_purpose_joins_doc_lines_until_the_sentence_ends() {
    let map: FileMap = scan(SRC);
    assert_eq!(
        map.purpose.as_deref(),
        Some("Parse widgets and keep them small.")
    );
    assert_eq!(map.lines, SRC.lines().count());
}

#[test]
fn items_carry_line_signature_doc_and_visibility() {
    let map: FileMap = scan(SRC);
    let max: &Item = named(&map, "MAX");
    assert_eq!((max.kind, max.vis, max.line), (Kind::Const, Vis::Public, 8));
    assert_eq!(max.signature, "pub const MAX: usize");
    let widget: &Item = named(&map, "Widget");
    assert_eq!(widget.signature, "pub struct Widget");
    assert_eq!(widget.doc.as_deref(), Some("A widget the user sees."));
    let new: &Item = named(&map, "new");
    assert_eq!(
        new.signature,
        "pub fn new(name: &str, size: [u8; 3]) -> Self"
    );
    assert_eq!(new.owner.as_deref(), Some("Widget"));
    assert_eq!(new.doc.as_deref(), Some("Make one."));
    assert_eq!(named(&map, "private_helper").vis, Vis::Private);
}

#[test]
fn bodies_strings_and_test_modules_hide_their_items() {
    let map: FileMap = scan(SRC);
    for hidden in ["fake", "inner", "hidden", "also", "text"] {
        assert!(
            map.items.iter().all(|i: &Item| i.name != hidden),
            "{hidden}"
        );
    }
    assert_eq!(map.tests, 2);
}

#[test]
fn enums_list_variants_and_impls_name_their_owner() {
    let map: FileMap = scan(SRC);
    assert_eq!(named(&map, "Colour").variants, ["Red", "Green", "Blue"]);
    let display: &Item = map
        .items
        .iter()
        .find(|i: &&Item| i.kind == Kind::Impl && i.signature.contains("Display"))
        .unwrap();
    assert_eq!(display.name, "Widget");
    assert_eq!(named(&map, "fmt").owner.as_deref(), Some("Widget"));
}

#[test]
fn trait_methods_take_the_trait_visibility_and_pub_use_is_kept() {
    let map: FileMap = scan(SRC);
    let paint: &Item = named(&map, "paint");
    assert_eq!(
        (paint.vis, paint.owner.as_deref()),
        (Vis::Restricted, Some("Paint"))
    );
    let uses: Vec<&Item> = map
        .items
        .iter()
        .filter(|i: &&Item| i.kind == Kind::Use)
        .collect();
    assert_eq!(uses.len(), 1, "a private use is not part of the map");
    assert_eq!(
        uses.first().unwrap().signature,
        "pub use crate::inner::{A, B}"
    );
}

#[test]
fn a_semicolon_inside_an_impl_header_keeps_its_methods() {
    let src: &str = "impl From<[u8; 3]> for Foo {\n    pub fn go(&self) {}\n}\n";
    let map: FileMap = scan(src);
    assert_eq!(named(&map, "go").owner.as_deref(), Some("Foo"));
}
