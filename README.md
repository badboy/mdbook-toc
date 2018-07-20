# mdbook-toc

A preprocessor for [mdbook][] to add inline Table of Content support.

[mdbook]: https://github.com/rust-lang-nursery/mdBook

It turns this:

```
<!-- toc -->
```

into a Table of Contents based on all top- and second-level headings of the chapter.

## Installation

If you want to use only this preprocessor, install the tool:

```
cargo install --git https://github.com/badboy/mdbook-toc
```

Finally, build your book:

```
mdbook-toc path/to/book
```

### Programmatic use

You can also use this programmatically, e.g. in order to use multiple additional preprocessors.
Add `mdbook-toc` as a dependency in your `Cargo.toml`:

```toml
[dependencies.mdbook-toc]
git = "https://github.com/badboy/mdbook-toc"
```

Then add it to your code:

```rust
extern crate mdbook_toc;

// ...

let mut book = MDBook::load(&book_dir)?;
book.with_preprecessor(mdbook_toc::Toc);
```

## License

MPL. See [LICENSE](LICENSE).  
Copyright (c) 2018 Jan-Erik Rediger <janerik@fnordig.de>
