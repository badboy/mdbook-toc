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


### want make sidebar toc?

If you want to make a sidebar toc, you may do like following

If your page .md, you may put like this

```
# CloudShell

<div class="bsmi-toc">


<!-- toc -->



</div>
```

See these `<div>` ? it is define a div container to mark a class id which may reference a css stylesheet . 

and in your `book.toml`, you may add a additional css 

```toml

[output.html]
additional-css=["toc.css"]
```
Create a `toc.css` file aside of `book.toml` with following css

```css
.bsmi-toc {
    border: 1px solid #ccc;
    float: right;
    width: 340px;
    min-height: 400px;
    position: fixed;
    right: 24px;
    top: 110px;
    background: #fff;
    padding: 10px;
    overflow: auto;
    max-height: 800px
}

.bsmi-toc ul {
    padding-left: 24px
}

.bsmi-toc h2 {
    margin: 4px
}

.bsmi-toc li {
    list-style-type: none
}

```

and build your mdbook, view result.

you may see preview like this:

https://infobsmi.github.io/cloud-wills/tencent_cloud/cloudshell.html


![image](https://user-images.githubusercontent.com/278153/183241122-ef07a3a4-830d-4742-9c08-907b41b6fee1.png)


## License

MPL. See [LICENSE](LICENSE).  
Copyright (c) 2018-2020 Jan-Erik Rediger <janerik@fnordig.de>
