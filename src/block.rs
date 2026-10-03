use crate::config::Config;
use crate::inline;
use crate::types::{
    Alignment, Callout, CalloutType, CodeBlock, Element, Heading, HeadingLevel, ListCallout,
    ListItem, NestedList, OrderedList, Quote, Table, Text, UnorderedList,
};

#[cfg(test)]
pub fn parse(content: &str) -> Vec<Element> {
    parse_with(content, &Config::default())
}

pub fn parse_with(content: &str, config: &Config) -> Vec<Element> {
    // Editors on Windows happily save a UTF-8 byte order mark, which would
    // otherwise turn the first line into something unrecognisable.
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let lines: Vec<&str> = content.lines().collect();
    let mut elements: Vec<Element> = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let trimmed = lines[index].trim();

        if trimmed.is_empty() {
            index += 1;
            continue;
        }

        if inline::starts_html(trimmed) {
            let (markup, next) = collect_html(&lines, index);
            elements.push(Element::Html(markup));
            index = next;
            continue;
        }

        if let Some(fence) = open_fence(trimmed) {
            let (code, next) = collect_code(&lines, index, fence.marker);
            elements.push(Element::CodeBlock(CodeBlock {
                language: fence.language,
                code,
            }));
            index = next;
            continue;
        }

        if is_divider(trimmed) {
            elements.push(Element::Divider);
            index += 1;
            continue;
        }

        if let Some(heading) = parse_heading(trimmed) {
            elements.push(Element::Heading(heading));
            index += 1;
            continue;
        }

        if let Some((callout, next)) = parse_callout(&lines, index) {
            elements.push(Element::Callout(callout));
            index = next;
            continue;
        }

        if trimmed.starts_with('>') {
            let (quote, next) = parse_quote(&lines, index);
            elements.push(Element::Quote(quote));
            index = next;
            continue;
        }

        if is_unordered_marker(trimmed) || is_ordered_marker(trimmed) {
            let ordered = is_ordered_marker(trimmed);
            let (items, next) = collect_items(&lines, index, config);
            elements.push(if ordered {
                Element::OrderedList(OrderedList { items })
            } else {
                Element::UnorderedList(UnorderedList { items })
            });
            index = next;
            continue;
        }

        if let Some((table, next)) = parse_table(&lines, index) {
            elements.push(Element::Table(table));
            index = next;
            continue;
        }

        let (paragraph, next) = parse_paragraph(&lines, index);
        elements.push(Element::Text(paragraph));
        index = next;
    }

    elements
}

/// Collects a run of raw HTML. Following CommonMark, only a blank line ends
/// the block: Markdown syntax inside it is copied through untouched.
fn collect_html(lines: &[&str], start: usize) -> (String, usize) {
    let mut body: Vec<&str> = Vec::new();
    let mut index = start;

    while index < lines.len() {
        if index > start && lines[index].trim().is_empty() {
            break;
        }
        body.push(lines[index]);
        index += 1;
    }

    (body.join("\n"), index)
}

struct Fence {
    marker: char,
    language: Option<String>,
}

fn open_fence(trimmed: &str) -> Option<Fence> {
    let marker = trimmed.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let count = trimmed.chars().take_while(|c| *c == marker).count();
    if count < 3 {
        return None;
    }
    let info = trimmed[count..].trim();
    let language = info.split_whitespace().next().map(|word| word.to_string());
    Some(Fence { marker, language })
}

fn closes_fence(trimmed: &str, marker: char) -> bool {
    trimmed.chars().take_while(|c| *c == marker).count() >= 3
        && trimmed.chars().all(|c| c == marker)
}

fn collect_code(lines: &[&str], start: usize, marker: char) -> (String, usize) {
    let mut body: Vec<&str> = Vec::new();
    let mut index = start + 1;

    while index < lines.len() {
        if closes_fence(lines[index].trim(), marker) {
            while body.last().is_some_and(|line| line.trim().is_empty()) {
                body.pop();
            }
            return (body.join("\n"), index + 1);
        }
        body.push(lines[index]);
        index += 1;
    }

    (body.join("\n"), index)
}

