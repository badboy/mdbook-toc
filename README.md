# mdbook-toc

A preprocessor for [mdbook][] to add inline Table of Contents support.

[mdbook]: https://github.com/rust-lang-nursery/mdBook

It turns this:

```md
<!-- toc -->
```

into a Table of Contents based on all top- and second-level headings of the chapter.

## Installation

If you want to use only this preprocessor, install the tool:

```sh
cargo install mdbook-toc
```

Add it as a preprocessor to your `book.toml`:

```toml
[preprocessor.toc]
command = "mdbook-toc"
renderer = ["html"]
```

Finally, build your book as normal:

```sh
mdbook path/to/book
```

## Custom TOC marker

The default marker is:
```md
<!-- toc -->
```

However that may not be everyone's favorite, if you wish to use a different such as the GitLab marker `[[_TOC_]]`, you must add the following settings to your `book.toml`.

```toml
[preprocessor.toc]
marker = "[[_TOC_]]"
```

And for an example with the GitHub marker, which is:
```md
* auto-gen TOC;
{:toc}
```

The setting is:
```toml
[preprocessor.toc]
marker = "* auto-gen TOC;\n{:toc}"
```

Or with multi-line strings
```toml
[preprocessor.toc]
marker = """* auto-gen TOC;
{:toc}"""
```

## License

MPL. See [LICENSE](LICENSE).  
Copyright (c) 2018-2020 Jan-Erik Rediger <janerik@fnordig.de>
