use mdbook::book::Chapter;
use mdbook_toc::{Config, Toc};
use pretty_assertions::assert_eq;

fn default<T: Default>() -> T {
    Default::default()
}

fn with_marker<S: Into<String>>(marker: S) -> Config {
    Config {
        marker: marker.into(),
        ..Config::default()
    }
}

fn with_max_level(level: u32) -> Config {
    Config {
        max_level: level,
        ..Config::default()
    }
}

trait FromContent {
    fn from_content(content: String) -> Self;
}

impl FromContent for Chapter {
    fn from_content(content: String) -> Self {
        Self {
            name: "chapter".into(),
            content,
            number: None,
            sub_items: vec![],
            path: None,
            source_path: None,
            parent_names: vec![],
        }
    }
}

/// Assert the Table of Content generation for an input file against the expected output file.
///
/// Reads `tests/$name.in.md` and checks the generated ToC code against `tests/$name.out.md`.
macro_rules! assert_toc {
    ($name:expr) => {
        assert_toc!($name, default())
    };
    ($name:expr, $config:expr) => {
        let _ = env_logger::builder().is_test(true).try_init();

        let config = $config;
        let content = ::std::fs::read_to_string(format!("tests/{}.in.md", $name)).expect(concat!(
            "Can't read ",
            $name,
            ".in.md"
        ));
        let expected = ::std::fs::read_to_string(format!("tests/{}.out.md", $name))
            .expect(concat!("Can't read ", $name, ".out.md"));

        let chapter = Chapter::from_content(content);
        let result = Toc::add_toc(&chapter, &config);
        match result {
            Ok(result) => assert_eq!(expected, result),
            Err(e) => panic!("{} failed. Error: {}", $name, e),
        }
    };
}

#[test]
fn adds_toc() {
    assert_toc!("adds_toc", default());
}

#[test]
fn adds_toc_with_inline_code() {
    assert_toc!("with_inline_code", default());
}

#[test]
fn leaves_tables_untouched() {
    // Regression test.
    // Previously we forgot to enable the same markdwon extensions as mdbook itself.
    // Markdown roundtripping removes some insignificant whitespace
    assert_toc!("tables_untouched");
}

#[test]
fn handles_inline_code() {
    // Regression test.
    // Inline code in a header was broken up into multiple items.
    // Also test for deeply nested headers.
    assert_toc!("handles_inline_code");
}

#[test]
fn multi_header_regression() {
    assert_toc!("multi_header");
}

#[test]
fn multi_header_linear_regression_3() {
    assert_toc!("multi_header_linear");
}

#[test]
fn add_toc_with_gitlab_marker() {
    let marker = "[[_TOC_]]".to_owned();

    assert_toc!("gitlab_marker", with_marker(marker));
}

#[test]
fn unique_slugs() {
    assert_toc!("unique_slugs");
}

#[test]
fn add_toc_with_github_marker() {
    let marker = "* auto-gen TOC:\n{:toc}\n".to_owned();
    assert_toc!("github_marker", with_marker(marker));
}

#[test]
fn lower_max_level() {
    assert_toc!("lower_max_level", with_max_level(2));
}

#[test]
fn higher_max_level() {
    assert_toc!("higher_max_level", with_max_level(7));
}

// Regression test for [#13](https://github.com/badboy/mdbook-toc/issues/13).
// Choosing a non-HTML TOC marker breaks sites that don't use it at all,
// removed the header and first paragraph.
#[test]
fn nonhtml_marker_no_toc_in_page() {
    let marker = "[[_TOC_]]".to_owned();
    assert_toc!("nonhtml_marker_no_use", with_marker(marker));
}

#[test]
fn similar_heading_different_casing() {
    // Regression test #15
    // Previously we didn't use the normalized header ("slug") to decide whether to use
    // different link anchors.

    assert_toc!("similar_heading_different_casing");
}

#[test]
fn tables_with_html() {
    assert_toc!("tables_with_html");
}

#[test]
fn backslash_escapes() {
    // Regression test #21
    // Backslash-escaped elements should still be escaped.
    assert_toc!("backslash_escapes");
}

#[test]
fn empty_document() {
    // Regression test #31
    // Empty documents should not fail
    assert_toc!("empty_document");
}

#[test]
fn crlf() {
    assert_toc!("crlf");
}

#[test]
fn attributes() {
    assert_toc!("attributes");
}