fn is_divider(trimmed: &str) -> bool {
    let compact: Vec<char> = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
    compact.len() >= 3 && compact.iter().all(|c| matches!(c, '-' | '*' | '_'))
}

fn is_heading(trimmed: &str) -> bool {
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&hashes) {
        return false;
    }
    trimmed[hashes..]
        .chars()
        .next()
        .is_none_or(|character| character.is_whitespace())
}

fn parse_heading(trimmed: &str) -> Option<Heading> {
    if !is_heading(trimmed) {
        return None;
    }
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    let level = HeadingLevel::from_hashes(hashes)?;
    let body = trimmed[hashes..].trim();
    let stripped = body.trim_end_matches('#');
    let text = if stripped.len() < body.len() && stripped.ends_with(char::is_whitespace) {
        stripped.trim_end()
    } else {
        body
    };
    Some(Heading {
        level,
        text: inline::parse(text),
    })
}

fn parse_callout(lines: &[&str], start: usize) -> Option<(Callout, usize)> {
    let trimmed = lines[start].trim();
    let inner = trimmed.strip_prefix('>')?.trim_start();
    let marker = inner.strip_prefix("[!")?;
    let close = marker.find(']')?;
    let kind = CalloutType::from_marker(&marker[..close])?;
    let raw_title = marker[close + 1..].trim();
    let title = if raw_title.is_empty() {
        inline::parse(kind.label())
    } else {
        inline::parse(raw_title)
    };

    let mut body: Vec<Text> = Vec::new();
    let mut index = start + 1;

    while index < lines.len() {
        let current = lines[index].trim();
        let Some(content) = current.strip_prefix('>') else {
            break;
        };
        let content = content.trim();
        if !content.is_empty() {
            body.push(inline::parse(content));
        }
        index += 1;
    }

    Some((Callout { kind, title, body }, index))
}

fn parse_quote(lines: &[&str], start: usize) -> (Quote, usize) {
    let mut paragraphs: Vec<Text> = Vec::new();
    let mut index = start;

    while index < lines.len() {
        let trimmed = lines[index].trim();
        let Some(content) = trimmed.strip_prefix('>') else {
            break;
        };
        let content = content.trim();
        if !content.is_empty() {
            paragraphs.push(inline::parse(content));
        }
        index += 1;
    }

    (Quote { paragraphs }, index)
}

fn is_unordered_marker(trimmed: &str) -> bool {
    unordered_marker_len(trimmed).is_some()
}

fn is_ordered_marker(trimmed: &str) -> bool {
    ordered_marker_len(trimmed).is_some()
}

fn unordered_marker_len(trimmed: &str) -> Option<usize> {
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            if !rest.trim().is_empty() {
                return Some(marker.len());
            }
        }
    }
    None
}

fn ordered_marker_len(trimmed: &str) -> Option<usize> {
    let digits = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 || digits > 9 {
        return None;
    }
    for marker in [". ", ") "] {
        if let Some(rest) = trimmed[digits..].strip_prefix(marker) {
            if !rest.trim().is_empty() {
                return Some(digits + marker.len());
            }
        }
    }
    None
}

/// One list item line, before the tree is assembled.
struct Entry {
    indent: usize,
    ordered: bool,
    callout: Option<ListCallout>,
    lines: Vec<String>,
}

fn collect_items(lines: &[&str], start: usize, config: &Config) -> (Vec<ListItem>, usize) {
    let (entries, index) = collect_entries(lines, start, config);
    let mut cursor = 0;
    let base = entries.first().map_or(0, |entry| entry.indent);
    (build_level(&entries, &mut cursor, base), index)
}

/// The first item always starts a list. After that, a marker joins the list
/// when it is a sibling of the same kind, or when it is indented deeper.
fn continues_list(entries: &[Entry], ordered: bool, indent: usize) -> bool {
    match entries.first() {
        None => true,
        Some(first) => indent > first.indent || ordered == first.ordered,
    }
}

