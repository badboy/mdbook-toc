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
cargo install mdbook-toc
```

Add it as a preprocessor to your `book.toml`:

```
[preprocessor.toc]
command = "mdbook-toc"
renderer = ["html"]
```

Finally, build your book as normal:

```
mdbook path/to/book
```

## License

MPL. See [LICENSE](LICENSE).  
Copyright (c) 2018 Jan-Erik Rediger <janerik@fnordig.de>
