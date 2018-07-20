extern crate mdbook;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;

use std::borrow::Cow;
use mdbook::errors::{Error, Result};
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{Event, Parser};
use pulldown_cmark::Tag::*;
use pulldown_cmark_to_cmark::fmt::cmark;

pub struct Toc;

impl Preprocessor for Toc {
    fn name(&self) -> &str {
        "toc"
    }

    fn run(&self, _ctx: &PreprocessorContext, book: &mut Book) -> Result<()> {
        let mut res: Option<_> = None;
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
        res.unwrap_or(Ok(()))
    }
}

fn build_toc<'a>(toc: &[(i32, Cow<'a, str>)]) -> String {
    let mut result = String::new();

    for (level, name) in toc {
        let width = 2*(level-1) as usize;
        let slug = mdbook::utils::normalize_id(&name);
        let entry = format!("{1:0$}* [{2}](#{3})\n", width, "", name, slug);
        result.push_str(&entry);
    }

    result
}

impl Toc {
    fn add_toc(chapter: &mut Chapter) -> Result<String> {
        let mut buf = String::with_capacity(chapter.content.len());
        let mut toc_found = false;

        let mut toc_content = vec![];
        let mut current_header_level : Option<i32> = None;

        for e in Parser::new(&chapter.content) {
            if let Event::Html(html) = e {
                if html == "<!-- toc -->\n" {
                    toc_found = true;
                }
                continue;
            }
            if !toc_found {
                continue;
            }

            if let Event::Start(Header(lvl)) = e {
                if lvl < 3 {
                    current_header_level = Some(lvl);
                }
                continue;
            }
            if let Event::End(Header(_)) = e {
                current_header_level = None;
                continue;
            }
            if current_header_level.is_none() {
                continue;
            }

            if let Event::Text(header) = e {
                toc_content.push((current_header_level.unwrap(), header));
            }
        }

        let toc_events = build_toc(&toc_content);
        let toc_events = Parser::new(&toc_events).collect::<Vec<_>>();

        let events = Parser::new(&chapter.content).map(|e| {
            if let Event::Html(html) = e.clone() {
                if html == "<!-- toc -->\n" {
                    return toc_events.clone();
                }
            }
            vec![e]
        }).flat_map(|e| e);

        cmark(events, &mut buf, None)
            .map(|_| buf)
            .map_err(|err| Error::from(format!("Markdown serialization failed: {}", err)))
    }
}