fn collect_entries(lines: &[&str], start: usize, config: &Config) -> (Vec<Entry>, usize) {
    let mut entries: Vec<Entry> = Vec::new();
    let mut index = start;

    while index < lines.len() {
        let trimmed = lines[index].trim();
        let indent = lines[index].len() - trimmed.len();

        if trimmed.is_empty() {
            if !list_continues(lines, index, entries.last().map_or(0, |entry| entry.indent)) {
                break;
            }
            index += 1;
            continue;
        }

        let marker = marker_at(trimmed);
        match marker {
            Some((is_ordered, length)) => {
                if !continues_list(&entries, is_ordered, indent) {
                    break;
                }

                let (callout, body) = split_callout(trimmed[length..].trim(), config);
                entries.push(Entry {
                    indent,
                    ordered: is_ordered,
                    callout,
                    lines: vec![body],
                });
                index += 1;
                continue;
            }
            None => {
                if entries.is_empty() || starts_block(lines, index) {
                    break;
                }
                // A plain line either continues the item text or opens a
                // nested block, depending on how far it is indented.
                if let Some(entry) = entries.last_mut() {
                    entry.lines.push(trimmed.to_string());
                }
                index += 1;
                continue;
            }
        }
    }

    (entries, index)
}

/// A blank line only keeps the list alive when indented content follows.
fn list_continues(lines: &[&str], index: usize, indent: usize) -> bool {
    let mut next = index + 1;

    while next < lines.len() && lines[next].trim().is_empty() {
        next += 1;
    }

    match lines.get(next) {
        Some(line) if marker_at(line.trim()).is_some() => line.len() - line.trim().len() >= indent,
        Some(line) if !line.trim().is_empty() => {
            line.len() - line.trim().len() > indent && !starts_block(lines, next)
        }
        _ => false,
    }
}

fn marker_at(trimmed: &str) -> Option<(bool, usize)> {
    ordered_marker_len(trimmed)
        .map(|length| (true, length))
        .or_else(|| unordered_marker_len(trimmed).map(|length| (false, length)))
}

/// Peels a callout character off the front of a list item, so `- ! text`
/// becomes a note callout whose body is still ordinary inline Markdown.
fn split_callout(content: &str, config: &Config) -> (Option<ListCallout>, String) {
    let mut characters = content.chars();
    let Some(key) = characters.next() else {
        return (None, content.to_string());
    };

    let tail = &content[key.len_utf8()..];
    let followed_by_space = tail.starts_with(char::is_whitespace);
    let rest = tail.trim_start();
    if !(rest.is_empty() || followed_by_space) {
        return (None, content.to_string());
    }

    let Some(rule) = config.rule(key) else {
        return (None, content.to_string());
    };

    (
        Some(ListCallout {
            kind: rule.kind,
            accent: rule.accent.clone(),
        }),
        rest.to_string(),
    )
}

/// Turns the flat entry list into a tree, following indentation.
fn build_level(entries: &[Entry], cursor: &mut usize, indent: usize) -> Vec<ListItem> {
    let mut items = Vec::new();

    while *cursor < entries.len() {
        let entry = &entries[*cursor];
        if entry.indent < indent {
            break;
        }
        let level = entry.indent;
        *cursor += 1;

        let children = if entries.get(*cursor).is_some_and(|next| next.indent > level) {
            Some(NestedList {
                ordered: entries[*cursor].ordered,
                items: build_level(entries, cursor, entries[*cursor].indent),
            })
        } else {
            None
        };

        items.push(build_item(entry, children));
    }

    items
}

fn build_item(entry: &Entry, children: Option<NestedList>) -> ListItem {
    let raw = entry.lines.join(" ");
    let (task, text) = split_task(&raw);
    ListItem {
        text: inline::parse(text),
        task,
        callout: entry.callout.clone(),
        children,
    }
}

fn split_task(raw: &str) -> (Option<bool>, &str) {
    let bytes = raw.as_bytes();
    if bytes.len() >= 3 && bytes[0] == b'[' && bytes[2] == b']' {
        match bytes[1] {
            b' ' => return (Some(false), raw[3..].trim_start()),
            b'x' | b'X' => return (Some(true), raw[3..].trim_start()),
            _ => {}
        }
    }
    (None, raw)
}

