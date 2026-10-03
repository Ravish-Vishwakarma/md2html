use std::collections::HashMap;

use crate::helper::escape_html;
use crate::types::{
    Alignment, Callout, Element, HeadingLevel, ListCallout, ListItem, Quote, Table, Text, TextSpan,
    TextType,
};

const STYLESHEET: &str = include_str!("../index.css");

const THEME_SCRIPT: &str = r#"(function () {
  var theme = null;
  try { theme = localStorage.getItem('md2html-theme'); } catch (error) {}
  if (theme !== 'light' && theme !== 'dark') {
    theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  document.documentElement.dataset.theme = theme;
})();"#;

const APP_SCRIPT: &str = r#"(function () {
  var root = document.documentElement;
  var toggles = document.querySelectorAll('[data-theme-toggle]');

  Array.prototype.forEach.call(toggles, function (toggle) {
    toggle.addEventListener('click', function () {
      var next = root.dataset.theme === 'dark' ? 'light' : 'dark';
      root.dataset.theme = next;
      try { localStorage.setItem('md2html-theme', next); } catch (error) {}
      Array.prototype.forEach.call(toggles, function (other) {
        other.setAttribute(
          'aria-label',
          next === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'
        );
      });
    });
  });

  function copy(text, done) {
    if (navigator.clipboard && window.isSecureContext) {
      navigator.clipboard.writeText(text).then(done, function () {});
      return;
    }
    var area = document.createElement('textarea');
    area.value = text;
    area.setAttribute('readonly', '');
    area.style.position = 'fixed';
    area.style.opacity = '0';
    document.body.appendChild(area);
    area.select();
    try { document.execCommand('copy'); done(); } catch (error) {}
    document.body.removeChild(area);
  }

  Array.prototype.forEach.call(
    document.querySelectorAll('[data-copy]'),
    function (button) {
      button.addEventListener('click', function () {
        var block = button.closest('.code-block');
        var code = block && block.querySelector('code');
        if (!code) return;
        copy(code.textContent, function () {
          button.textContent = 'Copied';
          button.classList.add('is-copied');
          window.setTimeout(function () {
            button.textContent = 'Copy';
            button.classList.remove('is-copied');
          }, 1600);
        });
      });
    }
  );

  var links = document.querySelectorAll('.toc a');
  if (links.length && 'IntersectionObserver' in window) {
    var bySlug = {};
    Array.prototype.forEach.call(links, function (link) {
      bySlug[link.getAttribute('href').slice(1)] = link;
    });
    var observer = new IntersectionObserver(
      function (entries) {
        entries.forEach(function (entry) {
          if (!entry.isIntersecting) return;
          Array.prototype.forEach.call(links, function (link) {
            link.classList.remove('is-active');
          });
          var active = bySlug[entry.target.id];
          if (active) active.classList.add('is-active');
        });
      },
      { rootMargin: '0px 0px -70% 0px' }
    );
    Array.prototype.forEach.call(
      document.querySelectorAll('.heading[id]'),
      function (heading) {
        observer.observe(heading);
      }
    );
  }

  Array.prototype.forEach.call(document.images, function (img) {
    img.addEventListener('error', function () {
      var figure = img.closest('.image');
      var holder = figure || img;
      holder.classList.add('is-broken');
      holder.setAttribute(
        'data-broken-alt',
        img.getAttribute('alt') || img.getAttribute('src') || 'unknown image'
      );
    });
  });
})();"#;

const MOON_ICON: &str = r#"<svg class="icon-moon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20.5 14.5A8.5 8.5 0 1 1 9.5 3.5a7 7 0 0 0 11 11z"/></svg>"#;

const SUN_ICON: &str = r#"<svg class="icon-sun" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="4.2"/><path d="M12 2.6v2.1M12 19.3v2.1M4.3 4.3l1.5 1.5M18.2 18.2l1.5 1.5M2.6 12h2.1M19.3 12h2.1M4.3 19.7l1.5-1.5M18.2 5.8l1.5-1.5"/></svg>"#;

pub fn render_document(elements: &[Element], fallback_title: &str) -> String {
    let title = document_title(elements, fallback_title);
    let mut renderer = Renderer::default();
    let body = renderer.render(elements);
    let toc = renderer.render_toc();

    format!(
        "<!DOCTYPE html>\n\
<html lang=\"en\">\n\
<head>\n\
<meta charset=\"UTF-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<meta name=\"generator\" content=\"md2html\">\n\
<meta name=\"color-scheme\" content=\"light dark\">\n\
<title>{title}</title>\n\
<script>{theme_script}</script>\n\
<style>\n{stylesheet}\n</style>\n\
</head>\n\
<body>\n\
<div class=\"layout\">\n\
<aside class=\"sidebar\">\
<div class=\"sidebar-inner\">\
<div class=\"sidebar-head\">\
<span class=\"toc-title\">Contents</span>\
<button class=\"theme-toggle\" type=\"button\" data-theme-toggle \
aria-label=\"Toggle colour theme\" title=\"Toggle colour theme\">{moon}{sun}</button>\
</div>\
{toc}\
</div>\
</aside>\n\
<main class=\"page\">\
{body}\
<footer class=\"page-footer\">Generated by <strong>md2html</strong></footer>\
</main>\n\
</div>\n\
<script>{app_script}</script>\n\
</body>\n\
</html>\n",
        title = title,
        theme_script = THEME_SCRIPT,
        stylesheet = STYLESHEET.trim(),
        moon = MOON_ICON,
        sun = SUN_ICON,
        toc = toc,
        body = body,
        app_script = APP_SCRIPT,
    )
}

