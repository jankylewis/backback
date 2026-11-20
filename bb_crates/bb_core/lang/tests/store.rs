//! BBEOD26: the ticks saved per workspace read back as written; a file
//! backback did not write is refused, never guessed. BBEOD41: v2 marks each
//! record as picked by the user; a v1 file (auto-saved found sets mixed with
//! picks, not told apart) reads as no choice at all.

use std::path::{Path, PathBuf};

use bb_lang::{choice_for, decode, encode, with_choice, Choices, Lang, LangSet, StoreError};

#[test]
fn every_tick_set_saved_for_a_workspace_reads_back_the_same() {
    for set in LangSet::non_empty() {
        let saved: Choices = with_choice(Vec::new(), Path::new("/w/App"), set);
        let read: Choices = decode(&encode(&saved)).unwrap();
        assert_eq!(choice_for(&read, Path::new("/w/App")), Some(set), "{set:?}");
    }
}

#[test]
fn a_new_choice_replaces_the_old_one_and_goes_first() {
    let csharp: LangSet = LangSet::of(&[Lang::CSharp]);
    let rust: LangSet = LangSet::of(&[Lang::Rust, Lang::Python]);
    let first: Choices = with_choice(Vec::new(), Path::new("/a"), csharp);
    let second: Choices = with_choice(first, Path::new("/b"), csharp);
    let third: Choices = with_choice(second, Path::new("/a"), rust);
    assert_eq!(
        third,
        vec![(PathBuf::from("/a"), rust), (PathBuf::from("/b"), csharp)]
    );
}

#[test]
fn an_empty_tick_set_is_never_saved() {
    let saved: Choices = with_choice(Vec::new(), Path::new("/a"), LangSet::EMPTY);
    assert_eq!(saved.len(), 0);
    assert_eq!(choice_for(&saved, Path::new("/a")), None);
}

#[test]
fn an_unknown_header_or_language_is_refused() {
    assert_eq!(
        decode("backback langs v9\npicked\tcsharp\t/a\n"),
        Err(StoreError::UnknownFormat("backback langs v9".to_owned()))
    );
    assert_eq!(
        decode("backback langs v2\npicked\tgo\t/a\n"),
        Err(StoreError::BadLine("picked\tgo\t/a".to_owned()))
    );
    assert_eq!(
        decode("backback langs v2\npicked\t\t/a\n"),
        Err(StoreError::BadLine("picked\t\t/a".to_owned()))
    );
}

#[test]
fn every_saved_record_carries_the_picked_mark() {
    let saved: Choices = with_choice(
        Vec::new(),
        Path::new("/w/App"),
        LangSet::of(&[Lang::CSharp]),
    );
    assert_eq!(
        encode(&saved),
        "backback langs v2\npicked\tcsharp\t/w/App\n"
    );
}

#[test]
fn a_record_without_the_picked_mark_is_refused() {
    assert_eq!(
        decode("backback langs v2\ncsharp\t/a\n"),
        Err(StoreError::BadLine("csharp\t/a".to_owned()))
    );
    assert_eq!(
        decode("backback langs v2\nfound\tcsharp,python\t/a\n"),
        Err(StoreError::BadLine("found\tcsharp,python\t/a".to_owned()))
    );
}

#[test]
fn a_v1_file_reads_as_no_choice_for_any_workspace() {
    let old: &str = "backback langs v1\ncsharp,python\t/a\npython\t/b\ncsharp\t/c\n";
    let read: Choices = decode(old).unwrap();
    assert_eq!(read, Vec::new());
    for workspace in ["/a", "/b", "/c"] {
        assert_eq!(choice_for(&read, Path::new(workspace)), None, "{workspace}");
    }
}