fn parse_table(lines: &[&str], start: usize) -> Option<(Table, usize)> {
    let header_line = lines[start].trim();
    if !header_line.contains('|') {
        return None;
    }
    let delimiter_line = lines.get(start + 1)?.trim();
    if !is_delimiter_row(delimiter_line) {
        return None;
    }

    let header_cells = split_cells(header_line);
    if header_cells.is_empty() {
        return None;
    }

    let alignments: Vec<Alignment> = split_cells(delimiter_line)
        .iter()
        .map(|cell| alignment_of(cell))
        .collect();

    let mut rows: Vec<Vec<Text>> = Vec::new();
    let mut index = start + 2;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed.is_empty() || !trimmed.contains('|') {
            break;
        }
        rows.push(
            split_cells(trimmed)
                .iter()
                .map(|cell| inline::parse(cell))
                .collect(),
        );
        index += 1;
    }

    Some((
        Table {
            header: header_cells
                .iter()
                .map(|cell| inline::parse(cell))
                .collect(),
            alignments,
            rows,
        },
        index,
    ))
}

fn is_delimiter_row(trimmed: &str) -> bool {
    if !trimmed.contains('|') {
        return false;
    }
    let cells = split_cells(trimmed);
    !cells.is_empty()
        && cells
            .iter()
            .all(|cell| cell.contains('-') && cell.chars().all(|c| c == '-' || c == ':'))
}

fn split_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    if inner.trim().is_empty() {
        return Vec::new();
    }
    inner
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn alignment_of(cell: &str) -> Alignment {
    match (cell.starts_with(':'), cell.ends_with(':')) {
        (true, true) => Alignment::Center,
        (true, false) => Alignment::Left,
        (false, true) => Alignment::Right,
        _ => Alignment::Default,
    }
}

fn parse_paragraph(lines: &[&str], start: usize) -> (Text, usize) {
    let mut parts: Vec<String> = Vec::new();
    let mut index = start;

    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed.is_empty() {
            break;
        }
        if index > start && starts_block(lines, index) {
            break;
        }
        parts.push(trimmed.to_string());
        index += 1;
    }

    (inline::parse(&parts.join(" ")), index)
}

