#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Heading(Heading),
    Text(Text),
    OrderedList(OrderedList),
    UnorderedList(UnorderedList),
    CodeBlock(CodeBlock),
    Table(Table),
    Callout(Callout),
    Quote(Quote),
    Divider,
    Html(String),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Text {
    pub spans: Vec<TextSpan>,
}

impl Text {
    pub fn plain(&self) -> String {
        let joined: String = self.spans.iter().map(|span| span.text.as_str()).collect();
        crate::helper::unescape_html(&joined)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSpan {
    pub text: String,
    pub styles: Vec<TextType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextType {
    Bold,
    Italic,
    Highlight,
    InlineCode,
    Strikethrough,
    Link(String),
    Image { src: String, title: Option<String> },
    RawHtml,
}

impl TextType {
    pub fn image(&self) -> Option<(&str, Option<&str>)> {
        match self {
            TextType::Image { src, title } => Some((src, title.as_deref())),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl HeadingLevel {
    pub fn from_hashes(count: usize) -> Option<Self> {
        match count {
            1 => Some(HeadingLevel::H1),
            2 => Some(HeadingLevel::H2),
            3 => Some(HeadingLevel::H3),
            4 => Some(HeadingLevel::H4),
            5 => Some(HeadingLevel::H5),
            6 => Some(HeadingLevel::H6),
            _ => None,
        }
    }

    pub fn tag(&self) -> &'static str {
        match self {
            HeadingLevel::H1 => "h1",
            HeadingLevel::H2 => "h2",
            HeadingLevel::H3 => "h3",
            HeadingLevel::H4 => "h4",
            HeadingLevel::H5 => "h5",
            HeadingLevel::H6 => "h6",
        }
    }

    pub fn depth(&self) -> usize {
        match self {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub level: HeadingLevel,
    pub text: Text,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub text: Text,
    pub task: Option<bool>,
    pub callout: Option<ListCallout>,
    pub children: Option<NestedList>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListCallout {
    pub kind: CalloutType,
    /// Optional accent colour override, e.g. `#e91e63` from the config file.
    pub accent: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NestedList {
    pub ordered: bool,
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderedList {
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnorderedList {
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Default,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub header: Vec<Text>,
    pub alignments: Vec<Alignment>,
    pub rows: Vec<Vec<Text>>,
}

impl Table {
    pub fn alignment_at(&self, column: usize) -> Alignment {
        self.alignments
            .get(column)
            .copied()
            .unwrap_or(Alignment::Default)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Callout {
    pub kind: CalloutType,
    pub title: Text,
    pub body: Vec<Text>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutType {
    Note,
    Info,
    Todo,
    Tip,
    Abstract,
    Question,
    Quote,
    Example,
    Success,
    Warning,
    Failure,
    Danger,
    Bug,
}

impl CalloutType {
    pub fn from_marker(marker: &str) -> Option<Self> {
        match marker.trim().to_ascii_lowercase().as_str() {
            "note" | "notes" => Some(CalloutType::Note),
            "info" | "information" => Some(CalloutType::Info),
            "todo" => Some(CalloutType::Todo),
            "tip" | "hint" | "important" => Some(CalloutType::Tip),
            "abstract" | "summary" | "tldr" => Some(CalloutType::Abstract),
            "question" | "help" | "faq" => Some(CalloutType::Question),
            "quote" | "cite" => Some(CalloutType::Quote),
            "example" => Some(CalloutType::Example),
            "success" | "check" | "done" => Some(CalloutType::Success),
            "warning" | "warn" | "caution" | "attention" => Some(CalloutType::Warning),
            "failure" | "fail" | "missing" => Some(CalloutType::Failure),
            "danger" | "error" => Some(CalloutType::Danger),
            "bug" => Some(CalloutType::Bug),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CalloutType::Note => "Note",
            CalloutType::Info => "Info",
            CalloutType::Todo => "Todo",
            CalloutType::Tip => "Tip",
            CalloutType::Abstract => "Abstract",
            CalloutType::Question => "Question",
            CalloutType::Quote => "Quote",
            CalloutType::Example => "Example",
            CalloutType::Success => "Success",
            CalloutType::Warning => "Warning",
            CalloutType::Failure => "Failure",
            CalloutType::Danger => "Danger",
            CalloutType::Bug => "Bug",
        }
    }

    pub fn css(&self) -> &'static str {
        match self {
            CalloutType::Note => "note",
            CalloutType::Info => "info",
            CalloutType::Todo => "todo",
            CalloutType::Tip => "tip",
            CalloutType::Abstract => "abstract",
            CalloutType::Question => "question",
            CalloutType::Quote => "quote",
            CalloutType::Example => "example",
            CalloutType::Success => "success",
            CalloutType::Warning => "warning",
            CalloutType::Failure => "failure",
            CalloutType::Danger => "danger",
            CalloutType::Bug => "bug",
        }
    }

    pub fn icon(&self) -> String {
        svg(self.path())
    }

    fn path(&self) -> &'static str {
        match self {
            CalloutType::Note => "<path d=\"M4 20l1-4 9.5-9.5a2.12 2.12 0 0 1 3 3L8 19l-4 1z\"/><path d=\"M13.5 6.5l4 4\"/>",
            CalloutType::Info => "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M12 11.2v5.3\"/><path d=\"M12 7.7h.01\"/>",
            CalloutType::Todo => "<rect x=\"3.5\" y=\"3.5\" width=\"17\" height=\"17\" rx=\"4.5\"/><path d=\"M8 12.3l2.7 2.7 5.3-5.6\"/>",
            CalloutType::Tip => "<path d=\"M9.2 17.5h5.6\"/><path d=\"M10 20.5h4\"/><path d=\"M12 3.2a5.8 5.8 0 0 0-3.4 10.5c.7.5 1.1 1.3 1.2 2h4.4c.1-.8.5-1.5 1.2-2A5.8 5.8 0 0 0 12 3.2z\"/>",
            CalloutType::Abstract => "<rect x=\"5\" y=\"4\" width=\"14\" height=\"17\" rx=\"2.5\"/><path d=\"M9.2 4.5h5.6v2.6H9.2z\"/><path d=\"M8.6 11.4h6.8\"/><path d=\"M8.6 15.4h4.4\"/>",
            CalloutType::Question => "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M9.4 9.6a2.7 2.7 0 0 1 5.2.9c0 1.8-2.6 2.2-2.6 3.9\"/><path d=\"M12 17.3h.01\"/>",
            CalloutType::Quote => "<path d=\"M10 8H7.5A2.5 2.5 0 0 0 5 10.5v1A2.5 2.5 0 0 0 7.5 14H9v.5a3 3 0 0 1-3 3\"/><path d=\"M19 8h-2.5a2.5 2.5 0 0 0-2.5 2.5v1a2.5 2.5 0 0 0 2.5 2.5H18v.5a3 3 0 0 1-3 3\"/>",
            CalloutType::Example => "<path d=\"M9.5 6.5H20\"/><path d=\"M9.5 12H20\"/><path d=\"M9.5 17.5H20\"/><path d=\"M5 6.5h.01\"/><path d=\"M5 12h.01\"/><path d=\"M5 17.5h.01\"/>",
            CalloutType::Success => "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M8 12.3l2.7 2.7 5.3-5.6\"/>",
            CalloutType::Warning => "<path d=\"M12 3.6 2.9 19.8h18.2L12 3.6z\"/><path d=\"M12 9.8v4.4\"/><path d=\"M12 17.4h.01\"/>",
            CalloutType::Failure => "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M9.2 9.2l5.6 5.6\"/><path d=\"M14.8 9.2l-5.6 5.6\"/>",
            CalloutType::Danger => "<path d=\"M13.2 2.5 4.8 13.4h5.6L9.9 21.5l8.3-11.2h-5.6l.6-7.8z\"/>",
            CalloutType::Bug => "<rect x=\"8.2\" y=\"7.5\" width=\"7.6\" height=\"11\" rx=\"3.8\"/><path d=\"M9.5 7.5a2.5 2.5 0 0 1 5 0\"/><path d=\"M8.4 11H4.6\"/><path d=\"M8.4 15H4.6\"/><path d=\"M15.6 11h3.8\"/><path d=\"M15.6 15h3.8\"/>",
        }
    }
}

fn svg(body: &str) -> String {
    format!(
        "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" \
         stroke-width=\"1.7\" stroke-linecap=\"round\" stroke-linejoin=\"round\" \
         aria-hidden=\"true\" focusable=\"false\">{body}</svg>"
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    pub paragraphs: Vec<Text>,
}