fn document_title(elements: &[Element], fallback: &str) -> String {
    elements
        .iter()
        .find_map(|element| match element {
            Element::Heading(heading) if heading.level == HeadingLevel::H1 => {
                Some(escape_html(&heading.text.plain()))
            }
            _ => None,
        })
        .unwrap_or_else(|| escape_html(fallback))
}

#[derive(Default)]
struct Renderer {
    slugs: HashMap<String, usize>,
    toc: Vec<(usize, String, String)>,
}

impl Renderer {
    fn render(&mut self, elements: &[Element]) -> String {
        let mut output = String::new();

        for element in elements {
            match element {
                Element::Heading(heading) => {
                    let tag = heading.level.tag();
                    let slug = self.slug(&heading.text.plain());
                    let depth = heading.level.depth();
                    if depth == 2 || depth == 3 {
                        self.toc
                            .push((depth, escape_html(&heading.text.plain()), slug.clone()));
                    }
                    output.push_str(&format!(
                        "<{tag} id=\"{slug}\" class=\"heading\">\
<span class=\"heading-text\">{}</span>\
<a class=\"anchor\" href=\"#{slug}\" aria-label=\"Link to this section\">#</a></{tag}>\n",
                        render_spans(&heading.text.spans)
                    ));
                }
                Element::Text(text) => match render_lone_image(text) {
                    Some(figure) => output.push_str(&figure),
                    None => output.push_str(&format!("<p>{}</p>\n", render_spans(&text.spans))),
                },
                Element::Divider => output.push_str("<hr class=\"divider\">\n"),
                Element::Html(markup) => {
                    output.push_str(markup);
                    if !markup.ends_with('\n') {
                        output.push('\n');
                    }
                }
                Element::Quote(quote) => render_quote(quote, &mut output),
                Element::CodeBlock(block) => {
                    let language = block.language.as_deref().unwrap_or("text");
                    output.push_str(&format!(
                        "<div class=\"code-block\">\
<div class=\"code-header\">\
<span class=\"code-language\">{label}</span>\
<button class=\"copy-button\" type=\"button\" data-copy>Copy</button>\
</div>\
<pre><code>{code}</code></pre></div>\n",
                        label = escape_html(language),
                        code = escape_html(&block.code),
                    ));
                }
                Element::UnorderedList(list) => render_list(&list.items, "ul", &mut output),
                Element::OrderedList(list) => render_list(&list.items, "ol", &mut output),
                Element::Table(table) => render_table(table, &mut output),
                Element::Callout(callout) => render_callout(callout, &mut output),
            }
        }

        output
    }

    fn slug(&mut self, text: &str) -> String {
        let slugified = slugify(text);
        let base = if slugified.is_empty() {
            "section".to_string()
        } else {
            slugified
        };

        let count = self.slugs.entry(base.clone()).or_insert(0);
        let slug = if *count == 0 {
            base.clone()
        } else {
            format!("{base}-{count}")
        };
        *count += 1;
        slug
    }

    fn render_toc(&self) -> String {
        if self.toc.is_empty() {
            return String::new();
        }

        let mut output = String::from("<nav class=\"toc\" aria-label=\"Table of contents\"><ul>\n");
        for (depth, text, slug) in &self.toc {
            output.push_str(&format!(
                "<li class=\"depth-{depth}\"><a href=\"#{slug}\">{text}</a></li>\n"
            ));
        }
        output.push_str("</ul></nav>");
        output
    }
}

fn render_spans(spans: &[TextSpan]) -> String {
    let mut output = String::new();

    for span in spans {
        let alt = span.text.clone();
        let mut html = alt.clone();
        for style in &span.styles {
            html = match style {
                TextType::Bold => format!("<strong>{html}</strong>"),
                TextType::Italic => format!("<em>{html}</em>"),
                TextType::Highlight => format!("<mark>{html}</mark>"),
                TextType::InlineCode => format!("<code>{html}</code>"),
                TextType::Strikethrough => format!("<del>{html}</del>"),
                TextType::Link(target) => {
                    format!("<a href=\"{target}\" rel=\"noopener\">{html}</a>")
                }
                TextType::Image { src, title } => render_image_tag(src, title.as_deref(), &alt),
                TextType::RawHtml => html,
            };
        }
        output.push_str(&html);
    }

    output
}

fn render_image_tag(src: &str, title: Option<&str>, alt: &str) -> String {
    format!(
        "<img src=\"{src}\" alt=\"{alt}\"{} loading=\"lazy\" decoding=\"async\">",
        title
            .map(|value| format!(" title=\"{value}\""))
            .unwrap_or_default()
    )
}

