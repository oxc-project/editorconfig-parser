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
/// Skips numeric ranges, which are not supported yet,
/// and `\.txt`, which is not a file name on Windows.
#[test]
fn braces() {
    assert_indent_sizes(
        r"
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
        [{a\,b,cd}.txt]
        indent_size = 14
        [{e,\},f}.txt]
        indent_size = 15
        [{g,\\,i}.txt]
        indent_size = 16
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
            // escaped comma
            ("a,b.txt", Some(14)),
            ("a.txt", None),
            ("cd.txt", Some(14)),
            // escaped closing brace
            ("e.txt", Some(15)),
            ("}.txt", Some(15)),
            ("f.txt", Some(15)),
            // escaped backslash
            ("g.txt", Some(16)),
            ("i.txt", Some(16)),
        ],
    );
}

/// Ported from https://github.com/editorconfig/editorconfig-core-test/blob/master/glob/star.in
/// and `glob/question.in`.
#[test]
fn star_and_question() {
    assert_indent_sizes(
        "
        [*]
        indent_size = 1
        [a*e.c]
        indent_size = 2
        [Bar/*]
        indent_size = 3
        [som?.c]
        indent_size = 4
    ",
        &[
            // star
            ("ace.c", Some(2)),
            ("ae.c", Some(2)),
            ("abcde.c", Some(2)),
            ("a/e.c", Some(1)),
            ("Bar/foo.txt", Some(3)),
            ("Bar/.editorconfig", Some(3)),
            (".editorconfig", Some(1)),
            ("bat/Bar/foo.txt", Some(1)),
            ("bat/Bar/.editorconfig", Some(1)),
            // question
            ("some.c", Some(4)),
            ("som.c", Some(1)),
            ("something.c", Some(1)),
            ("som/.c", Some(1)),
        ],
    );
}

/// Ported from https://github.com/editorconfig/editorconfig-core-test/blob/master/glob/star_star.in
/// and the example on https://editorconfig.org
#[test]
fn star_star() {
    assert_indent_sizes(
        "
        [a**z.c]
        indent_size = 1
        [b/**z.c]
        indent_size = 2
        [c**/z.c]
        indent_size = 3
        [d/**/z.c]
        indent_size = 4
        [lib/**.js]
        indent_size = 5
    ",
        &[
            ("a/z.c", Some(1)),
            ("amnz.c", Some(1)),
            ("am/nz.c", Some(1)),
            ("a/mnz.c", Some(1)),
            ("amn/z.c", Some(1)),
            ("a/mn/z.c", Some(1)),
            ("b/z.c", Some(2)),
            ("b/mnz.c", Some(2)),
            ("b/mn/z.c", Some(2)),
            ("bmnz.c", None),
            ("bm/nz.c", None),
            ("bmn/z.c", None),
            ("c/z.c", Some(3)),
            ("cmn/z.c", Some(3)),
            ("c/mn/z.c", Some(3)),
            ("cmnz.c", None),
            ("cm/nz.c", None),
            ("c/mnz.c", None),
            ("d/z.c", Some(4)),
            ("d/mn/z.c", Some(4)),
            ("dmnz.c", None),
            ("dm/nz.c", None),
            ("d/mnz.c", None),
            ("dmn/z.c", None),
            ("lib/a.js", Some(5)),
            ("lib/a/b.js", Some(5)),
            ("src/lib/a.js", None),
        ],
    );
}

/// Ported from https://github.com/editorconfig/editorconfig-core-test/blob/master/filetree/path_separator.in
/// Skips `windows\separator2`, which is not a file name on Windows.
#[test]
fn path_separator() {
    assert_indent_sizes(
        r"
        [path/separator]
        indent_size = 1
        [/top/of/path]
        indent_size = 2
        [windows\separator]
        indent_size = 3
        [windows\\separator2]
        indent_size = 4
    ",
        &[
            ("path/separator", Some(1)),
            ("nested/path/separator", None),
            ("top/of/path", Some(2)),
            ("not/top/of/path", None),
            ("windows/separator", None),
            ("windows/separator2", None),
        ],
    );
}

/// Ported from https://github.com/editorconfig/editorconfig-core-test/blob/master/parser/leading_slash.in
#[test]
fn leading_slash() {
    let editor_config = EditorConfig::parse(
        "
        [/test1/file.{java,js,rb}]
        [test1/file.{java,js,rb}]
        [*.{java,js,rb}]
    ",
    );
    for section in editor_config.sections() {
        let is_match = section.matcher.as_ref().is_some_and(|m| m.is_match("test1/file.java"));
        assert!(is_match, "[{}]", section.name);
    }
}