fn starts_block(lines: &[&str], index: usize) -> bool {
    let trimmed = lines[index].trim();
    if trimmed.is_empty() {
        return true;
    }
    if open_fence(trimmed).is_some()
        || is_divider(trimmed)
        || is_heading(trimmed)
        || trimmed.starts_with('>')
        || is_unordered_marker(trimmed)
        || is_ordered_marker(trimmed)
    {
        return true;
    }
    trimmed.contains('|')
        && lines
            .get(index + 1)
            .is_some_and(|next| is_delimiter_row(next.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TextType;

    fn items(source: &str) -> Vec<ListItem> {
        let elements = parse_with(source, &Config::default());
        match elements.first() {
            Some(Element::UnorderedList(list)) => list.items.clone(),
            Some(Element::OrderedList(list)) => list.items.clone(),
            other => panic!("expected a list, got {other:?}"),
        }
    }

    fn callout_kind(item: &ListItem) -> Option<CalloutType> {
        item.callout.as_ref().map(|callout| callout.kind)
    }

    #[test]
    fn turns_a_marked_item_into_a_callout() {
        let items = items("- Normal item\n- ! Important item\n");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].callout, None);
        assert_eq!(items[0].text.plain(), "Normal item");
        assert_eq!(callout_kind(&items[1]), Some(CalloutType::Note));
        assert_eq!(items[1].text.plain(), "Important item");
    }

    #[test]
    fn each_marker_maps_to_its_own_kind() {
        let items = items("- ! note\n- ? question\n- * tip\n- + success\n");
        assert_eq!(callout_kind(&items[0]), Some(CalloutType::Note));
        assert_eq!(callout_kind(&items[1]), Some(CalloutType::Question));
        assert_eq!(callout_kind(&items[2]), Some(CalloutType::Tip));
        assert_eq!(callout_kind(&items[3]), Some(CalloutType::Success));
    }

    #[test]
    fn callout_items_keep_inline_markdown() {
        let items = items("- ! a **bold** word and `code`\n");
        assert_eq!(callout_kind(&items[0]), Some(CalloutType::Note));
        assert!(items[0]
            .text
            .spans
            .iter()
            .any(|span| span.styles.contains(&TextType::Bold)));
    }

    #[test]
    fn a_marker_needs_whitespace_after_it() {
        let items = items("- !important\n");
        assert_eq!(items[0].callout, None);
        assert_eq!(items[0].text.plain(), "!important");
    }

    #[test]
    fn an_unconfigured_character_stays_text() {
        let items = items("- @ handle\n");
        assert_eq!(items[0].callout, None);
        assert_eq!(items[0].text.plain(), "@ handle");
    }

    #[test]
    fn ordered_items_can_be_callouts_too() {
        let elements = parse_with("1. ! first\n", &Config::default());
        let Some(Element::OrderedList(list)) = elements.first() else {
            panic!("expected an ordered list");
        };
        assert_eq!(callout_kind(&list.items[0]), Some(CalloutType::Note));
    }

    #[test]
    fn a_callout_can_also_be_a_task() {
        let items = items("- ! [x] done and styled\n");
        assert_eq!(callout_kind(&items[0]), Some(CalloutType::Note));
        assert_eq!(items[0].task, Some(true));
        assert_eq!(items[0].text.plain(), "done and styled");
    }

    #[test]
    fn a_config_file_remaps_the_characters() {
        let config = Config::with_overrides("? danger\n! success\n");
        let elements = parse_with("- ? risky\n- ! good\n- * idea\n", &config);
        let Some(Element::UnorderedList(list)) = elements.first() else {
            panic!("expected a list");
        };
        assert_eq!(callout_kind(&list.items[0]), Some(CalloutType::Danger));
        assert_eq!(callout_kind(&list.items[1]), Some(CalloutType::Success));
        assert_eq!(
            callout_kind(&list.items[2]),
            Some(CalloutType::Tip),
            "characters left out of the config keep their default"
        );
    }

    #[test]
    fn a_config_file_can_set_an_accent_colour() {
        let config = Config::with_overrides("! note #ff9800\n");
        let elements = parse_with("- ! colourful\n", &config);
        let Some(Element::UnorderedList(list)) = elements.first() else {
            panic!("expected a list");
        };
        assert_eq!(
            list.items[0]
                .callout
                .as_ref()
                .and_then(|c| c.accent.as_deref()),
            Some("#ff9800")
        );
    }

    #[test]
    fn nests_sub_items() {
        let items = items("- one\n  - one a\n  - one b\n- two\n");
        assert_eq!(items.len(), 2);
        let children = items[0].children.as_ref().expect("first item nests");
        assert!(!children.ordered);
        assert_eq!(children.items.len(), 2);
        assert_eq!(children.items[0].text.plain(), "one a");
        assert_eq!(items[1].children, None);
    }

    #[test]
    fn nests_three_levels_deep() {
        let items = items("- a\n  - b\n    - c\n");
        let level_two = items[0].children.as_ref().unwrap().items[0]
            .children
            .as_ref()
            .expect("third level nests");
        assert_eq!(level_two.items[0].text.plain(), "c");
    }

    #[test]
    fn a_nested_callout_keeps_its_kind() {
        let items = items("- parent\n  - ! nested note\n");
        let children = items[0].children.as_ref().unwrap();
        assert_eq!(callout_kind(&children.items[0]), Some(CalloutType::Note));
        assert_eq!(children.items[0].text.plain(), "nested note");
    }

    #[test]
    fn a_nested_list_may_switch_to_ordered() {
        let items = items("- one\n  1. first\n  2. second\n");
        let children = items[0].children.as_ref().unwrap();
        assert!(children.ordered);
        assert_eq!(children.items.len(), 2);
    }

    #[test]
    fn nested_tasks_work_too() {
        let items = items("- parent\n  - [ ] sub task\n");
        let children = items[0].children.as_ref().unwrap();
        assert_eq!(children.items[0].task, Some(false));
    }

    #[test]
    fn a_blank_line_between_items_keeps_the_list() {
        let items = items("- one\n\n- two\n");
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn a_blank_line_then_a_paragraph_ends_the_list() {
        let elements = parse("- one\n\nNot an item\n");
        assert_eq!(elements.len(), 2);
        assert!(matches!(elements[1], Element::Text(_)));
    }

    #[test]
    fn a_different_marker_type_starts_a_new_list() {
        let elements = parse("- one\n1. two\n");
        assert_eq!(elements.len(), 2);
    }

    #[test]
    fn continuation_lines_still_join_the_item() {
        let items = items("- one\n  continued here\n- two\n");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text.plain(), "one continued here");
    }

    #[test]
    fn a_byte_order_mark_does_not_hide_the_first_block() {
        let elements = parse("\u{feff}# Title\n\n- one\n");
        assert!(matches!(elements[0], Element::Heading(Heading { .. })));
        assert!(matches!(elements[1], Element::UnorderedList(_)));
    }

    #[test]
    fn collects_a_raw_html_block() {
        let elements = parse("<div class=\"x\">\nhello\n</div>\n");
        assert_eq!(
            elements,
            vec![Element::Html(
                "<div class=\"x\">\nhello\n</div>".to_string()
            )]
        );
    }

    #[test]
    fn html_block_swallows_markdown_lookalikes() {
        let elements = parse("<details>\n- one\n# two\n***\n</details>\n");
        assert_eq!(
            elements,
            vec![Element::Html(
                "<details>\n- one\n# two\n***\n</details>".to_string()
            )]
        );
    }

    #[test]
    fn a_blank_line_ends_the_html_block() {
        let elements = parse("<div>\n\nafter\n");
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0], Element::Html("<div>".to_string()));
        assert!(matches!(elements[1], Element::Text(_)));
    }

    #[test]
    fn markdown_resumes_after_an_html_block() {
        let elements = parse("<hr>\n\n# Heading\n");
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0], Element::Html("<hr>".to_string()));
        assert!(matches!(elements[1], Element::Heading(Heading { .. })));
    }

    #[test]
    fn a_tag_mid_paragraph_stays_inline() {
        let elements = parse("text <b>bold</b> more\n");
        assert_eq!(elements.len(), 1);
        let Element::Text(text) = &elements[0] else {
            panic!("expected text element");
        };
        assert_eq!(text.plain(), "text <b>bold</b> more");
    }

    #[test]
    fn a_comparison_is_not_an_html_block() {
        let elements = parse("3 < 4 is true\n");
        assert_eq!(elements.len(), 1);
        assert!(matches!(elements[0], Element::Text(_)));
    }

    #[test]
    fn a_fence_still_wins_over_html_detection() {
        let elements = parse("```html\n<div>x</div>\n```\n");
        assert_eq!(
            elements,
            vec![Element::CodeBlock(CodeBlock {
                language: Some("html".to_string()),
                code: "<div>x</div>".to_string(),
            })]
        );
    }

    #[test]
    fn parses_headings_and_paragraph() {
        let elements = parse("# Title\n## Sub\n\none\ntwo\n");
        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[0], Element::Heading(Heading { .. })));
        assert!(matches!(elements[1], Element::Heading(Heading { .. })));
        let Element::Text(text) = &elements[2] else {
            panic!("expected text element");
        };
        assert_eq!(text.plain(), "one two");
    }

    #[test]
    fn does_not_treat_hashtag_as_heading() {
        let elements = parse("#hashtag not a heading\n");
        assert!(matches!(elements[0], Element::Text(_)));
    }

    #[test]
    fn strips_only_spaced_closing_hashes() {
        let elements = parse("## C# and Title ###\n");
        let Element::Heading(heading) = &elements[0] else {
            panic!("expected heading");
        };
        assert_eq!(heading.text.plain(), "C# and Title");
    }

    #[test]
    fn parses_divider_variants() {
        let elements = parse("---\n\n***\n\n- - -\n\n___\n");
        assert_eq!(elements.len(), 4);
        assert!(elements.iter().all(|e| *e == Element::Divider));
    }

    #[test]
    fn parses_fenced_code_with_language() {
        let elements = parse("```rust\nfn main() {}\n```\n");
        let Element::CodeBlock(block) = &elements[0] else {
            panic!("expected code block");
        };
        assert_eq!(block.language.as_deref(), Some("rust"));
        assert_eq!(block.code, "fn main() {}");
    }

    #[test]
    fn unterminated_fence_still_parses() {
        let elements = parse("```\nlet x = 1;\n");
        let Element::CodeBlock(block) = &elements[0] else {
            panic!("expected code block");
        };
        assert_eq!(block.language, None);
        assert_eq!(block.code, "let x = 1;");
    }

    #[test]
    fn keeps_markdown_inside_code_fence_verbatim() {
        let elements = parse("```\n# not a heading\n- not a list\n```\n");
        assert_eq!(elements.len(), 1);
        let Element::CodeBlock(block) = &elements[0] else {
            panic!("expected code block");
        };
        assert_eq!(block.code, "# not a heading\n- not a list");
    }

    #[test]
    fn parses_callout_with_body() {
        let elements = parse("> [!warning] Heads up\n> This **matters**.\n");
        let Element::Callout(callout) = &elements[0] else {
            panic!("expected callout");
        };
        assert_eq!(callout.kind, CalloutType::Warning);
        assert_eq!(callout.title.plain(), "Heads up");
        assert_eq!(callout.body.len(), 1);
        assert_eq!(callout.body[0].plain(), "This matters.");
    }

    #[test]
    fn callout_without_title_uses_label() {
        let elements = parse("> [!tip]\n> Body\n");
        let Element::Callout(callout) = &elements[0] else {
            panic!("expected callout");
        };
        assert_eq!(callout.title.plain(), "Tip");
    }

    #[test]
    fn plain_quote_is_blockquote_not_callout() {
        let elements = parse("> just a quote\n");
        let Element::Quote(quote) = &elements[0] else {
            panic!("expected quote");
        };
        assert_eq!(quote.paragraphs[0].plain(), "just a quote");
    }

    #[test]
    fn parses_both_list_flavours_with_tasks() {
        let elements = parse("- one\n- [x] two\n\n1. first\n2. second\n");
        let Element::UnorderedList(list) = &elements[0] else {
            panic!("expected unordered list");
        };
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].task, None);
        assert_eq!(list.items[1].task, Some(true));
        assert_eq!(list.items[1].text.plain(), "two");

        let Element::OrderedList(list) = &elements[1] else {
            panic!("expected ordered list");
        };
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[1].text.plain(), "second");
    }

    #[test]
    fn list_items_join_continuation_lines() {
        let elements = parse("- first line\n  second line\n- next\n");
        let Element::UnorderedList(list) = &elements[0] else {
            panic!("expected unordered list");
        };
        assert_eq!(list.items[0].text.plain(), "first line second line");
        assert_eq!(list.items.len(), 2);
    }

    #[test]
    fn parses_table_with_alignment() {
        let source = "| Name | Qty |\n| :--- | ---: |\n| a | 1 |\n| b | 2 |\n";
        let elements = parse(source);
        let Element::Table(table) = &elements[0] else {
            panic!("expected table");
        };
        assert_eq!(table.header.len(), 2);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.alignment_at(0), Alignment::Left);
        assert_eq!(table.alignment_at(1), Alignment::Right);
        assert_eq!(table.rows[1][0].plain(), "b");
    }

    #[test]
    fn pipe_line_without_delimiter_stays_a_paragraph() {
        let elements = parse("a | b\nnot a table\n");
        assert!(matches!(elements[0], Element::Text(_)));
    }

    #[test]
    fn paragraph_stops_before_next_block() {
        let elements = parse("intro text\n# Heading\nafter\n");
        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[1], Element::Heading(_)));
    }
}