/// A standalone image, with its alt text shown underneath inside the frame.
/// An empty alt text means a decorative image, so no caption area is drawn.
fn render_image_figure(src: &str, title: Option<&str>, alt: &str) -> String {
    let caption = if alt.trim().is_empty() {
        String::new()
    } else {
        format!("<figcaption>{alt}</figcaption>")
    };

    format!(
        "<figure class=\"image\">{}{caption}</figure>\n",
        render_image_tag(src, title, alt)
    )
}

/// A paragraph holding nothing but an image becomes a captioned figure.
fn render_lone_image(text: &Text) -> Option<String> {
    if text.spans.len() != 1 {
        return None;
    }

    let span = &text.spans[0];
    let (src, title) = span.styles.iter().find_map(TextType::image)?;

    Some(render_image_figure(src, title, &span.text))
}

fn render_callout(callout: &Callout, output: &mut String) {
    output.push_str(&format!(
        "<aside class=\"callout callout-{kind}\">\
<div class=\"callout-icon\">{icon}</div>\
<div class=\"callout-content\">\
<div class=\"callout-title\">{title}</div>\
<div class=\"callout-body\">",
        kind = callout.kind.css(),
        icon = callout.kind.icon(),
        title = render_spans(&callout.title.spans),
    ));
    for paragraph in &callout.body {
        output.push_str(&format!("<p>{}</p>\n", render_spans(&paragraph.spans)));
    }
    output.push_str("</div>\n</div>\n</aside>\n");
}

fn render_list(items: &[ListItem], tag: &str, output: &mut String) {
    output.push_str(&format!("<{tag}>\n"));
    for item in items {
        if let Some(callout) = &item.callout {
            render_callout_item(item, callout, output);
            continue;
        }

        let content = render_spans(&item.text.spans);
        match item.task {
            Some(checked) => output.push_str(&format!(
                "<li class=\"task-item\"><input type=\"checkbox\" disabled{checked}>\
<span>{content}</span>{children}</li>\n",
                children = render_children(item),
                checked = if checked { " checked" } else { "" },
            )),
            None => output.push_str(&format!("<li>{content}{}</li>\n", render_children(item))),
        }
    }
    output.push_str(&format!("</{tag}>\n"));
}

fn render_children(item: &ListItem) -> String {
    match &item.children {
        Some(nested) => {
            let tag = if nested.ordered { "ol" } else { "ul" };
            let mut output = String::new();
            render_list(&nested.items, tag, &mut output);
            output
        }
        None => String::new(),
    }
}

/// A list item marked with a callout character. It stays an ordinary bullet:
/// the icon is inline and the accent only tints the background and border.
fn render_callout_item(item: &ListItem, callout: &ListCallout, output: &mut String) {
    output.push_str(&format!(
        "<li class=\"list-callout callout-{kind}\"{style}>\
<span class=\"list-callout-icon\">{icon}</span>{body}{children}</li>\n",
        kind = callout.kind.css(),
        style = callout
            .accent
            .as_ref()
            .map(|accent| format!(" style=\"--callout-accent: {accent}\""))
            .unwrap_or_default(),
        icon = callout.kind.icon(),
        body = render_spans(&item.text.spans),
        children = render_children(item),
    ));
}

fn render_table(table: &Table, output: &mut String) {
    output.push_str("<div class=\"table-wrap\">\n<table>\n<thead>\n<tr>\n");
    for (column, cell) in table.header.iter().enumerate() {
        output.push_str(&format!(
            "<th{}>{}</th>\n",
            align_attribute(table.alignment_at(column)),
            render_spans(&cell.spans)
        ));
    }
    output.push_str("</tr>\n</thead>\n<tbody>\n");

    for row in &table.rows {
        output.push_str("<tr>\n");
        for (column, cell) in row.iter().enumerate() {
            output.push_str(&format!(
                "<td{}>{}</td>\n",
                align_attribute(table.alignment_at(column)),
                render_spans(&cell.spans)
            ));
        }
        output.push_str("</tr>\n");
    }

    output.push_str("</tbody>\n</table>\n</div>\n");
}

fn render_quote(quote: &Quote, output: &mut String) {
    output.push_str("<blockquote>\n");
    for paragraph in &quote.paragraphs {
        output.push_str(&format!("<p>{}</p>\n", render_spans(&paragraph.spans)));
    }
    output.push_str("</blockquote>\n");
}

