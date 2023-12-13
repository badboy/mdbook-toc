# v0.14.2 (2023-12-13)

* Upgrade mdbook

# v0.14.1 (2023-08-05)

* Upgrade mdbook and downgrade toml

# v0.14.0 (2023-08-02)

* Use custom header IDs if present
* Upgrade dependencies

# v0.13.0 (2023-07-18)

* Upgrade dependencies

# v0.12.0 (2023-05-15)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.11.2...0.12.0)

* Normalize to LF line endings so that the default marker works on CRLF documents

# v0.11.2 (2023-02-14)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.11.1...0.11.2)

* Upgrade dependencies and avoid breakage (v0.11.1 was yanked)

# v0.11.1 (2023-02-09)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.11.0...0.11.1)

* Upgrade dependencies

# v0.11.0 (2022-12-15)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.10.0...0.11.0)

* Upgrade dependencies

# v0.10.0 (2022-10-11)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.9.0...0.10.0)

* Bump to mdbook 0.4.21
* Dependency updates

# v0.9.0 (2022-05-26)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.8.0...0.9.0)

* Dependency updates

# v0.8.0 (2022-01-26)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.7.0...0.8.0)

* Avoid roundtripping through pulldown-cmark

# v0.7.0 (2021-07-06)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.6.4...0.7.0)

* Bump pulldown dependency to fix issue with table rendering

# v0.6.4 (2021-06-11)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.6.3...0.6.4)

* Bump to mdbook v0.4.10

# v0.6.3 (2021-04-21)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.6.2...0.6.3)

* Bugfix: Use slug (normalied header) to decide whether to use a different link anchor

# v0.6.2 (2021-04-06)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.6.1...0.6.2)

* Don't write the TOC if no marker was found

# v0.6.1 (2021-01-06)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.6.0...0.6.1)

* Fix Windows release asset

# v0.6.0 (2021-01-06)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.5.1...0.6.0)

* Generate unique slugs for identically named headers
* Allow custom ToC markers through configuration
* Make maximum header level configurable

# v0.5.1 (2020-09-28)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.5.0...0.5.1)

* Avoid broken compilation in CI

# v0.5.0 (2020-09-28)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.4.3...0.5.0)

* Upgrade dependencies, including mdbook itself to 0.4.3

# v0.4.3 (2020-06-24)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.4.2...0.4.3)

* Start normalizing the levels based on the first header's level

# v0.4.2 (2020-06-05)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.4.1...0.4.2)

* Try to fix indentation levels when header levels have holes

# v0.4.1 (2020-05-18)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.4.0...0.4.1)

* Bug fix: newlines around code blocks were fixed upstream

# v0.4.0 (2020-05-06)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.3.0...0.4.0)

* Combine multiple elements inside a single header

# v0.3.0 (2020-04-22)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.2.4...0.3.0)

* Upgrade dependencies to fix nested HTML/markdown

# v0.2.4 (2020-04-08)

[Full changelog](https://github.com/badboy/mdbook-toc/compare/0.2.3...0.2.4)

* Enable the same markdown extensions as mdbook
