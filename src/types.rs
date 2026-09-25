pub enum Element {
    Text(Text),
    Heading(Heading),
    Divider(Divider),
    Table(Table),
    Callout(Callout),
    OrderedList(OrderedList),
    UnorderedList(UnorderedList),
}
pub struct Text {
    text: String,
    text_type: Vec<TextSpan>,
}

struct TextSpan {
    text: String,
    styles: Vec<TextType>,
}
pub enum TextType {
    Normal,
    Bold,
    Italic,
    Highlight,
    Underline,
    InlineCode,
    Strikethrough,
}

pub struct Heading {
    text: String,
    level: HeadingLevel,
}

pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

pub struct Divider {
    text: String,
}

pub struct Table {
    column: TableColumn,
    rows: Vec<TableRows>,
}

pub struct TableColumn {
    text: Vec<String>,
}

pub struct TableRows {
    rows: Vec<String>,
}

pub struct Callout {
    title: String,
    body: String,
    callout_type: CalloutType,
}

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

pub struct OrderedList {
    list: Vec<String>,
}

pub struct UnorderedList {
    list: Vec<String>,
}
