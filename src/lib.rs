use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::CowStr;
use pulldown_cmark::Tag::*;
use pulldown_cmark::{Event, Parser, Options};
use pulldown_cmark_to_cmark::cmark;

pub struct Toc;

impl Preprocessor for Toc {
    fn name(&self) -> &str {
        "toc"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let mut res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }

            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(Toc::add_toc(chapter).map(|md| {
                    chapter.content = md;
                }));
            }
        });

        res.unwrap_or(Ok(())).map(|_| book)
    }
}

fn build_toc<'a>(toc: &[(u32, CowStr<'a>)]) -> String {
    let mut result = String::new();

    for (level, name) in toc {
        let width = 2 * (level - 1) as usize;
        let slug = mdbook::utils::normalize_id(&name);
        let entry = format!("{1:0$}* [{2}](#{3})\n", width, "", name, slug);
        result.push_str(&entry);
    }

    result
}

fn add_toc(content: &str) -> Result<String> {
    let mut buf = String::with_capacity(content.len());
    let mut toc_found = false;

    let mut toc_content = vec![];
    let mut current_header_level: Option<u32> = None;

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    for e in Parser::new_ext(&content, opts.clone()) {
        if let Event::Html(html) = e {
            if &*html == "<!-- toc -->\n" {
                toc_found = true;
            }
            continue;
        }
        if !toc_found {
            continue;
        }

        if let Event::Start(Heading(lvl)) = e {
            if lvl < 5 {
                current_header_level = Some(lvl);
            }
            continue;
        }
        if let Event::End(Heading(_)) = e {
            current_header_level = None;
            continue;
        }
        if current_header_level.is_none() {
            continue;
        }

        match e {
            Event::Text(header) => toc_content.push((current_header_level.unwrap(), header)),
            Event::Code(code) => {
                let text = format!("`{}`", code);
                toc_content.push((current_header_level.unwrap(), text.into()));
            }
            _ => {} // Rest is unhandled
        }
    }

    let toc_events = build_toc(&toc_content);
    let toc_events = Parser::new(&toc_events).collect::<Vec<_>>();

    let events = Parser::new_ext(&content, opts)
        .map(|e| {
            if let Event::Html(html) = e.clone() {
                if &*html == "<!-- toc -->\n" {
                    return toc_events.clone();
                }
            }
            vec![e]
        })
        .flat_map(|e| e);

    cmark(events, &mut buf, None)
        .map(|_| buf)
        .map_err(|err| Error::from(format!("Markdown serialization failed: {}", err)))
}

impl Toc {
    fn add_toc(chapter: &mut Chapter) -> Result<String> {
        add_toc(&chapter.content)
    }
}

#[cfg(test)]
mod test {
    use super::add_toc;
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

        assert_eq!(expected, add_toc(content).unwrap());
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

        assert_eq!(expected, add_toc(content).unwrap());
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

        assert_eq!(expected, add_toc(content).unwrap());
    }
}
