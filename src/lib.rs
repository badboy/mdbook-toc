use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Write;

use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::Tag::*;
use pulldown_cmark::{Event, Options, Parser};
use toml::value::Table;

pub struct Toc;

static DEFAULT_MARKER: &str = "<!-- toc -->\n";

/// Configuration for Table of Contents generation
pub struct Config {
    /// Marker to use, defaults to `<!-- toc -->\n`
    pub marker: String,
    /// The maximum level of headers to include in the table of contents.
    /// Defaults to `4`.
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
                        "Marker {marker:?} is not a valid string",
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
                        "Level {level:?} is not a valid integer",
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
    log::trace!("ToC from {toc:?}");
    let mut result = String::new();

    // "Normalize" header levels.
    // If headers skip a level, we need to normalize them to avoid the skip.
    // Otherwise the markdown render will escape nested levels.
    //
    // This is a rough approximation only.
    let mut toc_iter = toc.iter().peekable();

    // Start from the level of the first header.
    let min_level = toc.iter().map(|(lvl, _, _)| *lvl).min().unwrap_or(1);
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
        let width = 2 * (level - min_level) as usize;
        writeln!(result, "{:width$}* [{name}](#{slug})", "").unwrap();
    }

    result
}

fn add_toc(content: &str, cfg: &Config) -> Result<String> {
    let mut toc_found = false;

    let mut toc_content = vec![];
    let mut current_header = String::new();
    let mut current_header_level: Option<u32> = None;
    let mut id_counter = HashMap::new();

    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES;

    let mark: Vec<Event> = Parser::new(&cfg.marker).collect();
    log::trace!("Marker: {mark:?}");
    let mut mark_start = None;
    let mut mark_end = 0..0;
    let mut mark_loc = 0;

    let content = content.replace("\r\n", "\n");
    for (e, span) in Parser::new_ext(&content, opts).into_offset_iter() {
        log::trace!("Event: {e:?} (span: {span:?})");
        if !toc_found {
            log::trace!("TOC not found yet. Location: {mark_loc}, Start: {mark_start:?}");
            if e == mark[mark_loc] {
                if mark_start.is_none() {
                    mark_start = Some(span.clone());
                }
                mark_loc += 1;
                if mark_loc >= mark.len() {
                    mark_end = span;
                    toc_found = true
                }
            } else if mark_loc > 0 {
                mark_loc = 0;
                mark_start = None;
            } else {
                continue;
            }
        }

        if let Event::Start(Heading(lvl, fragment, classes)) = e {
            log::trace!("Header(lvl={lvl}, fragment={fragment:?}, classes={classes:?})");
            current_header_level = Some(lvl as u32);
            continue;
        }
        if let Event::End(Heading(_, fragment, _)) = e {
            // Skip if this header is nested too deeply.
            if let Some(level) = current_header_level.take() {
                let header = current_header.clone();
                let slug = if let Some(slug) = fragment {
                    // If a fragment is defined, take it as is, not trying to append an extra ID
                    // in case of duplicates (same behavior as mdBook)
                    slug.to_owned()
                } else {
                    let mut slug = mdbook::utils::normalize_id(&header);
                    let id_count = id_counter.entry(slug.clone()).or_insert(0);

                    // Append unique ID if multiple headers with the same name exist
                    // to follow what mdBook does
                    if *id_count > 0 {
                        write!(slug, "-{id_count}").unwrap();
                    }

                    *id_count += 1;
                    slug
                };

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
            Event::Text(header) => write!(current_header, "{header}").unwrap(),
            Event::Code(code) => write!(current_header, "`{code}`").unwrap(),
            _ => {} // Rest is unhandled
        }
    }

    let toc = build_toc(&toc_content);
    log::trace!("Built TOC: {toc:?}");
    log::trace!("toc_found={toc_found} mark_start={mark_start:?} mark_end={mark_end:?}");

    let content = if toc_found {
        let mark_start = mark_start.unwrap();
        let content_before_toc = &content[0..mark_start.start];
        let content_after_toc = &content[mark_end.end..];
        log::trace!("content_before_toc={content_before_toc:?}");
        log::trace!("content_after_toc={content_after_toc:?}");
        // Multiline markers might have consumed trailing newlines,
        // we ensure there's always one before the content.
        let extra = if content_after_toc.is_empty() || content_after_toc.as_bytes()[0] == b'\n' {
            ""
        } else {
            "\n"
        };
        format!("{content_before_toc}{toc}{extra}{content_after_toc}")
    } else {
        content.to_string()
    };

    Ok(content)
}

impl Toc {
    /// Add a table of contents to the given chapter.
    pub fn add_toc(chapter: &Chapter, cfg: &Config) -> Result<String> {
        add_toc(&chapter.content, cfg)
    }
}
