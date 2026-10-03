# md2html

Convert Markdown files to standalone HTML pages.

A single Rust binary with **no dependencies** — not one crate in `Cargo.toml`.
Each `.md` file becomes one self-contained `.html` file: the stylesheet is
embedded in a `<style>` tag, the behaviour is a few lines of inline script, and
there is nothing to install, host or fetch at runtime.


## Demo

**[ravish-vishwakarma.github.io/md2html](https://ravish-vishwakarma.github.io/md2html/)**



## Usage

```
md2html <file.md> [more.md ...]

  -h, --help    Show this message
```

Only `.md` is accepted (case-insensitive). Each input is written to
`<name>.html` **in the current directory**:

```sh
md2html notes.md docs/guide.md      # -> notes.html, guide.html
```

Two things worth knowing about that:

- Output goes to the working directory, not next to the source. Converting
  `docs/guide.md` produces `./guide.html`, not `docs/guide.html`.
- Only the file stem is used, so two sources with the same stem in different
  directories (`a/intro.md` and `b/intro.md`) overwrite each other.

Existing output is overwritten without warning. Each file is reported as it is
written, and the run continues past failures.

### Exit codes

| Code | Meaning |
| :--- | :--- |
| `0` | every input converted |
| `1` | one or more inputs failed (missing file, unreadable, unwritable) |
| `2` | bad usage — no arguments, or an unrecognised option |


### Embedded HTML

Raw HTML passes through untouched: block-level elements become
`Element::Html`, and inline tags, comments and doctypes are preserved verbatim
as inline raw HTML.

> **This is a security trade-off, not an oversight.** Embedded `<script>` and
> event-handler attributes (`onerror`, `onclick`, …) are executed by the
> browser. Only convert Markdown you trust.

A blank line closes an HTML block, matching GitHub-flavoured Markdown. Markdown
syntax *inside* such a block is copied as plain markup.

### Layout

| File | Responsibility |
| :--- | :--- |
| `src/main.rs` | entry point, per-file error handling, exit codes |
| `src/cli.rs` | argument parsing and usage text |
| `src/conversion.rs` | read → parse → render → write pipeline |
| `src/block.rs` | block parsing: lists, tables, callouts, HTML blocks |
| `src/inline.rs` | inline spans: emphasis, links, images, raw HTML |
| `src/html.rs` | document shell, table of contents, and all styling |
| `src/types.rs` | the document model shared by both parsers |
| `src/config.rs` | `md2html.conf` parsing and defaults |
| `src/filesystem.rs` | file IO and output naming |
| `src/helper.rs` | shared string utilities |
| `index.css` | the stylesheet, embedded into every page at build time |

`index.css` is included with `include_str!`, so it is compiled into the binary.
Edit it and rebuild; there is no runtime fetch.

## Limitations

These render as literal Markdown:

- Lists, tables or code fences inside a callout body — every `>` line becomes
  its own paragraph
- Reference-style links and images (`[text][ref]`, `![alt][ref]`)
- Indented four-space code blocks
- Hard line breaks from two trailing spaces
- Syntax highlighting inside fences
- Setext headings (`Title` underlined with `===`)
