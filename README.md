# md2html

Convert Markdown files to standalone HTML pages.

A single Rust binary with **no dependencies** — not one crate in `Cargo.toml`.
Each `.md` file becomes one self-contained `.html` file: the stylesheet is
embedded in a `<style>` tag, the behaviour is a few lines of inline script, and
there is nothing to install, host or fetch at runtime.

```
md2html notes.md
notes.md -> notes.html
```

## Demo

**[ravish-vishwakarma.github.io/md2html](https://ravish-vishwakarma.github.io/md2html/)**

That page is `showcase.md` run through this tool and published with GitHub
Pages. It exercises every feature below on a single scroll, and is the quickest
way to see what the output looks like.

## Install

Requires Rust 1.56 or newer (edition 2021). Developed against 1.97.

```sh
git clone https://github.com/Ravish-Vishwakarma/md2html
cd md2html
cargo build --release
```

The binary lands in `target/release/md2html`.

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

## Features

**Inline** — `**bold**`, `*italic*`, `***both***`, `~~strikethrough~~`,
`==highlight==`, `` `code` ``, `[links](url)`, `![images](src)`, autolinks from
`<https://example.com>` and `<someone@example.com>`, and bare `**`/`__`
delimiters.

**Blocks** — ATX headings with a generated table of contents, fenced code blocks
with a language label and a copy button, blockquotes, block callouts, ordered
and unordered lists, tables with per-column alignment, `---` dividers, and
embedded HTML.

**Lists** — task list checkboxes, three levels of nesting, mixed ordered and
unordered children, and per-item callouts (see below).

**Images** — `![alt](src "title")` and `![alt](<src with spaces>)`. A
standalone image is wrapped in a `<figure>` and its alt text is printed
underneath in a bordered caption bar; an image with no alt text is treated as
decorative and gets no caption. Inline images stay at the text size and get no
caption. Images are capped at the column width, load lazily, and fall back to
their alt text if they fail.

**Themes** — light (`#f5f5f0`) and dark (`#0a0a0a`), following the operating
system preference, with a toggle in the sidebar that persists to
`localStorage`. All text meets WCAG AA contrast in both themes.

**Layout** — sidebar with a table of contents, collapsing to a slide-over drawer
on narrow screens. One font stack (JetBrains Mono when installed locally, the
system monospace otherwise) across body text, code and UI.

### Embedded HTML

Raw HTML passes through untouched: block-level elements become
`Element::Html`, and inline tags, comments and doctypes are preserved verbatim
as inline raw HTML.

> **This is a security trade-off, not an oversight.** Embedded `<script>` and
> event-handler attributes (`onerror`, `onclick`, …) are executed by the
> browser. Only convert Markdown you trust.

A blank line closes an HTML block, matching GitHub-flavoured Markdown. Markdown
syntax *inside* such a block is copied as plain markup.

### Callouts

Block callouts use the GitHub alert syntax inside a blockquote:

```markdown
> [!WARNING] Only trust the Markdown you convert
> Scripts run as-is.
```

Thirteen types are recognised — `note`, `info`, `todo`, `tip`, `abstract`,
`question`, `quote`, `example`, `success`, `warning`, `failure`, `danger` and
`bug` — each with an icon, label and accent colour. Common aliases work too
(`hint`, `important`, `tldr`, `faq`, `caution`, `error`, and others).

### List callouts

A list item whose text starts with a marker character becomes a tinted bullet
with an inline icon. The defaults:

| Marker | Type |
| :----- | :--- |
| `!` | note |
| `?` | question |
| `*` | tip |
| `+` | success |
| `~` | warning |

```markdown
- ! Keep the summary above the details
- ? Which parser is this using?
- * Pre-commit the generated HTML
```

Callouts nest, so a marked item can hold a child list.

## Configuration

List-callout markers are configurable through an optional `md2html.conf`. One
rule per line:

```
<char> <type> [accent]
```

```conf
! note
? danger
% bug #e0407a
```

- `<char>` may be **any** character.
- `<type>` is a callout name or alias, case-insensitive.
- `<accent>` is an optional CSS colour, applied as `--callout-accent`.

The file is looked up **beside the Markdown source first**, then in the current
directory. If neither exists, the built-in defaults above are used — a missing
config file is never an error.

Any default marker you do not mention keeps its built-in meaning, so you can
override a single character and leave the rest alone. Unknown callout names are
ignored rather than treated as an error, so one typo cannot silently disable the
whole file. A UTF-8 byte order mark is tolerated.

## Development

```sh
cargo build           # debug binary
cargo test            # 147 tests
cargo fmt             # format
cargo clippy --all-targets
```

`showcase.md` is the fixture and the visual test: it exercises every feature in
one document. Convert it and open the result.

```sh
cargo run --release -- showcase.md
```

### The demo page

`index.html` is that conversion, committed to the repository and served by
GitHub Pages as <https://ravish-vishwakarma.github.io/md2html/>. Publishing
works because Pages serves whatever sits at the root of the published branch.

Note that `.gitignore` excludes `*.html`, so regenerating the page will not show
up as a change unless `index.html` is staged explicitly. To refresh the demo:

```sh
cargo run --release -- showcase.md
copy showcase.html index.html
```

Keep it in step with the fixture — the deployed page is currently byte-identical
to a fresh conversion of `showcase.md`, and it is the first thing most visitors
will see.

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

## Security

Only convert Markdown you trust. Embedded HTML is passed through untouched, so
`<script>` tags and `on*` handlers in a source file execute in the generated
page. Ordinary text is escaped, and titles and table-of-contents entries are
escaped too, so prose cannot inject markup — but raw HTML is a deliberate hole.