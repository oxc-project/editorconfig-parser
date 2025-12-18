use std::path::{Path, PathBuf};

use editorconfig_parser::{
    Charset, EditorConfig, EditorConfigProperties, EditorConfigProperty::Value, EndOfLine,
    IndentStyle, MaxLineLength,
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
