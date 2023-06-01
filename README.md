# mdbook-toc

A preprocessor for [mdbook][] to add inline Table of Contents support.

[mdbook]: https://github.com/rust-lang-nursery/mdBook

It turns this marker:

```md
<!-- toc -->
```

into a Table of Contents based on headings of the chapter following the marker.

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

## Configuration

### Custom TOC marker

The default marker is:

```md
<!-- toc -->
```

If you wish to use a different marker, such as the GitLab marker `[[_TOC_]]`, you must add the following settings to your `book.toml`.

```toml
[preprocessor.toc]
marker = "[[_TOC_]]"
```

You can also use multi-line markers such as the GitHub marker, which is:

```md
* auto-gen TOC;
{:toc}
```

Configure the string with a newline:

```toml
[preprocessor.toc]
marker = "* auto-gen TOC;\n{:toc}"
```

or with multi-line strings:

```toml
[preprocessor.toc]
marker = """* auto-gen TOC;
{:toc}"""
```

### Maximum header level

By default the ToC will include headings up to level 4 (`####`).
This can be configured in your `book.toml` as follows:

```toml
[preprocessor.toc]
max-level = 4
```

## License

MPL. See [LICENSE](LICENSE).  
Copyright (c) 2018-2020 Jan-Erik Rediger <janerik@fnordig.de>
