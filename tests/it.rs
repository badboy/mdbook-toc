use mdbook_toc::{Toc, Config};
use pretty_assertions::assert_eq;
use mdbook::book::Chapter;
use mdbook::errors::Result;

fn default<T: Default>() -> T {
    Default::default()
}

fn with_marker<S: Into<String>>(marker: S) -> Config {
    let mut cfg = Config::default();
    cfg.marker = marker.into();
    cfg
}

fn with_max_level(level: u32) -> Config {
    let mut cfg = Config::default();
    cfg.max_level = level;
    cfg
}

trait FromContent {
    fn from_content(content: &str) -> Self;
}

impl FromContent for Chapter {
    fn from_content(content: &str) -> Self {
        Self {
            name: "chapter".into(),
            content: content.into(),
            number: None,
            sub_items: vec![],
            path: None,
            source_path: None,
            parent_names: vec![]
        }
    }
}

fn add_toc(content: &str, config: &Config) -> Result<String> {
    let chapter = Chapter::from_content(content);
    Toc::add_toc(&chapter, config)
}

#[test]
fn adds_toc() {
    let content = r#"# Chapter

<!-- toc -->

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1

"#;

        let expected = r#"# Chapter

* [Header 1](#header-1)
  * [Header 1.1](#header-11)
* [Header 2](#header-2)
  * [Header 2.1](#header-21)
  * [Header 2.2](#header-22)
    * [Header 2.2.1](#header-221)

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1"#;

    assert_eq!(expected, add_toc(content, &default()).unwrap());
}


    #[test]
    fn adds_toc_with_inline_code() {
        let content = r#"# Chapter

<!-- toc -->

# Header 1

## `Header 1.1`

# Header 2

## Header 2.1
"#;

        let expected = r#"# Chapter

* [Header 1](#header-1)
  * [`Header 1.1`](#header-11)
* [Header 2](#header-2)
  * [Header 2.1](#header-21)

# Header 1

## `Header 1.1`

# Header 2

## Header 2.1"#;

        assert_eq!(expected, add_toc(content, &default()).unwrap());
    }

    #[test]
    fn leaves_tables_untouched() {
        // Regression test.
        // Previously we forgot to enable the same markdwon extensions as mdbook itself.

        let content = r#"# Heading

| Head 1 | Head 2 |
|--------|--------|
| Row 1  | Row 2  |
"#;

        // Markdown roundtripping removes some insignificant whitespace
        let expected = r#"# Heading

|Head 1|Head 2|
|------|------|
|Row 1|Row 2|"#;

        assert_eq!(expected, add_toc(content, &default()).unwrap());
    }

    #[test]
    fn handles_inline_code() {
        let _ = env_logger::builder().is_test(true).try_init();

        // Regression test.
        // Inline code in a header was broken up into multiple items.
        // Also test for deeply nested headers.

        let content = r#"# Chapter

<!-- toc -->

# Header 1

## Header 1.1

### Header 1.1.1

#### Header 1.1.1.1

##### Header 1.1.1.1.1

# Another header `with inline` code

"#;

        let expected = r#"# Chapter

* [Header 1](#header-1)
  * [Header 1.1](#header-11)
    * [Header 1.1.1](#header-111)
      * [Header 1.1.1.1](#header-1111)
* [Another header `with inline` code](#another-header-with-inline-code)

# Header 1

## Header 1.1

### Header 1.1.1

#### Header 1.1.1.1

##### Header 1.1.1.1.1

# Another header `with inline` code"#;

        assert_eq!(expected, add_toc(content, &default()).unwrap());
    }

    #[test]
    fn multi_header_regression() {
        let content = r#"# Main Summary

<!-- toc -->

# Introduction

### Contents

### Background and Caveats

#### Test

### Accessing the Data

# Adding New Fields

## User Preferences"#;

        let expected = r#"# Main Summary

* [Introduction](#introduction)
  * [Contents](#contents)
  * [Background and Caveats](#background-and-caveats)
  * [Test](#test)
  * [Accessing the Data](#accessing-the-data)
* [Adding New Fields](#adding-new-fields)
  * [User Preferences](#user-preferences)

# Introduction

### Contents

### Background and Caveats

#### Test

### Accessing the Data

# Adding New Fields

## User Preferences"#;

        assert_eq!(expected, add_toc(content, &default()).unwrap());
    }

    #[test]
    fn multi_header_linear_regression_3() {
        let content = r#"# Heading

<!-- toc -->

## Level 1.1
### Level 1.1.1
### Level 1.1.2
## Level 1.2
### Level 1.2.1

text"#;

        let expected = r#"# Heading

* [Level 1.1](#level-11)
  * [Level 1.1.1](#level-111)
  * [Level 1.1.2](#level-112)
* [Level 1.2](#level-12)
  * [Level 1.2.1](#level-121)

## Level 1.1

### Level 1.1.1

### Level 1.1.2

## Level 1.2

### Level 1.2.1

text"#;

        assert_eq!(expected, add_toc(content, &default()).unwrap());
    }

    #[test]
    fn add_toc_with_gitlab_marker() {
        let marker = "[[_TOC_]]".to_owned();
        let content = r#"# Chapter

[[_TOC_]]

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1

"#;

        let expected = r#"# Chapter

* [Header 1](#header-1)
  * [Header 1.1](#header-11)
* [Header 2](#header-2)
  * [Header 2.1](#header-21)
  * [Header 2.2](#header-22)
    * [Header 2.2.1](#header-221)

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1"#;

        assert_eq!(expected, add_toc(content, &with_marker(marker)).unwrap());
    }

    #[test]
    fn unique_slugs() {
        let content = r#"# Chapter

<!-- toc -->

## Duplicate

### Duplicate

#### Duplicate

##### Duplicate

## Duplicate"#;

        let expected = r#"# Chapter

* [Duplicate](#duplicate)
  * [Duplicate](#duplicate-1)
    * [Duplicate](#duplicate-2)
* [Duplicate](#duplicate-4)

## Duplicate

### Duplicate

#### Duplicate

##### Duplicate

## Duplicate"#;

        assert_eq!(expected, add_toc(content, &default()).unwrap());
    }

    #[test]
    fn add_toc_with_github_marker() {
        let marker = "* auto-gen TOC:\n{:toc}".to_owned();
        let content = r#"# Chapter

* auto-gen TOC:
{:toc}

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1

"#;

        let expected = r#"# Chapter

* [Header 1](#header-1)
  * [Header 1.1](#header-11)
* [Header 2](#header-2)
  * [Header 2.1](#header-21)
  * [Header 2.2](#header-22)
    * [Header 2.2.1](#header-221)

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1"#;

        assert_eq!(expected, add_toc(content, &with_marker(marker)).unwrap());
    }

    #[test]
    fn lower_max_level() {
        let content = r#"# Chapter

<!-- toc -->

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1

"#;

        let expected = r#"# Chapter

* [Header 1](#header-1)
  * [Header 1.1](#header-11)
* [Header 2](#header-2)
  * [Header 2.1](#header-21)
  * [Header 2.2](#header-22)

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1"#;

        assert_eq!(expected, add_toc(content, &with_max_level(2)).unwrap());
    }

    #[test]
    fn higher_max_level() {
        let content = r#"# Chapter

<!-- toc -->

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1

"#;

        let expected = r#"# Chapter

* [Header 1](#header-1)
  * [Header 1.1](#header-11)
* [Header 2](#header-2)
  * [Header 2.1](#header-21)
  * [Header 2.2](#header-22)
    * [Header 2.2.1](#header-221)

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1"#;

        assert_eq!(expected, add_toc(content, &with_max_level(7)).unwrap());
    }

    // Regression test for [#13](https://github.com/badboy/mdbook-toc/issues/13).
    // Choosing a non-HTML TOC marker breaks sites that don't use it at all,
    // removed the header and first paragraph.
    #[test]
    fn nonhtml_marker_no_toc_in_page() {
        let _ = env_logger::builder().is_test(true).try_init();

        let marker = "[[_TOC_]]".to_owned();
        let content = r#"# Chapter

First paragraph

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1

"#;

        let expected = r#"# Chapter

First paragraph

# Header 1

## Header 1.1

# Header 2

## Header 2.1

## Header 2.2

### Header 2.2.1"#;

        assert_eq!(expected, add_toc(content, &with_marker(marker)).unwrap());
    }

    #[test]
    fn similar_heading_different_casing() {
        // Regression test #15
        // Previously we didn't use the normalized header ("slug") to decide whether to use
        // different link anchors.
        //
        let content = r#"# Chapter

<!-- toc -->

# Tag

## tag

"#;

        let expected = r#"# Chapter

* [Tag](#tag)
  * [tag](#tag-1)

# Tag

## tag"#;

        assert_eq!(expected, add_toc(content, &default()).unwrap());
    }
