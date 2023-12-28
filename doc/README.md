# Ruxgo documentation

This directory contains Ruxgo's documentation: The Ruxgo Book which is built with [mdbook](https://github.com/rust-lang/mdBook) .

## Building the book

Building the book requires mdBook. To get it:

```
$ cargo install mdbook
```

To build the book:

```
$ mdbook build
```

`mdbook` provides a variety of different commands and options to help you work on the book:

- `mdbook build --open`: Build the book and open it in a web browser.
- `mdbook serve`: Launches a web server on localhost. It also automatically rebuilds the book whenever any file changes and automatically reloads your web browser.

The book contents are driven by the `SUMMARY.md` file, and every file must be linked there.