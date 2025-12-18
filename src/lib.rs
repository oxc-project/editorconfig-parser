use std::path::{Path, PathBuf};

use globset::{Glob, GlobMatcher};

#[derive(Debug, Default, Clone)]
pub struct EditorConfig {
    /// Set to true to tell the core not to check any higher directory for EditorConfig settings for on the current filename.
    root: bool,

    sections: Vec<EditorConfigSection>,

    /// The base directory for resolving absolute paths.
    cwd: Option<PathBuf>,
}

impl EditorConfig {
    pub fn root(&self) -> bool {
        self.root
    }

    pub fn sections(&self) -> &[EditorConfigSection] {
        &self.sections
    }

    pub fn cwd(&self) -> Option<&Path> {
        self.cwd.as_deref()
    }

    /// Sets the current working directory for resolving absolute paths.
    pub fn with_cwd(mut self, cwd: impl AsRef<Path>) -> Self {
        self.cwd = Some(cwd.as_ref().to_path_buf());
        self
    }
}

/// <https://spec.editorconfig.org/index.html>
#[derive(Debug, Default, Clone)]
pub struct EditorConfigSection {
    /// Section Name: the string between the beginning `[` and the ending `]`.
    pub name: String,

    pub matcher: Option<GlobMatcher>,

    pub properties: EditorConfigProperties,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum EditorConfigProperty<T> {
    #[default]
    None,
    Unset,
    Value(T),
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct EditorConfigProperties {
    /// Set to tab or space to use tabs or spaces for indentation, respectively.
    /// Option tab implies that an indentation is to be filled by as many hard tabs as possible, with the rest of the indentation filled by spaces.
    /// A non-normative explanation can be found in the indentation_ section.
    /// The values are case-insensitive.
    pub indent_style: EditorConfigProperty<IndentStyle>,

    /// Set to a whole number defining the number of columns used for each indentation level and the width of soft tabs (when supported).
    /// If this equals tab, the indent_size shall be set to the tab size, which should be tab_width (if specified); else, the tab size set by the editor.
    /// The values are case-insensitive.
    pub indent_size: EditorConfigProperty<usize>,

    /// Set to a whole number defining the number of columns used to represent a tab character.
    /// This defaults to the value of indent_size and should not usually need to be specified.
    pub tab_width: EditorConfigProperty<usize>,

    /// Set to lf, cr, or crlf to control how line breaks are represented.
    /// The values are case-insensitive.
    pub end_of_line: EditorConfigProperty<EndOfLine>,

    /// Set to latin1, utf-8, utf-8-bom, utf-16be or utf-16le to control the character set.
    /// Use of utf-8-bom is discouraged.
    /// The values are case-insensitive.
    pub charset: EditorConfigProperty<Charset>,

    /// Set to true to remove all whitespace characters preceding newline characters in the file and false to ensure it doesn’t.
    pub trim_trailing_whitespace: EditorConfigProperty<bool>,

    /// Set to true ensure file ends with a newline when saving and false to ensure it doesn’t.
    /// Editors must not insert newlines in empty files when saving those files, even if insert_final_newline = true.
    pub insert_final_newline: EditorConfigProperty<bool>,

    /// Prettier print width.
    /// Not part of spec <https://github.com/editorconfig/editorconfig-vscode/issues/53#issuecomment-462432616>
    /// But documented in <https://prettier.io/docs/next/configuration#editorconfig>
    pub max_line_length: EditorConfigProperty<MaxLineLength>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MaxLineLength {
    /// A numeric line length limit
    Number(usize),
    /// Line length limit is disabled
    Off,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum IndentStyle {
    Tab,
    Space,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EndOfLine {
    Lf,
    Cr,
    Crlf,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Charset {
    Latin1,
    Utf8,
    Utf8bom,
    Utf16be,
    Utf16le,
}

impl EditorConfig {
    /// <https://spec.editorconfig.org/index.html#id6>
    pub fn parse(source_text: &str) -> Self {
        // EditorConfig files are in an INI-like file format.
        // To read an EditorConfig file, take one line at a time, from beginning to end.
        // For each line:
        // 1. Remove all leading and trailing whitespace.
        // 2. Process the remaining text as specified for its type below.
        let mut root = false;
        let mut sections = vec![];
        let mut preamble = true;
        for line in source_text.lines() {
            let line = line.trim();
            // Blank: Contains nothing. Blank lines are ignored.
            if line.is_empty() {
                continue;
            }
            // Comment: starts with a ; or a #. Comment lines are ignored.
            if line.starts_with([';', '#']) {
                continue;
            }
            // Parse `root`. Must be specified in the preamble. The value is case-insensitive.
            if preamble
                && !line.starts_with('[')
                && let Some((key, value)) = line.split_once('=')
                && key.trim_end() == "root"
                && value.trim_start().eq_ignore_ascii_case("true")
            {
                root = true;
            }
            // Section Header: starts with a [ and ends with a ]. These lines define globs;
            if let Some(line) = line.strip_prefix('[') {
                preamble = false;
                if let Some(line) = line.strip_suffix(']') {
                    let name = line.to_string();
                    let matcher = Glob::new(&name).ok().map(|glob| glob.compile_matcher());
                    sections.push(EditorConfigSection {
                        name,
                        matcher,
                        ..EditorConfigSection::default()
                    });
                }
            }
            // Key-Value Pair (or Pair): contains a key and a value, separated by an `=`.
            if let Some(section) = sections.last_mut()
                && let Some((key, value)) = line.split_once('=')
            {
                let value = value.trim_start();
                let properties = &mut section.properties;
                match key.trim_end() {
                    "indent_style" => {
                        properties.indent_style = IndentStyle::parse(value);
                    }
                    "indent_size" => {
                        properties.indent_size = EditorConfigProperty::<usize>::parse(value);
                    }
                    "tab_width" => {
                        properties.tab_width = EditorConfigProperty::<usize>::parse(value);
                    }
                    "end_of_line" => {
                        properties.end_of_line = EditorConfigProperty::<EndOfLine>::parse(value);
                    }
                    "charset" => {
                        properties.charset = EditorConfigProperty::<Charset>::parse(value);
                    }
                    "trim_trailing_whitespace" => {
                        properties.trim_trailing_whitespace =
                            EditorConfigProperty::<bool>::parse(value);
                    }
                    "insert_final_newline" => {
                        properties.insert_final_newline =
                            EditorConfigProperty::<bool>::parse(value);
                    }
                    "max_line_length" => {
                        properties.max_line_length =
                            EditorConfigProperty::<MaxLineLength>::parse(value);
                    }
                    _ => {}
                }
            }
        }

        Self { root, sections, cwd: None }
    }

    /// Resolve a given path and return the resolved properties.
    /// If `cwd` is set, absolute paths will be resolved relative to `cwd`.
    pub fn resolve(&self, path: &Path) -> EditorConfigProperties {
        let path =
            if let Some(cwd) = &self.cwd { path.strip_prefix(cwd).unwrap_or(path) } else { path };
        let mut properties = EditorConfigProperties::default();
        for section in &self.sections {
            if section.matcher.as_ref().is_some_and(|matcher| matcher.is_match(path)) {
                properties.override_with(&section.properties);
            }
        }
        properties
    }
}

impl<T: Copy> EditorConfigProperty<T> {
    fn override_with(&mut self, other: &Self) {
        match other {
            Self::Value(value) => {
                *self = Self::Value(*value);
            }
            Self::Unset => {
                *self = Self::None;
            }
            Self::None => {}
        }
    }
}

impl EditorConfigProperties {
    fn override_with(&mut self, other: &Self) {
        self.indent_style.override_with(&other.indent_style);
        self.indent_size.override_with(&other.indent_size);
        self.tab_width.override_with(&other.tab_width);
        self.end_of_line.override_with(&other.end_of_line);
        self.charset.override_with(&other.charset);
        self.trim_trailing_whitespace.override_with(&other.trim_trailing_whitespace);
        self.insert_final_newline.override_with(&other.insert_final_newline);
        self.max_line_length.override_with(&other.max_line_length);
    }
}

impl EditorConfigProperty<usize> {
    fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("unset") {
            Self::Unset
        } else {
            s.parse::<usize>().map_or(Self::None, EditorConfigProperty::Value)
        }
    }
}

impl EditorConfigProperty<bool> {
    fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("true") {
            EditorConfigProperty::Value(true)
        } else if s.eq_ignore_ascii_case("false") {
            EditorConfigProperty::Value(false)
        } else if s.eq_ignore_ascii_case("unset") {
            EditorConfigProperty::Unset
        } else {
            EditorConfigProperty::None
        }
    }
}

impl IndentStyle {
    fn parse(s: &str) -> EditorConfigProperty<Self> {
        if s.eq_ignore_ascii_case("tab") {
            EditorConfigProperty::Value(Self::Tab)
        } else if s.eq_ignore_ascii_case("space") {
            EditorConfigProperty::Value(Self::Space)
        } else if s.eq_ignore_ascii_case("unset") {
            EditorConfigProperty::Unset
        } else {
            EditorConfigProperty::None
        }
    }
}

impl EditorConfigProperty<EndOfLine> {
    fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("lf") {
            Self::Value(EndOfLine::Lf)
        } else if s.eq_ignore_ascii_case("cr") {
            Self::Value(EndOfLine::Cr)
        } else if s.eq_ignore_ascii_case("crlf") {
            Self::Value(EndOfLine::Crlf)
        } else if s.eq_ignore_ascii_case("unset") {
            Self::Unset
        } else {
            Self::None
        }
    }
}

impl EditorConfigProperty<Charset> {
    fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("utf-8") {
            Self::Value(Charset::Utf8)
        } else if s.eq_ignore_ascii_case("latin1") {
            Self::Value(Charset::Latin1)
        } else if s.eq_ignore_ascii_case("utf-16be") {
            Self::Value(Charset::Utf16be)
        } else if s.eq_ignore_ascii_case("utf-16le") {
            Self::Value(Charset::Utf16le)
        } else if s.eq_ignore_ascii_case("utf-8-bom") {
            Self::Value(Charset::Utf8bom)
        } else if s.eq_ignore_ascii_case("unset") {
            Self::Unset
        } else {
            Self::None
        }
    }
}

impl EditorConfigProperty<MaxLineLength> {
    fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("off") {
            Self::Value(MaxLineLength::Off)
        } else if s.eq_ignore_ascii_case("unset") {
            Self::Unset
        } else if let Ok(n) = s.parse::<usize>() {
            Self::Value(MaxLineLength::Number(n))
        } else {
            Self::None
        }
    }
}
