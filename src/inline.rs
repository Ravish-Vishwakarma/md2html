use crate::helper::{escape_html, unescape_html};
use crate::types::{Text, TextSpan, TextType};

pub fn parse(raw: &str) -> Text {
    Text {
        spans: parse_spans(raw, &[]),
    }
}

pub fn starts_html(trimmed: &str) -> bool {
    html_at(trimmed, 0).is_some()
}

fn parse_spans(source: &str, styles: &[TextType]) -> Vec<TextSpan> {
    let mut spans: Vec<TextSpan> = Vec::new();
    let mut buffer = String::new();
    let mut index = 0;

    while index < source.len() {
        let rest = &source[index..];

        if let Some(stripped) = rest.strip_prefix('\\') {
            if let Some(escaped) = stripped.chars().next() {
                buffer.push(escaped);
                index += 1 + escaped.len_utf8();
                continue;
            }
        }

        if let Some((markup, consumed)) = html_at(source, index) {
            flush(&mut spans, &mut buffer, styles);
            let mut raw_styles = styles.to_vec();
            raw_styles.push(TextType::RawHtml);
            spans.push(TextSpan {
                text: markup,
                styles: raw_styles,
            });
            index = consumed;
            continue;
        }

        if rest.starts_with('[') || rest.starts_with("![") {
            if let Some((link, consumed)) = parse_link(source, index, styles) {
                flush(&mut spans, &mut buffer, styles);
                spans.extend(link);
                index = consumed;
                continue;
            }
        }

        if let Some((code, consumed)) = parse_inline_code(source, index, styles) {
            flush(&mut spans, &mut buffer, styles);
            spans.push(code);
            index = consumed;
            continue;
        }

        let mut matched = false;
        if let Some(marker) = rest.chars().next() {
            let run = rest.chars().take_while(|c| *c == marker).count();
            if let Some(marker_styles) = styles_for(marker, run) {
                let inner_start = index + run;
                let close = find_closing_run(source, inner_start, marker, run)
                    .map(|start| (start, start + run))
                    .filter(|&(start, end)| {
                        is_inner(&source[inner_start..start])
                            && (marker != '_' || is_outside_word(source, index, end))
                    });
                if let Some((close_start, close_end)) = close {
                    flush(&mut spans, &mut buffer, styles);
                    let mut nested = styles.to_vec();
                    nested.extend(marker_styles);
                    spans.extend(parse_spans(&source[inner_start..close_start], &nested));
                    index = close_end;
                    matched = true;
                }
            }
        }
        if matched {
            continue;
        }

        let character = rest.chars().next().expect("index is inside source");
        buffer.push(character);
        index += character.len_utf8();
    }

    flush(&mut spans, &mut buffer, styles);
    spans
}

fn styles_for(marker: char, run: usize) -> Option<Vec<TextType>> {
    match (marker, run) {
        ('*', 1) | ('_', 1) => Some(vec![TextType::Italic]),
        ('*', 2) | ('_', 2) => Some(vec![TextType::Bold]),
        ('*', 3) | ('_', 3) => Some(vec![TextType::Bold, TextType::Italic]),
        ('~', 2) => Some(vec![TextType::Strikethrough]),
        ('=', 2) => Some(vec![TextType::Highlight]),
        _ => None,
    }
}

fn is_inner(inner: &str) -> bool {
    let trimmed = inner.trim();
    !trimmed.is_empty() && trimmed.len() == inner.len()
}