fn align_attribute(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::Default => "",
        Alignment::Left => " style=\"text-align: left\"",
        Alignment::Center => " style=\"text-align: center\"",
        Alignment::Right => " style=\"text-align: right\"",
    }
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;

    for character in text.chars() {
        if character.is_alphanumeric() {
            if pending_dash {
                slug.push('-');
                pending_dash = false;
            }
            slug.extend(character.to_lowercase());
        } else if !slug.is_empty() {
            pending_dash = true;
        }
    }

    slug
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block;
    use crate::types::{Callout, CalloutType, Text};

    /// The body of the stylesheet rule that starts with `selector`.
    fn stylesheet_rule(selector: &str) -> String {
        let start = STYLESHEET
            .find(selector)
            .unwrap_or_else(|| panic!("no rule for {selector}"));
        let body = &STYLESHEET[start + selector.len()..];
        let end = body.find('}').unwrap_or(body.len());
        body[..end].to_string()
    }

    #[test]
    fn renders_inline_styles() {
        let elements = block::parse("plain **bold** and `code` and ~~old~~ and ==new==\n");
        let html = Renderer::default().render(&elements);
        assert!(html.contains("<p>plain <strong>bold</strong> and <code>code</code> and <del>old</del> and <mark>new</mark></p>"));
    }

    #[test]
    fn escapes_text_content() {
        let elements = block::parse("a < b & c\n");
        let html = Renderer::default().render(&elements);
        assert!(html.contains("<p>a &lt; b &amp; c</p>"));
    }

    #[test]
    fn callouts_use_obsidian_structure() {
        let elements = block::parse("> [!danger] Stop\n> Careful.\n");
        let html = Renderer::default().render(&elements);
        assert!(html.contains("<aside class=\"callout callout-danger\">"));
        assert!(html.contains("<div class=\"callout-icon\"><svg viewBox=\"0 0 24 24\""));
        assert!(html.contains("<div class=\"callout-title\">Stop</div>"));
        assert!(html.contains("<p>Careful.</p>"));
    }

    #[test]
    fn every_callout_kind_has_an_icon() {
        let kinds = [
            CalloutType::Note,
            CalloutType::Info,
            CalloutType::Todo,
            CalloutType::Tip,
            CalloutType::Abstract,
            CalloutType::Question,
            CalloutType::Quote,
            CalloutType::Example,
            CalloutType::Success,
            CalloutType::Warning,
            CalloutType::Failure,
            CalloutType::Danger,
            CalloutType::Bug,
        ];
        for kind in kinds {
            let callout = Callout {
                kind,
                title: Text::default(),
                body: Vec::new(),
            };
            let mut output = String::new();
            render_callout(&callout, &mut output);
            assert!(
                output.contains("<svg viewBox=\"0 0 24 24\""),
                "{kind:?} has no icon"
            );
            assert!(
                output.contains("stroke=\"currentColor\""),
                "{kind:?} icon cannot inherit colour"
            );
        }
    }

    #[test]
    fn code_blocks_get_a_language_header_and_copy_button() {
        let elements = block::parse("```sh\nls -la\n```\n");
        let html = Renderer::default().render(&elements);
        assert!(html.contains("<span class=\"code-language\">sh</span>"));
        assert!(
            html.contains("<button class=\"copy-button\" type=\"button\" data-copy>Copy</button>")
        );
        assert!(html.contains("ls -la"));
    }

    #[test]
    fn escapes_code_block_content() {
        let elements = block::parse("```\n<div>raw</div>\n```\n");
        let html = Renderer::default().render(&elements);
        assert!(html.contains("&lt;div&gt;raw&lt;/div&gt;"));
    }

    #[test]
    fn assigns_unique_heading_ids() {
        let elements = block::parse("# Setup\n\n## Setup\n\n## Setup\n");
        let html = Renderer::default().render(&elements);
        assert!(html.contains("id=\"setup\""));
        assert!(html.contains("id=\"setup-1\""));
        assert!(html.contains("id=\"setup-2\""));
    }

    #[test]
    fn renders_an_image_tag() {
        let html = Renderer::default().render(&block::parse(
            "look ![a cat](https://example.com/cat.jpg)\n",
        ));
        assert!(
            html.contains(
                "<img src=\"https://example.com/cat.jpg\" alt=\"a cat\" loading=\"lazy\""
            ),
            "{html}"
        );
    }

    #[test]
    fn image_title_becomes_a_tooltip() {
        let html = Renderer::default().render(&block::parse("![cat](cat.jpg \"Sleeping cat\")\n"));
        assert!(html.contains("title=\"Sleeping cat\""), "{html}");
    }

    #[test]
    fn a_lone_image_becomes_a_figure() {
        let html = Renderer::default().render(&block::parse("![cat](cat.jpg \"Sleeping cat\")\n"));
        assert!(html.starts_with("<figure class=\"image\">"), "{html}");
        assert!(html.contains("</figure>"));
        assert!(!html.contains("<p>"), "no paragraph wrapper: {html}");
    }

    #[test]
    fn inline_images_stay_inside_the_paragraph() {
        let html = Renderer::default().render(&block::parse("before ![cat](cat.jpg) after\n"));
        assert!(html.starts_with("<p>before <img"), "{html}");
        assert!(!html.contains("<figure"), "{html}");
        assert!(!html.contains("figcaption"), "{html}");
    }

    #[test]
    fn the_alt_text_becomes_a_caption_under_the_image() {
        let html = Renderer::default().render(&block::parse("![A cat](cat.jpg)\n"));
        let image_end = html.find(">").expect("img tag");
        let caption = html.find("<figcaption>").expect("no caption");
        assert!(caption > image_end, "the caption must follow the image");
        assert!(html.contains("<figcaption>A cat</figcaption>"), "{html}");
    }

    #[test]
    fn an_empty_alt_text_draws_no_caption_area() {
        let html = Renderer::default().render(&block::parse("![](cat.jpg)\n"));
        assert!(html.contains("<img src=\"cat.jpg\" alt=\"\""), "{html}");
        assert!(!html.contains("figcaption"), "no caption wanted: {html}");
    }

    #[test]
    fn a_whitespace_alt_text_draws_no_caption_area() {
        let html = Renderer::default().render(&block::parse("![ ](cat.jpg)\n"));
        assert!(!html.contains("figcaption"), "no caption wanted: {html}");
    }

    #[test]
    fn the_caption_is_bordered_and_separated_from_the_image() {
        let caption = stylesheet_rule(".image figcaption {");
        assert!(caption.contains("border-top: 1px solid"), "{caption}");

        let figure = stylesheet_rule(".image {");
        assert!(figure.contains("border: 1px solid"), "{figure}");
        assert!(figure.contains("overflow: hidden"), "{figure}");
        assert!(
            !stylesheet_rule(".image img {").contains("border:"),
            "the frame belongs on the figure"
        );
    }

    #[test]
    fn a_broken_image_hides_its_caption() {
        assert!(stylesheet_rule(".image.is-broken figcaption {").contains("display: none"));
    }

    #[test]
    fn images_are_capped_at_the_column_width() {
        let html = render_document(&block::parse("![cat](cat.jpg)\n"), "t");
        assert!(
            html.contains("max-width: 100%"),
            "stylesheet was not inlined"
        );
        assert!(
            STYLESHEET.contains(".image img {"),
            "figure styles are missing"
        );
        assert!(
            STYLESHEET.contains(".image.is-broken::after"),
            "no broken state"
        );
        assert!(html.contains("document.images"), "no broken-image script");
    }

    #[test]
    fn a_raw_html_image_is_also_capped() {
        let html = render_document(
            &block::parse("<img src=\"https://example.com/wide.png\" alt=\"wide\">\n"),
            "t",
        );
        assert!(html.contains("<img src=\"https://example.com/wide.png\" alt=\"wide\">"));
        assert!(
            html.contains("img {\n    max-width: 100%;"),
            "global img rule"
        );
    }

    #[test]
    fn builds_a_table_of_contents_from_h2_and_h3() {
        let elements = block::parse("# Doc\n\n## One\n\n### Deep\n\n## Two\n\n#### Ignored\n");
        let mut renderer = Renderer::default();
        renderer.render(&elements);
        let toc = renderer.render_toc();
        assert!(toc.contains("<a href=\"#one\">One</a>"));
        assert!(toc.contains("<li class=\"depth-3\"><a href=\"#deep\">Deep</a></li>"));
        assert!(toc.contains("<a href=\"#two\">Two</a>"));
        assert!(!toc.contains("Ignored"));
        assert!(!toc.contains(">Doc</a>"));
        assert!(
            !toc.contains("toc-title"),
            "the sidebar header owns the label"
        );
    }

    #[test]
    fn the_sidebar_header_owns_the_contents_label() {
        let html = render_document(&block::parse("# Doc\n\n## One\n"), "t");
        assert_eq!(html.matches(">Contents<").count(), 1, "{html}");
    }

    #[test]
    fn omits_the_contents_navigation_when_there_is_nothing_to_list() {
        let html = render_document(&block::parse("just text\n"), "notes");
        assert!(!html.contains("<nav class=\"toc\""));
        assert!(html.contains("data-theme-toggle"), "toggle must survive");
        assert!(html.contains("<main class=\"page\">"));
    }

    #[test]
    fn has_no_header_bar() {
        let html = render_document(&block::parse("# Title\n\n## Section\n"), "t");
        assert!(!html.contains("<header"), "the top bar was removed");
        assert!(!html.contains("topbar"));
        assert!(!html.contains("class=\"brand\""));
    }

    #[test]
    fn theme_toggle_lives_in_the_sidebar() {
        let html = render_document(&block::parse("# T\n\n## One\n"), "t");
        let aside = html.find("<aside class=\"sidebar\">").unwrap();
        let toggle = html.find("data-theme-toggle").unwrap();
        let toc = html.find("<nav class=\"toc\"").unwrap();
        let main = html.find("<main class=\"page\">").unwrap();
        assert!(aside < toggle, "toggle must be inside the sidebar");
        assert!(toggle < toc, "toggle sits above the contents list");
        assert!(toc < main, "sidebar precedes the page");
    }

    #[test]
    fn raw_html_blocks_pass_through_unescaped() {
        let html = render_document(
            &block::parse("<details><summary>More</summary>\nHidden\n</details>\n"),
            "t",
        );
        assert!(html.contains("<details><summary>More</summary>\nHidden\n</details>"));
        assert!(!html.contains("&lt;details&gt;"));
    }

    #[test]
    fn raw_html_inline_passes_through_unescaped() {
        let html = render_document(&block::parse("H<sub>2</sub>O and <b>x</b>\n"), "t");
        assert!(html.contains("<p>H<sub>2</sub>O and <b>x</b></p>"));
    }

    #[test]
    fn autolinks_render_as_anchors() {
        let html = render_document(&block::parse("see <https://example.com>\n"), "t");
        assert!(html.contains("<a href=\"https://example.com\">https://example.com</a>"));
    }

    #[test]
    fn plain_less_than_signs_are_still_escaped() {
        let html = render_document(&block::parse("a < b and 3 <3\n"), "t");
        assert!(html.contains("<p>a &lt; b and 3 &lt;3</p>"));
    }

    #[test]
    fn titles_with_ampersands_are_escaped_not_double_escaped() {
        let html = render_document(&block::parse("# Tom & Jerry <ok>\n"), "t");
        assert!(html.contains("<title>Tom &amp; Jerry &lt;ok&gt;</title>"));
        assert!(!html.contains("&amp;amp;"));
    }

    #[test]
    fn heading_slugs_ignore_entities_and_markup() {
        let elements = block::parse("## Tom & Jerry <b>x</b>\n");
        let html = Renderer::default().render(&elements);
        assert!(html.contains("id=\"tom-jerry-b-x-b\""), "{html}");
    }

    #[test]
    fn document_ships_theme_toggle_and_stylesheet() {
        let html = render_document(&block::parse("# T\n"), "t");
        assert!(html.contains("data-theme-toggle"));
        assert!(html.contains("prefers-color-scheme: dark"));
        assert!(html.contains(":root[data-theme=\"dark\"]"));
        assert!(html.contains("localStorage.getItem('md2html-theme')"));
        assert!(html.contains(".callout {"), "stylesheet was not inlined");
        assert!(!STYLESHEET.contains("topbar"), "topbar css was removed");
    }

    #[test]
    fn document_uses_first_h1_as_title() {
        let elements = block::parse("# My Title\n\nbody\n");
        let html = render_document(&elements, "ignored");
        assert!(html.contains("<title>My Title</title>"));
        assert!(!html.contains("ignored"));
    }

    #[test]
    fn document_falls_back_to_filename() {
        let html = render_document(&[], "notes.md");
        assert!(html.contains("<title>notes.md</title>"));
    }

    #[test]
    fn stylesheet_defines_both_themes_and_a_callout_palette() {
        assert!(STYLESHEET.contains(":root[data-theme=\"dark\"]"));
        assert!(STYLESHEET.contains("prefers-color-scheme: dark"));
        for kind in [
            CalloutType::Note,
            CalloutType::Info,
            CalloutType::Todo,
            CalloutType::Tip,
            CalloutType::Abstract,
            CalloutType::Question,
            CalloutType::Quote,
            CalloutType::Example,
            CalloutType::Success,
            CalloutType::Warning,
            CalloutType::Failure,
            CalloutType::Danger,
            CalloutType::Bug,
        ] {
            assert!(
                STYLESHEET.contains(&format!(".callout-{}", kind.css())),
                "no colour defined for {}",
                kind.css()
            );
        }
    }

    #[test]
    fn theme_script_runs_before_the_stylesheet() {
        let html = render_document(&block::parse("# T\n"), "t");
        let script = html.find("document.documentElement.dataset.theme").unwrap();
        let style = html.find("<style>").unwrap();
        assert!(script < style, "theme script must run before paint");
    }

    #[test]
    fn renders_a_list_callout_with_an_inline_icon() {
        let html = Renderer::default().render(&block::parse("- ! Important item\n"));
        assert!(
            html.contains("<li class=\"list-callout callout-note\">"),
            "{html}"
        );
        assert!(
            html.contains("<span class=\"list-callout-icon\">"),
            "{html}"
        );
        assert!(html.contains("</span>Important item</li>"), "{html}");
        assert!(html.contains("<svg"), "the icon svg is missing: {html}");
    }

    #[test]
    fn a_list_callout_stays_a_plain_bullet() {
        let html = Renderer::default().render(&block::parse("- ! Important item\n"));
        assert!(
            !html.contains("list-callout-body"),
            "no wrapper div: {html}"
        );

        let rule = stylesheet_rule("li.list-callout {");
        assert!(!rule.contains("list-style"), "the bullet must stay: {rule}");
        assert!(
            !rule.contains("display: flex"),
            "not a flex callout: {rule}"
        );
        assert!(!rule.contains("border-left"), "not an accent bar: {rule}");
    }

    #[test]
    fn the_callout_marker_is_not_left_in_the_text() {
        let html = Renderer::default().render(&block::parse("- ! Important item\n"));
        assert!(!html.contains("! Important"), "{html}");
    }

    #[test]
    fn each_list_callout_uses_its_own_palette_class() {
        let html = Renderer::default().render(&block::parse("- ! a\n- ? b\n- * c\n- + d\n"));
        assert!(html.contains("list-callout callout-note"), "{html}");
        assert!(html.contains("list-callout callout-question"), "{html}");
        assert!(html.contains("list-callout callout-tip"), "{html}");
        assert!(html.contains("list-callout callout-success"), "{html}");
    }

    #[test]
    fn a_plain_item_next_to_a_callout_stays_plain() {
        let html = Renderer::default().render(&block::parse("- plain\n- ! styled\n"));
        assert!(html.contains("<li>plain</li>"), "{html}");
    }

    #[test]
    fn renders_nested_lists() {
        let html = Renderer::default().render(&block::parse("- one\n  - one a\n- two\n"));
        assert_eq!(
            html,
            "<ul>\n<li>one<ul>\n<li>one a</li>\n</ul>\n</li>\n<li>two</li>\n</ul>\n"
        );
    }

    #[test]
    fn a_nested_list_callout_keeps_its_icon() {
        let html = Renderer::default().render(&block::parse("- parent\n  - ! nested\n"));
        assert!(html.contains("list-callout callout-note"), "{html}");
        assert!(html.contains("nested"), "{html}");
    }

    #[test]
    fn task_items_can_nest() {
        let html = Renderer::default().render(&block::parse("- parent\n  - [x] done\n"));
        assert!(html.contains("task-item"), "{html}");
        assert!(html.contains("checked"), "{html}");
    }

    #[test]
    fn list_callouts_are_styled_with_an_accent_and_border() {
        let rule = stylesheet_rule("li.list-callout {");
        assert!(!rule.is_empty(), "no list callout rule");
        assert!(rule.contains("border: 1px solid"), "no border: {rule}");
        assert!(
            rule.contains("border-radius: var(--radius);"),
            "no radius: {rule}"
        );
        assert!(
            rule.contains("color-mix(in srgb, var(--callout-accent) 8%, var(--bg))"),
            "no tinted background: {rule}"
        );
        assert!(
            STYLESHEET.contains(":where(li.list-callout)"),
            "no zero-specificity default"
        );
        assert!(stylesheet_rule(".list-callout-icon {").contains("display: inline-block"));
    }

    #[test]
    fn jetbrains_mono_is_the_only_stack() {
        let root = stylesheet_rule(":root {");
        assert!(
            root.contains("--font: \"JetBrains Mono\""),
            "the stack should start with JetBrains Mono: {root}"
        );
        assert!(root.contains("monospace;"), "no generic fallback: {root}");
        assert!(
            !STYLESHEET.contains("fonts.googleapis"),
            "no webfont request, the file stays self-contained"
        );
        assert_eq!(STYLESHEET.matches("--font:").count(), 1, "declared once");
        assert!(
            !STYLESHEET.contains("--sans") && !STYLESHEET.contains("--mono:"),
            "there should be a single font stack"
        );
    }

    #[test]
    fn every_text_element_uses_the_one_stack() {
        assert!(stylesheet_rule("body {").contains("font-family: var(--font)"));
        assert_eq!(
            STYLESHEET.matches("font-family: var(--font)").count(),
            4,
            "body, code, the language label and the copy button"
        );
        assert!(
            !STYLESHEET.contains("font-family: -apple-system"),
            "no leftover UI stack"
        );
        // Without this the theme toggle would render in the browser default font.
        let controls = stylesheet_rule("button,\ninput,\nselect,\ntextarea {");
        assert!(controls.contains("font: inherit"), "{controls}");
    }

    #[test]
    fn code_inherits_rather_than_overriding() {
        // `pre` never sets a family of its own: it inherits from the inner `code`.
        assert!(stylesheet_rule("code {").contains("font-family: var(--font)"));
        assert!(stylesheet_rule(".code-language {").contains("font-family: var(--font)"));
        assert!(
            !stylesheet_rule("pre {").contains("font-family"),
            "pre should inherit, not override"
        );
    }

    #[test]
    fn the_light_background_is_warm_paper() {
        let light = stylesheet_rule(":root {");
        assert!(light.contains("--bg: #f5f5f0;"), "{light}");
        assert!(
            light.contains("color-scheme: light"),
            "the default must still declare light"
        );
    }

    #[test]
    fn the_dark_background_is_near_black() {
        for selector in [
            ":root:not([data-theme=\"light\"]) {",
            ":root[data-theme=\"dark\"] {",
        ] {
            let dark = stylesheet_rule(selector);
            assert!(dark.contains("--bg: #0a0a0a;"), "{selector} -> {dark}");
            assert!(dark.contains("color-scheme: dark"), "{selector}");
        }
    }

    /// The dark palette is written out twice: once for `prefers-color-scheme` so
    /// no-JS visitors still get dark mode, once for the explicit toggle. Drift
    /// between them would silently ship two different dark themes.
    #[test]
    fn the_dark_palette_is_defined_once() {
        let names = [
            "--bg-bar",
            "--bg-subtle",
            "--bg-inset",
            "--bg-code",
            "--fg",
            "--fg-muted",
            "--fg-faint",
            "--border",
            "--border-strong",
            "--accent",
            "--accent-soft",
            "--shadow",
        ];

        let media = stylesheet_rule(":root:not([data-theme=\"light\"]) {");
        let toggle = stylesheet_rule(":root[data-theme=\"dark\"] {");
        for name in names {
            let value = |block: &str| {
                block
                    .lines()
                    .find(|line| line.trim_start().starts_with(name))
                    .unwrap_or_else(|| panic!("{name} missing"))
                    .split_once(':')
                    .map(|(_, rest)| rest.trim().trim_end_matches(';').to_string())
                    .unwrap()
            };
            assert_eq!(
                value(&media),
                value(&toggle),
                "{name} differs between the two dark blocks"
            );
        }
    }

    /// Real WCAG contrast, so the check works in both directions: light-theme
    /// text is darker than its page, dark-theme text is lighter.
    fn contrast(a: &str, b: &str) -> f64 {
        fn channel(value: u32) -> f64 {
            let c = f64::from(value) / 255.0;
            if c <= 0.039_28 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        let luminance = |hex: &str| {
            let raw = hex.trim_start_matches('#');
            let part = |at: usize| u32::from_str_radix(&raw[at..at + 2], 16).unwrap();
            0.2126 * channel(part(0)) + 0.7152 * channel(part(2)) + 0.0722 * channel(part(4))
        };

        let (hi, lo) = (luminance(a), luminance(b));
        let (hi, lo) = if hi > lo { (hi, lo) } else { (lo, hi) };
        (hi + 0.05) / (lo + 0.05)
    }

    fn palette(selector: &str) -> Vec<(String, String)> {
        stylesheet_rule(selector)
            .lines()
            .filter_map(|line| {
                let (name, value) = line.split_once(':')?;
                let name = name.trim();
                if !name.starts_with("--") {
                    return None;
                }
                Some((
                    name.to_string(),
                    value.trim().trim_end_matches(';').to_string(),
                ))
            })
            .collect()
    }

    #[test]
    fn text_is_readable_in_both_themes() {
        for (theme, selector) in [
            ("light", ":root {"),
            ("dark", ":root[data-theme=\"dark\"] {"),
        ] {
            let vars: std::collections::HashMap<_, _> = palette(selector).into_iter().collect();
            let get = |name: &str| {
                vars.get(name)
                    .unwrap_or_else(|| panic!("{theme} is missing {name}"))
                    .clone()
            };
            let bg = get("--bg");
            let ratio = |fg: &str| contrast(&get(fg), &bg);

            // Body text and the muted tone carry real prose, so AA.
            assert!(
                ratio("--fg") >= 4.5,
                "{theme}: --fg is only {:.2}:1",
                ratio("--fg")
            );
            assert!(
                ratio("--fg-muted") >= 4.5,
                "{theme}: --fg-muted is only {:.2}:1",
                ratio("--fg-muted")
            );
            // Faint is only the broken-image notice, so AA Large is enough.
            assert!(
                ratio("--fg-faint") >= 3.0,
                "{theme}: --fg-faint is only {:.2}:1",
                ratio("--fg-faint")
            );
            // Prose also sits on the subtle and inset surfaces.
            for surface in ["--bg-subtle", "--bg-inset", "--bg-code"] {
                let on = contrast(&get("--fg"), &get(surface));
                assert!(on >= 4.5, "{theme}: --fg on {surface} is only {on:.2}:1");
            }
        }
    }

    /// The table grid is the whole point of the bordered tables, so the strong
    /// border has to actually separate cells from the page.
    #[test]
    fn the_table_grid_stays_visible() {
        for (theme, selector) in [
            ("light", ":root {"),
            ("dark", ":root[data-theme=\"dark\"] {"),
        ] {
            let vars: std::collections::HashMap<_, _> = palette(selector).into_iter().collect();
            let get = |name: &str| vars.get(name).unwrap().clone();
            let on_page = contrast(&get("--border-strong"), &get("--bg"));
            assert!(
                on_page >= 1.5,
                "{theme}: --border-strong is only {on_page:.2}:1 against the page"
            );
            let on_inset = contrast(&get("--border-strong"), &get("--bg-inset"));
            assert!(
                on_inset >= 1.3,
                "{theme}: header cells would swallow their own border at {on_inset:.2}:1"
            );
        }
    }

    #[test]
    fn corners_are_only_slightly_rounded() {
        assert!(
            STYLESHEET.contains("--radius: 3px;"),
            "the radius scale should stay small"
        );
    }

    #[test]
    fn tables_have_borders() {
        let cells = stylesheet_rule("th,\ntd {");
        assert!(
            cells.contains("border: 1px solid"),
            "no cell border: {cells}"
        );
        assert!(
            !stylesheet_rule(".table-wrap {").contains("background"),
            "no fill"
        );
    }

    #[test]
    fn renders_task_list_items() {
        let elements = block::parse("- [x] done\n- [ ] todo\n");
        let html = Renderer::default().render(&elements);
        assert!(html.contains("<li class=\"task-item\"><input type=\"checkbox\" disabled checked>"));
        assert!(html.contains("<input type=\"checkbox\" disabled>"));
    }

    #[test]
    fn renders_table_alignment() {
        let source = "| a | b |\n| :-- | --: |\n| 1 | 2 |\n";
        let html = Renderer::default().render(&block::parse(source));
        assert!(html.contains("<th style=\"text-align: left\">a</th>"));
        assert!(html.contains("<td style=\"text-align: right\">2</td>"));
    }

    #[test]
    fn renders_divider_and_quote() {
        let html = Renderer::default().render(&block::parse("---\n\n> quoted\n"));
        assert!(html.contains("<hr class=\"divider\">"));
        assert!(html.contains("<blockquote>\n<p>quoted</p>\n</blockquote>"));
    }

    #[test]
    fn slugify_trims_punctuation() {
        assert_eq!(slugify("Hello, World! 42"), "hello-world-42");
        assert_eq!(slugify("---"), "");
    }

    #[test]
    fn text_plain_concatenates_spans() {
        let text = Text {
            spans: vec![
                TextSpan {
                    text: "a".to_string(),
                    styles: vec![TextType::Bold],
                },
                TextSpan {
                    text: "b".to_string(),
                    styles: vec![],
                },
            ],
        };
        assert_eq!(text.plain(), "ab");
    }
}
