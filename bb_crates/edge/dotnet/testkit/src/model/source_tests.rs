use super::*;

fn fqn(name: &str) -> Fqn {
    Fqn::new(name).unwrap()
}

fn tree(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    for (name, text) in files {
        let path: PathBuf = dir.path().join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir
}

const SPEC: &str = "using NUnit.Framework;\nnamespace App.Tests.Auth;\npublic sealed class OfsiSpec\n{\n    [Test]\n    public void Signs_in() { }\n\n    [TestCase(1)]\n    public async Task Counts(int n) { }\n}\n";

#[test]
fn a_test_opens_on_its_method_line() {
    let dir: tempfile::TempDir = tree(&[("Auth/OfsiSpec.cs", SPEC)]);
    let found: Option<(PathBuf, usize)> =
        find_source(dir.path(), &fqn("App.Tests.Auth.OfsiSpec.Signs_in"));
    assert_eq!(found, Some((dir.path().join("Auth/OfsiSpec.cs"), 5)));
}

#[test]
fn a_parameterised_case_opens_on_its_method() {
    let dir: tempfile::TempDir = tree(&[("OfsiSpec.cs", SPEC)]);
    let found: Option<(PathBuf, usize)> =
        find_source(dir.path(), &fqn("App.Tests.Auth.OfsiSpec.Counts(1)"));
    assert_eq!(found.map(|(_, line): (PathBuf, usize)| line), Some(8));
}

#[test]
fn a_method_not_in_the_text_opens_on_the_class() {
    let dir: tempfile::TempDir = tree(&[("OfsiSpec.cs", SPEC)]);
    let found: Option<(PathBuf, usize)> =
        find_source(dir.path(), &fqn("App.Tests.Auth.OfsiSpec.Inherited"));
    assert_eq!(found.map(|(_, line): (PathBuf, usize)| line), Some(2));
}

#[test]
fn the_class_in_the_tests_namespace_wins_and_build_output_is_skipped() {
    let other: &str = "namespace Other;\nclass OfsiSpec { void Signs_in() { } }\n";
    let dir: tempfile::TempDir = tree(&[
        ("A/OfsiSpec.cs", other),
        ("obj/OfsiSpec.cs", SPEC),
        ("Z/OfsiSpec.cs", SPEC),
    ]);
    let found: Option<(PathBuf, usize)> =
        find_source(dir.path(), &fqn("App.Tests.Auth.OfsiSpec.Signs_in"));
    assert_eq!(found, Some((dir.path().join("Z/OfsiSpec.cs"), 5)));
}

#[test]
fn a_longer_name_and_a_call_on_another_object_do_not_match() {
    let text: &str = "namespace N;\nclass C\n{\n    void Go() => other.Run();\n    void RunAll() { }\n    void Run() { }\n}\n";
    let dir: tempfile::TempDir = tree(&[("C.cs", text)]);
    let found: Option<(PathBuf, usize)> = find_source(dir.path(), &fqn("N.C.Run"));
    assert_eq!(found.map(|(_, line): (PathBuf, usize)| line), Some(5));
}

#[test]
fn nested_and_generic_classes_use_their_simple_name() {
    let text: &str =
        "namespace N;\nclass Outer\n{\n    class Inner\n    {\n        void T() { }\n    }\n}\n";
    let dir: tempfile::TempDir = tree(&[("O.cs", text)]);
    assert_eq!(
        find_source(dir.path(), &fqn("N.Outer+Inner.T")).map(|(_, l): (PathBuf, usize)| l),
        Some(5)
    );
    assert_eq!(simple_class("Spec`1"), "Spec");
}

#[test]
fn no_declaring_file_gives_nothing() {
    let dir: tempfile::TempDir = tree(&[("A.cs", "namespace N;\nclass A { }\n")]);
    assert_eq!(find_source(dir.path(), &fqn("N.Missing.T")), None);
}