fn find_closing_run(source: &str, from: usize, marker: char, run: usize) -> Option<usize> {
    let mut index = from;

    while index < source.len() {
        if source[index..].starts_with(marker) {
            let length = source[index..].chars().take_while(|c| *c == marker).count();
            let opens = source[..index].ends_with(marker);
            if length == run && !opens {
                return Some(index);
            }
            index += length;
        } else {
            index += source[index..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(1);
        }
    }

    None
}

fn html_at(source: &str, index: usize) -> Option<(String, usize)> {
    let rest = &source[index..];
    if !rest.starts_with('<') {
        return None;
    }

    if rest.starts_with("<!--") {
        let end = rest.find("-->")? + 3;
        return Some((rest[..end].to_string(), index + end));
    }

    let bytes = rest.as_bytes();
    match bytes.get(1) {
        Some(b'!') | Some(b'?') => {
            let end = rest.find('>')? + 1;
            return Some((rest[..end].to_string(), index + end));
        }
        _ => {}
    }

    if let Some(link) = autolink(rest) {
        return Some((link, index + 1 + link_source_len(rest)));
    }

    let mut cursor = 1;
    if bytes.get(1) == Some(&b'/') {
        cursor = 2;
    }
    let name_start = cursor;
    while cursor < bytes.len()
        && (bytes[cursor].is_ascii_alphanumeric() || matches!(bytes[cursor], b'-' | b':'))
    {
        cursor += 1;
    }
    if cursor == name_start {
        return None;
    }
    match bytes.get(cursor) {
        Some(b'>') | Some(b'/') | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') => {}
        _ => return None,
    }
    let end = rest[cursor..].find('>')? + cursor + 1;
    Some((rest[..end].to_string(), index + end))
}

fn link_source_len(rest: &str) -> usize {
    rest.find('>').map(|end| end + 1).unwrap_or(rest.len())
}

fn autolink(rest: &str) -> Option<String> {
    let body = rest.strip_prefix('<')?;
    let end = body.find('>')?;
    let target = &body[..end];

    if target.is_empty() || target.contains(char::is_whitespace) || target.contains(['<', '>', '"'])
    {
        return None;
    }

    let scheme = target.split_once(':').map(|(scheme, _)| scheme);
    let is_uri = scheme.is_some_and(|scheme| {
        !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'))
    });
    let is_email = !is_uri && target.contains('@') && target.contains('.');

    if !is_uri && !is_email {
        return None;
    }

    let href = if is_uri {
        target.to_string()
    } else {
        format!("mailto:{target}")
    };
    Some(format!(
        "<a href=\"{}\">{}</a>",
        escape_html(&href),
        escape_html(target)
    ))
}

fn parse_inline_code(source: &str, index: usize, styles: &[TextType]) -> Option<(TextSpan, usize)> {
    let fence = if source[index..].starts_with("``") {
        "``"
    } else if source[index..].starts_with('`') {
        "`"
    } else {
        return None;
    };

    let inner_start = index + fence.len();
    let offset = source[inner_start..].find(fence)?;
    let inner_end = inner_start + offset;

    let mut nested = styles.to_vec();
    nested.push(TextType::InlineCode);

    Some((
        TextSpan {
            text: escape_html(&source[inner_start..inner_end]),
            styles: nested,
        },
        inner_end + fence.len(),
    ))
}

fn parse_link(source: &str, index: usize, styles: &[TextType]) -> Option<(Vec<TextSpan>, usize)> {
    let is_image = source[index..].starts_with("![");
    let bracket = if is_image { index + 1 } else { index };

    if !source[bracket..].starts_with('[') {
        return None;
    }

    let label_start = bracket + 1;
    let label_end = find_matching(source, label_start, b'[', b']')?;
    let target_start = label_end + 1;
    if !source[target_start..].starts_with('(') {
        return None;
    }
    let target_end = find_matching(source, target_start + 1, b'(', b')')?;

    let label = &source[label_start..label_end];
    let (target, title) = split_destination(source[target_start + 1..target_end].trim());

    if is_image {
        let mut nested = styles.to_vec();
        nested.push(TextType::Image {
            src: escape_html(&target),
            title,
        });
        return Some((
            vec![TextSpan {
                text: escape_html(&label_text(label)),
                styles: nested,
            }],
            target_end + 1,
        ));
    }

    let mut nested = styles.to_vec();
    nested.push(TextType::Link(escape_html(&target)));

    let mut spans = parse_spans(label, &nested);
    if spans.is_empty() {
        spans.push(TextSpan {
            text: escape_html(label),
            styles: nested,
        });
    }

    Some((spans, target_end + 1))
}

/// Splits a link or image destination into its URL and optional title,
/// accepting both `url "title"` and `<url with spaces>` forms.
fn split_destination(raw: &str) -> (String, Option<String>) {
    if let Some(rest) = raw.strip_prefix('<') {
        if let Some(close) = rest.find('>') {
            return (rest[..close].to_string(), parse_title(&rest[close + 1..]));
        }
    }

    match raw.split_once(char::is_whitespace) {
        Some((destination, title)) => (destination.to_string(), parse_title(title)),
        None => (raw.to_string(), None),
    }
}

fn parse_title(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let mut characters = trimmed.chars();
    let first = characters.next()?;
    let last = characters.next_back()?;
    let quoted = matches!((first, last), ('"', '"') | ('\'', '\'') | ('(', ')'));

    if !quoted || trimmed.chars().count() < 2 {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    (!inner.is_empty()).then(|| escape_html(inner))
}

/// Reduces an image label to readable plain text for the `alt` attribute.
fn label_text(label: &str) -> String {
    let joined: String = parse_spans(label, &[])
        .iter()
        .map(|span| span.text.as_str())
        .collect();
    unescape_html(&joined)
}

fn find_matching(source: &str, start: usize, open: u8, close: u8) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 1i32;
    let mut index = start;

    while index < bytes.len() {
        let byte = bytes[index];
        if byte == open {
            depth += 1;
        } else if byte == close {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
        index += 1;
    }

    None
}

fn is_outside_word(source: &str, open_start: usize, close_end: usize) -> bool {
    let before = source[..open_start]
        .chars()
        .next_back()
        .is_none_or(|character| !character.is_alphanumeric());
    let after = source[close_end..]
        .chars()
        .next()
        .is_none_or(|character| !character.is_alphanumeric());
    before && after
}

fn flush(spans: &mut Vec<TextSpan>, buffer: &mut String, styles: &[TextType]) {
    if buffer.is_empty() {
        return;
    }
    spans.push(TextSpan {
        text: escape_html(buffer),
        styles: styles.to_vec(),
    });
    buffer.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn styles(source: &str) -> Vec<(String, Vec<TextType>)> {
        parse(source)
            .spans
            .into_iter()
            .map(|span| (span.text, span.styles))
            .collect()
    }

    fn image(src: &str, title: Option<&str>) -> TextType {
        TextType::Image {
            src: src.to_string(),
            title: title.map(str::to_string),
        }
    }

    #[test]
    fn parses_an_image_with_its_alt_text() {
        assert_eq!(
            styles("![A cat](https://example.com/cat.jpg)"),
            vec![(
                "A cat".to_string(),
                vec![image("https://example.com/cat.jpg", None)]
            )]
        );
    }

    #[test]
    fn image_alt_may_be_empty() {
        assert_eq!(
            styles("![](https://example.com/cat.jpg)"),
            vec![(
                String::new(),
                vec![image("https://example.com/cat.jpg", None)]
            )]
        );
    }

    #[test]
    fn parses_an_image_title() {
        assert_eq!(
            styles("![cat](cat.jpg \"A sleeping cat\")"),
            vec![(
                "cat".to_string(),
                vec![image("cat.jpg", Some("A sleeping cat"))]
            )]
        );
    }

    #[test]
    fn image_alt_is_reduced_to_plain_text() {
        assert_eq!(
            styles("![a **bold** & <b>cat</b>](cat.jpg)"),
            vec![(
                "a bold &amp; &lt;b&gt;cat&lt;/b&gt;".to_string(),
                vec![image("cat.jpg", None)]
            )]
        );
        assert_eq!(
            parse("![a & <b>cat</b>](cat.jpg)").plain(),
            "a & <b>cat</b>"
        );
    }

    #[test]
    fn escapes_ampersands_in_image_sources() {
        assert_eq!(
            styles("![x](https://example.com/i.png?a=1&b=2)"),
            vec![(
                "x".to_string(),
                vec![image("https://example.com/i.png?a=1&amp;b=2", None)]
            )]
        );
    }

    #[test]
    fn supports_angle_bracket_destinations() {
        assert_eq!(
            styles("![x](<https://example.com/a b.png> \"t\")"),
            vec![(
                "x".to_string(),
                vec![image("https://example.com/a b.png", Some("t"))]
            )]
        );
    }

    #[test]
    fn a_stray_bang_stays_text() {
        assert_eq!(
            styles("Wow! [link](x.md)"),
            vec![
                ("Wow! ".to_string(), vec![]),
                ("link".to_string(), vec![TextType::Link("x.md".to_string())]),
            ]
        );
    }

    #[test]
    fn an_unclosed_image_stays_text() {
        assert_eq!(
            styles("![alt](unclosed"),
            vec![("![alt](unclosed".to_string(), vec![])]
        );
    }

    #[test]
    fn images_work_inside_emphasis() {
        assert_eq!(
            styles("**![a](b.png)**"),
            vec![("a".to_string(), vec![TextType::Bold, image("b.png", None)])]
        );
    }

    #[test]
    fn plain_text_falls_back_to_the_alt_text() {
        assert_eq!(parse("![A cat](cat.jpg)").plain(), "A cat");
    }

    #[test]
    fn keeps_plain_text_untouched() {
        assert_eq!(
            styles("hello world"),
            vec![("hello world".to_string(), vec![])]
        );
    }

    #[test]
    fn parses_bold_and_italic() {
        assert_eq!(
            styles("a **b** c"),
            vec![
                ("a ".to_string(), vec![]),
                ("b".to_string(), vec![TextType::Bold]),
                (" c".to_string(), vec![]),
            ]
        );
        assert_eq!(
            styles("*hi*"),
            vec![("hi".to_string(), vec![TextType::Italic])]
        );
    }

    #[test]
    fn parses_strikethrough_and_highlight() {
        assert_eq!(
            styles("~~gone~~"),
            vec![("gone".to_string(), vec![TextType::Strikethrough])]
        );
        assert_eq!(
            styles("==kept=="),
            vec![("kept".to_string(), vec![TextType::Highlight])]
        );
    }

    #[test]
    fn keeps_underscores_inside_words() {
        assert_eq!(
            styles("some_variable_name"),
            vec![("some_variable_name".to_string(), vec![])]
        );
    }

    #[test]
    fn inline_code_is_opaque() {
        assert_eq!(
            styles("use `a **b** <i> c` here"),
            vec![
                ("use ".to_string(), vec![]),
                (
                    "a **b** &lt;i&gt; c".to_string(),
                    vec![TextType::InlineCode]
                ),
                (" here".to_string(), vec![]),
            ]
        );
    }

    #[test]
    fn escapes_html_when_it_is_not_a_tag() {
        assert_eq!(
            styles("5 < 6 & \"quoted\""),
            vec![("5 &lt; 6 &amp; &quot;quoted&quot;".to_string(), vec![])]
        );
    }

    #[test]
    fn passes_inline_tags_through_verbatim() {
        assert_eq!(
            styles("H<sub>2</sub>O and a <b>bold</b> word"),
            vec![
                ("H".to_string(), vec![]),
                ("<sub>".to_string(), vec![TextType::RawHtml]),
                ("2".to_string(), vec![]),
                ("</sub>".to_string(), vec![TextType::RawHtml]),
                ("O and a ".to_string(), vec![]),
                ("<b>".to_string(), vec![TextType::RawHtml]),
                ("bold".to_string(), vec![]),
                ("</b>".to_string(), vec![TextType::RawHtml]),
                (" word".to_string(), vec![]),
            ]
        );
    }

    #[test]
    fn handles_void_and_self_closing_tags() {
        assert_eq!(
            styles("a<br/>b<br>c"),
            vec![
                ("a".to_string(), vec![]),
                ("<br/>".to_string(), vec![TextType::RawHtml]),
                ("b".to_string(), vec![]),
                ("<br>".to_string(), vec![TextType::RawHtml]),
                ("c".to_string(), vec![]),
            ]
        );
    }

    #[test]
    fn passes_comments_and_declarations_through() {
        assert_eq!(
            styles("<!-- note --><!DOCTYPE html>"),
            vec![
                ("<!-- note -->".to_string(), vec![TextType::RawHtml]),
                ("<!DOCTYPE html>".to_string(), vec![TextType::RawHtml]),
            ]
        );
    }

    #[test]
    fn attributes_with_quotes_survive() {
        assert_eq!(
            styles(r#"<span data-x="1" class='y'>z"#),
            vec![
                (
                    "<span data-x=\"1\" class='y'>".to_string(),
                    vec![TextType::RawHtml]
                ),
                ("z".to_string(), vec![]),
            ]
        );
    }

    #[test]
    fn autolinks_become_anchors() {
        assert_eq!(
            styles("<https://example.com/a?b=1&c=2>"),
            vec![(
                "<a href=\"https://example.com/a?b=1&amp;c=2\">https://example.com/a?b=1&amp;c=2</a>"
                    .to_string(),
                vec![TextType::RawHtml]
            )]
        );
        assert_eq!(
            styles("<me@example.com>"),
            vec![(
                "<a href=\"mailto:me@example.com\">me@example.com</a>".to_string(),
                vec![TextType::RawHtml]
            )]
        );
    }

    #[test]
    fn lone_angle_brackets_stay_text() {
        assert_eq!(
            styles("<not a tag"),
            vec![("&lt;not a tag".to_string(), vec![])]
        );
        assert_eq!(styles("a <3 b"), vec![("a &lt;3 b".to_string(), vec![])]);
    }

    #[test]
    fn tags_work_inside_strong_emphasis() {
        assert_eq!(
            styles("**a <b>b</b>**"),
            vec![
                ("a ".to_string(), vec![TextType::Bold]),
                ("<b>".to_string(), vec![TextType::Bold, TextType::RawHtml]),
                ("b".to_string(), vec![TextType::Bold]),
                ("</b>".to_string(), vec![TextType::Bold, TextType::RawHtml]),
            ]
        );
    }

    #[test]
    fn parses_links_with_title() {
        assert_eq!(
            styles("[docs](https://example.com \"Docs\")"),
            vec![(
                "docs".to_string(),
                vec![TextType::Link("https://example.com".to_string())]
            )]
        );
    }

    #[test]
    fn escapes_ampersands_in_link_targets() {
        assert_eq!(
            styles("[q](https://example.com/?a=1&b=2)"),
            vec![(
                "q".to_string(),
                vec![TextType::Link(
                    "https://example.com/?a=1&amp;b=2".to_string()
                )]
            )]
        );
    }

    #[test]
    fn supports_nested_styles() {
        assert_eq!(
            styles("**bold with `code`**"),
            vec![
                ("bold with ".to_string(), vec![TextType::Bold]),
                (
                    "code".to_string(),
                    vec![TextType::Bold, TextType::InlineCode]
                )
            ]
        );
    }

    #[test]
    fn honours_backslash_escapes() {
        assert_eq!(
            styles("\\*not italic\\*"),
            vec![("*not italic*".to_string(), vec![])]
        );
        assert_eq!(styles("\\<b>"), vec![("&lt;b&gt;".to_string(), vec![])]);
    }

    #[test]
    fn leaves_unmatched_markers_as_text() {
        assert_eq!(styles("2 * 3 = 6"), vec![("2 * 3 = 6".to_string(), vec![])]);
        assert_eq!(
            styles("**unclosed"),
            vec![("**unclosed".to_string(), vec![])]
        );
    }

    #[test]
    fn triple_run_becomes_bold_italic() {
        assert_eq!(
            styles("***both***"),
            vec![("both".to_string(), vec![TextType::Bold, TextType::Italic])]
        );
    }

    #[test]
    fn a_stray_marker_does_not_cascade() {
        assert_eq!(
            styles("***a** plain ***text*** rest"),
            vec![
                ("*".to_string(), vec![]),
                ("a".to_string(), vec![TextType::Bold]),
                (" plain ".to_string(), vec![]),
                ("text".to_string(), vec![TextType::Bold, TextType::Italic]),
                (" rest".to_string(), vec![]),
            ]
        );
    }

    #[test]
    fn markers_glued_to_whitespace_are_literal() {
        assert_eq!(
            styles("a * not emphasis * b"),
            vec![("a * not emphasis * b".to_string(), vec![])]
        );
    }

    #[test]
    fn underscores_work_at_word_boundaries() {
        assert_eq!(
            styles("_emphasised_ word"),
            vec![
                ("emphasised".to_string(), vec![TextType::Italic]),
                (" word".to_string(), vec![]),
            ]
        );
        assert_eq!(
            styles("__strong__ word"),
            vec![
                ("strong".to_string(), vec![TextType::Bold]),
                (" word".to_string(), vec![]),
            ]
        );
    }

    #[test]
    fn plain_round_trips_escaped_text() {
        assert_eq!(parse("a & b < c \"d\"").plain(), "a & b < c \"d\"");
        assert_eq!(parse("<b>x</b>").plain(), "<b>x</b>");
    }
}
