#[derive(Debug, Default, Clone)]
pub struct EditorConfig<'a> {
    /// Set to true to tell the core not to check any higher directory for EditorConfig settings for on the current filename.
    root: bool,

    sections: Vec<EditorConfigSection<'a>>,
}

impl<'a> EditorConfig<'a> {
    pub fn root(&self) -> bool {
        self.root
    }

    pub fn sections(&self) -> &[EditorConfigSection<'a>] {
        &self.sections
    }
}

/// <https://spec.editorconfig.org/index.html>
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct EditorConfigSection<'a> {
    /// Section Name: the string between the beginning `[` and the ending `]`.
    pub name: &'a str,

    /// Set to tab or space to use tabs or spaces for indentation, respectively.
    /// Option tab implies that an indentation is to be filled by as many hard tabs as possible, with the rest of the indentation filled by spaces.
    /// A non-normative explanation can be found in the indentation_ section.
    /// The values are case-insensitive.
    pub indent_style: Option<IdentStyle>,

    /// Set to a whole number defining the number of columns used for each indentation level and the width of soft tabs (when supported).
    /// If this equals tab, the indent_size shall be set to the tab size, which should be tab_width (if specified); else, the tab size set by the editor.
    /// The values are case-insensitive.
    pub indent_size: Option<usize>,

    /// Set to a whole number defining the number of columns used to represent a tab character.
    /// This defaults to the value of indent_size and should not usually need to be specified.
    pub tab_width: Option<usize>,

    /// Set to lf, cr, or crlf to control how line breaks are represented.
    /// The values are case-insensitive.
    pub end_of_line: Option<EndOfLine>,

    /// Set to latin1, utf-8, utf-8-bom, utf-16be or utf-16le to control the character set.
    /// Use of utf-8-bom is discouraged.
    /// The values are case-insensitive.
    pub charset: Option<Charset>,

    /// Set to true to remove all whitespace characters preceding newline characters in the file and false to ensure it doesn’t.
    pub trim_trailing_whitespace: Option<bool>,

    /// Set to true ensure file ends with a newline when saving and false to ensure it doesn’t.
    /// Editors must not insert newlines in empty files when saving those files, even if insert_final_newline = true.
    pub insert_final_newline: Option<bool>,

    /// Prettier print width.
    /// Not part of spec <https://github.com/editorconfig/editorconfig-vscode/issues/53#issuecomment-462432616>
    /// But documented in <https://prettier.io/docs/next/configuration#editorconfig>
    pub max_line_length: Option<usize>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum IdentStyle {
    Tab,
    Space,
}

impl IdentStyle {
    fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("tab") {
            Some(Self::Tab)
        } else if s.eq_ignore_ascii_case("space") {
            Some(Self::Space)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EndOfLine {
    Lf,
    Cr,
    Crlf,
}

impl EndOfLine {
    fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("lf") {
            Some(Self::Lf)
        } else if s.eq_ignore_ascii_case("cr") {
            Some(Self::Cr)
        } else if s.eq_ignore_ascii_case("crlf") {
            Some(Self::Crlf)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Charset {
    Latin1,
    Utf8,
    Utf8bom,
    Utf16be,
    Utf16le,
}

impl Charset {
    fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("utf-8") {
            Some(Self::Utf8)
        } else if s.eq_ignore_ascii_case("latin1") {
            Some(Self::Latin1)
        } else if s.eq_ignore_ascii_case("utf-16be") {
            Some(Self::Utf16be)
        } else if s.eq_ignore_ascii_case("utf-16le") {
            Some(Self::Utf16le)
        } else if s.eq_ignore_ascii_case("utf-8-bom") {
            Some(Self::Utf8bom)
        } else {
            None
        }
    }
}

impl<'a> EditorConfig<'a> {
    /// <https://spec.editorconfig.org/index.html#id6>
    pub fn parse(source_text: &'a str) -> Self {
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
                    let name = line;
                    sections.push(EditorConfigSection { name, ..EditorConfigSection::default() });
                }
            }
            // Key-Value Pair (or Pair): contains a key and a value, separated by an `=`.
            if let Some(section) = sections.last_mut()
                && let Some((key, value)) = line.split_once('=')
            {
                let value = value.trim_start();
                match key.trim_end() {
                    "indent_style" => {
                        section.indent_style = IdentStyle::parse(value);
                    }
                    "indent_size" => {
                        section.indent_size = parse_number(value);
                    }
                    "tab_width" => {
                        section.indent_size = parse_number(value);
                    }
                    "end_of_line" => {
                        section.end_of_line = EndOfLine::parse(value);
                    }
                    "charset" => {
                        section.charset = Charset::parse(value);
                    }
                    "trim_trailing_whitespace" => {
                        section.trim_trailing_whitespace = parse_bool(value);
                    }
                    "insert_final_newline" => {
                        section.insert_final_newline = parse_bool(value);
                    }
                    "max_line_length" => {
                        section.max_line_length = parse_number(value);
                    }
                    _ => {}
                }
            }
        }

        Self { root, sections }
    }
}

fn parse_number(s: &str) -> Option<usize> {
    s.parse::<usize>().ok()
}

fn parse_bool(s: &str) -> Option<bool> {
    if s.eq_ignore_ascii_case("true") {
        Some(true)
    } else if s.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}
