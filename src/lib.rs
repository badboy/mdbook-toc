use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write;

use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::errors::Result;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{Event, Options, Parser};
use pulldown_cmark::{Tag::*, TagEnd};

pub struct Toc;

static DEFAULT_MARKER: &str = "<!-- toc -->\n";
static DEFAULT_MAX_LEVEL: u32 = 4;

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
            max_level: DEFAULT_MAX_LEVEL,
        }
    }
}

impl Preprocessor for Toc {
    fn name(&self) -> &str {
        "toc"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let mut res = None;
        let cfg_key = |key| format!("preprocessor.{}.{}", self.name(), key);
        let cfg = Config {
            marker: ctx
                .config
                .get(&cfg_key("marker"))?
                .unwrap_or_else(|| DEFAULT_MARKER.into()),
            max_level: ctx
                .config
                .get(&cfg_key("max_level"))?
                .unwrap_or(DEFAULT_MAX_LEVEL),
        };

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

/// Convert the given string to a valid HTML element ID.
/// The only restriction is that the ID must not contain any ASCII whitespace.
fn normalize_id(content: &str) -> String {
    content
        .chars()
        .filter_map(|ch| {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
}

fn add_toc(content: &str, cfg: &Config) -> Result<String> {
    let mut toc_found = false;

    let mut toc_content = vec![];
    let mut current_header = None;
    let mut current_header_level: Option<(u32, Option<String>)> = None;
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
        log::trace!(
            "Event: {e:?} (span: {span:?}, content: {:?})",
            &content[span.start..span.end]
        );
        if !toc_found {
            log::trace!("TOC not found yet. Location: {mark_loc}, Start: {mark_start:?}");
            if e == mark[mark_loc] {
                if mark_start.is_none() {
                    mark_start = Some(span.clone());
                }
                mark_loc += 1;
                if mark_loc >= mark.len() {
                    mark_end = span.clone();
                    toc_found = true
                }
            } else if mark_loc > 0 {
                mark_loc = 0;
                mark_start = None;
            } else {
                continue;
            }
        }
        log::trace!("TOC found. Location: {mark_loc}, Start: {mark_start:?}");

        if let Event::Start(Heading { level, id, .. }) = e {
            log::trace!("Header(lvl={level}, fragment={id:?})");
            let id = id.map(|s| s.to_string());
            current_header_level = Some((level as u32, id));

            // Find start of the header after `#`
            let header_content = content[span.start..span.end].trim_end();
            let idx = header_content.find(|c: char| c != '#' && !c.is_ascii_whitespace());
            current_header = Some((span.start + idx.unwrap_or(0), 0));
            continue;
        }
        // Headers might consist of text and code. pulldown_cmark unescapes `\\`, so we try to find
        // the correct span and extract the text ourselves later.
        // We enabled `HEADING_ATTRIBUTES` so attributes within `{ }` won't be in the emitted event
        if let Some(current_header) = &mut current_header {
            if let Event::Text(_) = &e
                && span.end > current_header.1
            {
                current_header.1 = span.end;
            }
            if let Event::Code(_) = &e
                && span.end > current_header.1
            {
                current_header.1 = span.end;
            }
        }
        if let Event::End(TagEnd::Heading(header_lvl)) = e {
            // Skip if this header is nested too deeply.
            if let Some((level, id)) = current_header_level.take() {
                assert!(header_lvl as u32 == level);
                let header_span = current_header.take().unwrap();
                let header = content[header_span.0..header_span.1].trim_end();
                let slug = if let Some(slug) = id {
                    // If a fragment is defined, take it as is, not trying to append an extra ID
                    // in case of duplicates (same behavior as mdBook)
                    slug.to_owned()
                } else {
                    let mut slug = normalize_id(header);
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
                    toc_content.push((level, header.to_string(), slug));
                }
            }
            continue;
        }
        if current_header_level.is_none() {
            continue;
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
