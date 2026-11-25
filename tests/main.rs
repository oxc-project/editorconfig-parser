use std::path::{Path, PathBuf};

use editorconfig_parser::{
    Charset, EditorConfig, EditorConfigProperties, EditorConfigProperty::Value, EndOfLine,
    IndentStyle, MaxLineLength, QuoteType,
};

#[test]
fn empty() {
    let editor_config = EditorConfig::parse("");
    assert!(editor_config.sections().is_empty());
}

#[test]
fn comment() {
    let editor_config = EditorConfig::parse(";");
    assert!(editor_config.sections().is_empty());

    let editor_config = EditorConfig::parse("#");
    assert!(editor_config.sections().is_empty());
}

#[test]
fn root() {
    let editor_config = EditorConfig::parse("root = true");
    assert!(editor_config.sections().is_empty());
    assert!(editor_config.root());

    let editor_config = EditorConfig::parse("root = false");
    assert!(!editor_config.root());
}

#[test]
fn sections() {
    let editor_config = EditorConfig::parse(
        "
        [*]
        [**]
        [?]
    ",
    );
    assert_eq!(editor_config.sections().len(), 3);
}

#[test]
fn values() {
    let editor_config = EditorConfig::parse(
        "
        [*]
        charset = utf-8
        insert_final_newline = true
        end_of_line = lf
        indent_style = space
        indent_size = 2
        max_line_length = 80",
    );
    assert_eq!(editor_config.sections().len(), 1);
    let properties = &editor_config.sections()[0].properties;
    assert_eq!(properties.charset, Value(Charset::Utf8));
    assert_eq!(properties.insert_final_newline, Value(true));
    assert_eq!(properties.end_of_line, Value(EndOfLine::Lf));
    assert_eq!(properties.indent_style, Value(IndentStyle::Space));
    assert_eq!(properties.indent_size, Value(2));
    assert_eq!(properties.max_line_length, Value(MaxLineLength::Number(80)));
}

#[test]
fn quote_type() {
    let editor_config = EditorConfig::parse(
        "
        [*]
        quote_type = single",
    );
    let properties = &editor_config.sections()[0].properties;
    assert_eq!(properties.quote_type, Value(QuoteType::Single));

    let editor_config = EditorConfig::parse(
        "
        [*]
        quote_type = double",
    );
    let properties = &editor_config.sections()[0].properties;
    assert_eq!(properties.quote_type, Value(QuoteType::Double));

    let editor_config = EditorConfig::parse(
        "
        [*]
        quote_type = auto",
    );
    let properties = &editor_config.sections()[0].properties;
    assert_eq!(properties.quote_type, Value(QuoteType::Auto));
}

#[test]
fn quote_type_case_insensitive() {
    let editor_config = EditorConfig::parse(
        "
        [*]
        quote_type = SINGLE",
    );
    let properties = &editor_config.sections()[0].properties;
    assert_eq!(properties.quote_type, Value(QuoteType::Single));
}

#[test]
fn max_line_length_off() {
    let editor_config = EditorConfig::parse(
        "
        [*]
        max_line_length = off",
    );
    assert_eq!(editor_config.sections().len(), 1);
    let properties = &editor_config.sections()[0].properties;
    assert_eq!(properties.max_line_length, Value(MaxLineLength::Off));
}

#[test]
fn resolve() {
    let editor_config = EditorConfig::parse(
        "
        # *
        [*]
        charset = utf-8
        insert_final_newline = true
        end_of_line = lf
        indent_style = space
        indent_size = 2
        max_line_length = 80

        ; foo
        [*.foo]
        charset = latin1
        insert_final_newline = false
        end_of_line = crlf
        indent_style = tab
        indent_size = 4
        max_line_length = 100

        [*.{ts,tsx,js,jsx,mts,cts}]
        indent_size = 8
        max_line_length = 120

        [*.rs]
        max_line_length = 140

        [**/__snapshots__/**]
        max_line_length = 160
    ",
    );

    let path = Path::new("/");
    let all = editor_config.resolve(path);
    assert_eq!(all.charset, Value(Charset::Utf8));
    assert_eq!(all.insert_final_newline, Value(true));
    assert_eq!(all.end_of_line, Value(EndOfLine::Lf));
    assert_eq!(all.indent_style, Value(IndentStyle::Space));
    assert_eq!(all.indent_size, Value(2));
    assert_eq!(all.max_line_length, Value(MaxLineLength::Number(80)));

    let properties = editor_config.resolve(&path.join("file.foo"));
    assert_eq!(properties.charset, Value(Charset::Latin1));
    assert_eq!(properties.insert_final_newline, Value(false));
    assert_eq!(properties.end_of_line, Value(EndOfLine::Crlf));
    assert_eq!(properties.indent_style, Value(IndentStyle::Tab));
    assert_eq!(properties.indent_size, Value(4));
    assert_eq!(properties.max_line_length, Value(MaxLineLength::Number(100)));

    for ext in ["ts", "tsx", "js", "jsx", "mts", "cts"] {
        assert_eq!(
            editor_config.resolve(&path.join("file").with_extension(ext)),
            EditorConfigProperties {
                indent_size: Value(8),
                max_line_length: Value(MaxLineLength::Number(120)),
                ..all.clone()
            }
        );
    }

    assert_eq!(
        editor_config.resolve(&path.join("file.rs")),
        EditorConfigProperties {
            max_line_length: Value(MaxLineLength::Number(140)),
            ..all.clone()
        }
    );

    assert_eq!(
        editor_config.resolve(&path.join("dir").join("__snapshots__").join("file")),
        EditorConfigProperties {
            max_line_length: Value(MaxLineLength::Number(160)),
            ..all.clone()
        }
    );
}

#[test]
fn unset() {
    let editor_config = EditorConfig::parse(
        "
        [*]
        charset = utf-8
        insert_final_newline = true
        end_of_line = lf
        indent_style = space
        indent_size = 2
        max_line_length = 80

        [*.foo]
        charset = unset
        insert_final_newline = unset
        end_of_line = unset
        indent_style = unset
        indent_size = unset
        max_line_length = unset
        quote_type = unset
    ",
    );
    let path = Path::new("/").join("file.foo");
    let properties = editor_config.resolve(&path);
    assert_eq!(properties, EditorConfigProperties::default());
}

#[test]
fn resolve_with_cwd() {
    let editor_config = EditorConfig::parse(
        "
        [*]
        indent_size = 2

        [*.ts]
        indent_size = 4

        [src/**/*.ts]
        indent_size = 8
    ",
    )
    .with_cwd("/project");

    assert_eq!(editor_config.cwd(), Some(Path::new("/project")));

    // Absolute path should be resolved relative to cwd
    let properties = editor_config.resolve(Path::new("/project/file.ts"));
    assert_eq!(properties.indent_size, Value(4));

    let properties = editor_config.resolve(Path::new("/project/src/file.ts"));
    assert_eq!(properties.indent_size, Value(8));

    // Path not under cwd should still work (uses path as-is)
    let properties = editor_config.resolve(Path::new("/other/file.ts"));
    assert_eq!(properties.indent_size, Value(4));

    // Relative path should work as before
    let properties = editor_config.resolve(Path::new("file.ts"));
    assert_eq!(properties.indent_size, Value(4));
}

