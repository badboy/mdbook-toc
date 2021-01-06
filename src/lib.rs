use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write;

use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::Tag::*;
use pulldown_cmark::{Event, Options, Parser};
use pulldown_cmark_to_cmark::{cmark_with_options, Options as COptions};

pub struct Toc;

static DEFAULT_MARKER: &str = "<!-- toc -->\n";

impl Preprocessor for Toc {
    fn name(&self) -> &str {
        "toc"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let mut res = None;
        let toc_marker = if let Some(cfg) = ctx.config.get_preprocessor(self.name()) {
            if let Some(marker) = cfg.get("marker") {
                match marker.as_str() {
                    Some(m) => m,
                    None => {
                        return Err(Error::msg(format!(
                            "Marker {:?} is not a valid string",
                            marker
                        )))
                    }
                }
            } else {
                DEFAULT_MARKER
            }
        } else {
            DEFAULT_MARKER
        };

        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }

            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(Toc::add_toc(chapter, &toc_marker).map(|md| {
                    chapter.content = md;
                }));
            }
        });

        res.unwrap_or(Ok(())).map(|_| book)
    }
}

fn build_toc(toc: &[(u32, String, String)]) -> String {
    log::trace!("ToC from {:?}", toc);
    let mut result = String::new();

    // "Normalize" header levels.
    // If headers skip a level, we need to normalize them to avoid the skip.
    // Otherwise the markdown render will escape nested levels.
    //
    // This is a rough approximation only.
    let mut toc_iter = toc.iter().peekable();

    // Start from the level of the first header.
    let mut last_lower = match toc_iter.peek() {
        Some((lvl, _, _)) => *lvl,
        None => 0,
    };
    let toc = toc.iter().map(|(lvl, name, slug)| {
        let lvl = *lvl;
        let lvl = match (last_lower + 1).cmp(&lvl) {
            Ordering::Less => last_lower + 1,
            _ => {
                last_lower = lvl;
                lvl
            }
        };
        (lvl, name, slug)
    });

    for (level, name, slug) in toc {
        let width = 2 * (level - 1) as usize;
        writeln!(result, "{1:0$}* [{2}](#{3})", width, "", name, slug).unwrap();
    }

    result
}

fn add_toc(content: &str, marker: &str) -> Result<String> {
    let mut buf = String::with_capacity(content.len());
    let mut toc_found = false;

    let mut toc_content = vec![];
    let mut current_header = String::new();
    let mut current_header_level: Option<u32> = None;
    let mut id_counter = HashMap::new();

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let mark: Vec<Event> = Parser::new(marker).collect();
    let mut mark_start = -1;
    let mut mark_loc = 0;
    let mut c = -1;

    for e in Parser::new_ext(&content, opts) {
        c += 1;
        log::trace!("Event: {:?}", e);
        if !toc_found {
            if e == mark[mark_loc] {
                if mark_start == -1 {
                    mark_start = c;
                }
                mark_loc += 1;
                if mark_loc >= mark.len() {
                    toc_found = true
                }
            } else if mark_loc > 0 {
                mark_loc = 0;
                mark_start = -1;
            } else {
                continue;
            }
        }

        if let Event::Start(Heading(lvl)) = e {
            current_header_level = Some(lvl);
            continue;
        }
        if let Event::End(Heading(_)) = e {
            // Skip if this header is nested too deeply.
            if let Some(level) = current_header_level.take() {
                let header = current_header.clone();
                let mut slug = mdbook::utils::normalize_id(&header);
                let id_count = id_counter.entry(header.clone()).or_insert(0);

                // Append unique ID if multiple headers with the same name exist
                // to follow what mdBook does
                if *id_count > 0 {
                    write!(slug, "-{}", id_count).unwrap();
                }

                *id_count += 1;

                if level < 5 {
                    toc_content.push((level, header, slug));
                }

                current_header.clear();
            }
            continue;
        }
        if current_header_level.is_none() {
            continue;
        }

        match e {
            Event::Text(header) => write!(current_header, "{}", header).unwrap(),
            Event::Code(code) => write!(current_header, "`{}`", code).unwrap(),
            _ => {} // Rest is unhandled
        }
    }

    let toc = build_toc(&toc_content);
    let toc_events = Parser::new(&toc).collect::<Vec<_>>();

    let mut c = -1;
    let events = Parser::new_ext(&content, opts)
        .map(|e| {
            c += 1;
            if c > mark_start && c < mark_start + (mark.len() as i32) {
                vec![]
            } else if c == mark_start {
                toc_events.clone()
            } else {
                vec![e]
            }
            // if let Event::Html(html) = e.clone() {
            //     if &*html == marker {
            //         return toc_events.clone();
            //     }
            // }
            // vec![e]
        })
        .flatten();

    let opts = COptions {
        newlines_after_codeblock: 1,
        ..Default::default()
    };
    cmark_with_options(events, &mut buf, None, opts)
        .map(|_| buf)
        .map_err(|err| Error::msg(format!("Markdown serialization failed: {}", err)))
}

impl Toc {
    fn add_toc(chapter: &mut Chapter, marker: &str) -> Result<String> {
        add_toc(&chapter.content, marker)
    }
}

#[cfg(test)]
mod test {
    use super::{add_toc, DEFAULT_MARKER};
    use pretty_assertions::assert_eq;

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

        assert_eq!(expected, add_toc(content, DEFAULT_MARKER).unwrap());
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

        assert_eq!(expected, add_toc(content, DEFAULT_MARKER).unwrap());
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

        assert_eq!(expected, add_toc(content, DEFAULT_MARKER).unwrap());
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

        assert_eq!(expected, add_toc(content, DEFAULT_MARKER).unwrap());
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

        assert_eq!(expected, add_toc(content, DEFAULT_MARKER).unwrap());
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

        assert_eq!(expected, add_toc(content, DEFAULT_MARKER).unwrap());
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

        assert_eq!(expected, add_toc(content, &marker).unwrap());
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

        assert_eq!(expected, add_toc(content, DEFAULT_MARKER).unwrap());
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

        assert_eq!(expected, add_toc(content, &marker).unwrap());
    }
}
