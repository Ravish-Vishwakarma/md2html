use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::types::CalloutType;

/// Name of the optional configuration file, looked up next to the Markdown
/// source first and then in the current directory.
pub const CONFIG_FILENAME: &str = "md2html.conf";

/// A single `character -> callout type` rule for list callouts.
#[derive(Debug, Clone, PartialEq)]
pub struct MarkerRule {
    pub key: char,
    pub kind: CalloutType,
    pub accent: Option<String>,
}

impl MarkerRule {
    fn new(key: char, kind: CalloutType) -> Self {
        MarkerRule {
            key,
            kind,
            accent: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    markers: HashMap<char, MarkerRule>,
}

impl Default for Config {
    fn default() -> Self {
        let rules = [
            MarkerRule::new('!', CalloutType::Note),
            MarkerRule::new('?', CalloutType::Question),
            MarkerRule::new('*', CalloutType::Tip),
            MarkerRule::new('+', CalloutType::Success),
            MarkerRule::new('~', CalloutType::Warning),
        ];

        Config {
            markers: rules.into_iter().map(|rule| (rule.key, rule)).collect(),
        }
    }
}

impl Config {
    /// Reads rules from `content`, keeping any default rule whose character is
    /// not mentioned. Unknown callout names are ignored so one typo cannot
    /// silently disable the whole file.
    pub fn with_overrides(content: &str) -> Self {
        let mut config = Config::default();
        let content = content.strip_prefix('\u{feff}').unwrap_or(content);

        for line in content.lines() {
            let mut fields = line.split_whitespace();
            let Some(key) = fields.next().and_then(|word| word.chars().next()) else {
                continue;
            };
            let Some(name) = fields.next() else {
                continue;
            };
            let Some(kind) = CalloutType::from_marker(name) else {
                continue;
            };

            let mut rule = MarkerRule::new(key, kind);
            rule.accent = fields.next().map(str::to_string);
            config.markers.insert(key, rule);
        }

        config
    }

    pub fn rule(&self, key: char) -> Option<&MarkerRule> {
        self.markers.get(&key)
    }

    /// Loads the config file for `source`, preferring a copy that sits beside
    /// the Markdown file over one in the current directory. A missing file is
    /// not an error; the built-in defaults are used instead.
    pub fn load_for(source: &str) -> io::Result<Self> {
        let beside = Path::new(source)
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(|parent| parent.join(CONFIG_FILENAME))
            .unwrap_or_else(|| PathBuf::from(CONFIG_FILENAME));

        if beside.is_file() {
            return Self::load(&beside);
        }

        let current = PathBuf::from(CONFIG_FILENAME);
        if current.is_file() {
            return Self::load(&current);
        }

        Ok(Config::default())
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        Ok(Config::with_overrides(&std::fs::read_to_string(path)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_cover_the_documented_characters() {
        let config = Config::default();
        assert_eq!(
            config.rule('!').map(|rule| rule.kind),
            Some(CalloutType::Note)
        );
        assert_eq!(
            config.rule('?').map(|rule| rule.kind),
            Some(CalloutType::Question)
        );
        assert_eq!(
            config.rule('*').map(|rule| rule.kind),
            Some(CalloutType::Tip)
        );
        assert_eq!(
            config.rule('+').map(|rule| rule.kind),
            Some(CalloutType::Success)
        );
        assert!(config.rule('#').is_none());
    }

    #[test]
    fn overrides_a_single_character() {
        let config = Config::with_overrides("? danger\n");
        assert_eq!(
            config.rule('?').map(|rule| rule.kind),
            Some(CalloutType::Danger)
        );
        assert_eq!(
            config.rule('!').map(|rule| rule.kind),
            Some(CalloutType::Note),
            "untouched defaults survive"
        );
    }

    #[test]
    fn accepts_an_accent_colour() {
        let config = Config::with_overrides("! note #ff9800\n");
        let rule = config.rule('!').unwrap();
        assert_eq!(rule.accent.as_deref(), Some("#ff9800"));
    }

    #[test]
    fn accepts_any_of_the_callout_aliases() {
        let config = Config::with_overrides("* hint\n? faq\n+ done\n~ caution\n! tldr\n");
        assert_eq!(config.rule('*').map(|r| r.kind), Some(CalloutType::Tip));
        assert_eq!(
            config.rule('?').map(|r| r.kind),
            Some(CalloutType::Question)
        );
        assert_eq!(config.rule('+').map(|r| r.kind), Some(CalloutType::Success));
        assert_eq!(config.rule('~').map(|r| r.kind), Some(CalloutType::Warning));
        assert_eq!(
            config.rule('!').map(|r| r.kind),
            Some(CalloutType::Abstract)
        );
    }

    #[test]
    fn ignores_comments_blank_lines_and_typos() {
        let config =
            Config::with_overrides("# a comment\n\n   \n! notarealtype\n? question\nnonsense\n");
        assert_eq!(
            config.rule('?').map(|rule| rule.kind),
            Some(CalloutType::Question)
        );
        assert_eq!(
            config.rule('!').map(|rule| rule.kind),
            Some(CalloutType::Note),
            "an unknown name must not drop the default"
        );
        assert!(config.rule('n').is_none(), "a single word is not a rule");
    }

    #[test]
    fn any_character_can_be_mapped() {
        let config = Config::with_overrides("% note\n$ danger\n");
        assert_eq!(
            config.rule('%').map(|rule| rule.kind),
            Some(CalloutType::Note)
        );
        assert_eq!(
            config.rule('$').map(|rule| rule.kind),
            Some(CalloutType::Danger)
        );
    }

    #[test]
    fn a_custom_key_is_allowed() {
        let config = Config::with_overrides("i info\n");
        assert_eq!(
            config.rule('i').map(|rule| rule.kind),
            Some(CalloutType::Info)
        );
    }

    #[test]
    fn a_missing_file_reports_an_error() {
        assert!(Config::load(Path::new("definitely-not-here-4b21.conf")).is_err());
    }

    #[test]
    fn an_empty_config_is_the_default_config() {
        assert_eq!(Config::with_overrides(""), Config::default());
    }

    #[test]
    fn load_for_reads_the_file_beside_the_source() {
        let directory = std::env::temp_dir().join("md2html-config-8f21");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join(CONFIG_FILENAME), "! success\n? failure\n").unwrap();
        let source = directory.join("doc.md");
        std::fs::write(&source, "- ! a\n- ? b\n").unwrap();

        let config = Config::load_for(source.to_str().unwrap()).unwrap();
        assert_eq!(
            config.rule('!').map(|rule| rule.kind),
            Some(CalloutType::Success)
        );
        assert_eq!(
            config.rule('?').map(|rule| rule.kind),
            Some(CalloutType::Failure)
        );

        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn load_for_tolerates_a_byte_order_mark() {
        let directory = std::env::temp_dir().join("md2html-config-bom-3d67");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join(CONFIG_FILENAME), "\u{feff}! success\n").unwrap();
        let source = directory.join("doc.md");
        std::fs::write(&source, "").unwrap();

        let config = Config::load_for(source.to_str().unwrap()).unwrap();
        assert_eq!(
            config.rule('!').map(|rule| rule.kind),
            Some(CalloutType::Success)
        );

        std::fs::remove_dir_all(&directory).unwrap();
    }
}
