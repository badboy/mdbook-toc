use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Write;

use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::Tag::*;
use pulldown_cmark::{Event, Options, Parser};
use pulldown_cmark_to_cmark::{cmark_with_options, Options as COptions};
use toml::value::Table;

pub struct Toc;

static DEFAULT_MARKER: &str = "<!-- toc -->\n";

pub struct Config {
    pub marker: String,
    pub max_level: u32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            marker: DEFAULT_MARKER.into(),
            max_level: 4,
        }
    }
}

impl<'a> TryFrom<Option<&'a Table>> for Config {
    type Error = Error;

    fn try_from(mdbook_cfg: Option<&Table>) -> Result<Config> {
        let mut cfg = Config::default();
        let mdbook_cfg = match mdbook_cfg {
            Some(c) => c,
            None => return Ok(cfg),
        };

        if let Some(marker) = mdbook_cfg.get("marker") {
            let marker = match marker.as_str() {
                Some(m) => m,
                None => {
                    return Err(Error::msg(format!(
                        "Marker {:?} is not a valid string",
                        marker
                    )))
                }
            };
            cfg.marker = marker.into();
        }

        if let Some(level) = mdbook_cfg.get("max-level") {
            let level = match level.as_integer() {
                Some(l) => l,
                None => {
                    return Err(Error::msg(format!(
                        "Level {:?} is not a valid integer",
                        level
                    )))
                }
            };
            cfg.max_level = level.try_into()?;
        }

        Ok(cfg)
    }
}

impl Preprocessor for Toc {
    fn name(&self) -> &str {
        "toc"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let mut res = None;
        let cfg = ctx.config.get_preprocessor(self.name()).try_into()?;

        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }

            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(Toc::add_toc(chapter, &cfg).map(|md| {
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

fn add_toc(content: &str, cfg: &Config) -> Result<String> {
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

    let mark: Vec<Event> = Parser::new(&cfg.marker).collect();
    let mut mark_start = -1;
    let mut mark_loc = 0;
    let mut c = -1;

    for e in Parser::new_ext(&content, opts) {
        c += 1;
        log::trace!("Event: {:?}", e);
        if !toc_found {
            log::trace!(
                "TOC not found yet. Location: {}, Start: {}",
                mark_loc,
                mark_start
            );
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
                let id_count = id_counter.entry(slug.clone()).or_insert(0);

                // Append unique ID if multiple headers with the same name exist
                // to follow what mdBook does
                if *id_count > 0 {
                    write!(slug, "-{}", id_count).unwrap();
                }

                *id_count += 1;

                if level <= cfg.max_level {
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
    log::trace!("Built TOC: {:?}", toc);
    let toc_events = Parser::new(&toc).collect::<Vec<_>>();

    let mut c = -1;
    let events = Parser::new_ext(&content, opts)
        .map(|e| {
            c += 1;
            if toc_found && c > mark_start && c < mark_start + (mark.len() as i32) {
                vec![]
            } else if toc_found && c == mark_start {
                toc_events.clone()
            } else {
                vec![e]
            }
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
    pub fn add_toc(chapter: &Chapter, cfg: &Config) -> Result<String> {
        add_toc(&chapter.content, cfg)
    }
}