#[test]
fn resolve_with_cwd_pathbuf() {
    let cwd = PathBuf::from("/my/project");
    let editor_config = EditorConfig::parse(
        "
        [*.rs]
        indent_size = 4
    ",
    )
    .with_cwd(&cwd);

    let properties = editor_config.resolve(&cwd.join("main.rs"));
    assert_eq!(properties.indent_size, Value(4));
}

/// Resolve each path, and check the `indent_size` of the last matched section.
fn assert_indent_sizes(source: &str, cases: &[(&str, Option<usize>)]) {
    let editor_config = EditorConfig::parse(source);
    for &(path, expected) in cases {
        let actual = editor_config.resolve(Path::new(path)).indent_size;
        assert_eq!(actual, expected.map_or_default(Value), "{path}");
    }
}

/// Ported from https://github.com/editorconfig/editorconfig-core-test/blob/master/glob/braces.in
/// Skips unsupported cases:
/// - `\` escapes, which `globset` does not support on Windows
/// - Numeric ranges
#[test]
fn braces() {
    assert_indent_sizes(
        "
        [*.{py,js,html}]
        indent_size = 1
        [{single}.b]
        indent_size = 2
        [{}.c]
        indent_size = 3
        [a{b,c,}.d]
        indent_size = 4
        [a{,b,,c,}.e]
        indent_size = 5
        [{.f]
        indent_size = 6
        [{word,{also},this}.g]
        indent_size = 7
        [{{a,b},c}.k]
        indent_size = 8
        [{a,{b,c}}.l]
        indent_size = 9
        [{},b}.h]
        indent_size = 10
        [{{,b,c{d}.i]
        indent_size = 11
        [{some,a{*c,b}[ef]}.j]
        indent_size = 12
        [{aardvark..antelope}]
        indent_size = 13
    ",
        &[
            // word choice
            ("test.py", Some(1)),
            ("test.js", Some(1)),
            ("test.html", Some(1)),
            ("test.pyc", None),
            // single choice
            ("{single}.b", Some(2)),
            ("single.b", None),
            (".b", None),
            // empty choice
            ("{}.c", Some(3)),
            (".c", None),
            // choice with empty word
            ("a.d", Some(4)),
            ("ab.d", Some(4)),
            ("ac.d", Some(4)),
            ("a,.d", None),
            // choice with empty words
            ("a.e", Some(5)),
            ("ab.e", Some(5)),
            ("ac.e", Some(5)),
            ("a,.e", None),
            // no closing brace
            ("{.f", Some(6)),
            (".f", None),
            // nested braces
            ("word,this}.g", None),
            ("{also,this}.g", None),
            ("word.g", Some(7)),
            ("{also}.g", Some(7)),
            ("this.g", Some(7)),
            // nested braces, adjacent at start
            ("{{a,b},c}.k", None),
            ("{a,b}.k", None),
            ("a.k", Some(8)),
            ("b.k", Some(8)),
            ("c.k", Some(8)),
            // nested braces, adjacent at end
            ("{a,{b,c}}.l", None),
            ("{b,c}.l", None),
            ("a.l", Some(9)),
            ("b.l", Some(9)),
            ("c.l", Some(9)),
            // closing inside beginning
            ("{},b}.h", Some(10)),
            // missing closing braces
            ("{{,b,c{d}.i", Some(11)),
            ("{.i", None),
            ("b.i", None),
            ("c{d.i", None),
            (".i", None),
            // patterns nested in braces
            ("some.j", Some(12)),
            ("abe.j", Some(12)),
            ("abf.j", Some(12)),
            ("abg.j", None),
            ("ace.j", Some(12)),
            ("acf.j", Some(12)),
            ("acg.j", None),
            ("abce.j", Some(12)),
            ("abcf.j", Some(12)),
            ("abcg.j", None),
            ("ae.j", None),
            (".j", None),
            // alphabetical brace range: letters should not be considered for ranges
            ("{aardvark..antelope}", Some(13)),
            ("a", None),
            ("aardvark", None),
            ("agreement", None),
            ("antelope", None),
            ("antimatter", None),
        ],
    );
}
