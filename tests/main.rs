use editorconfig_parser::{Charset, EditorConfig, EndOfLine, IdentStyle, MaxLineLength};

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
    let section = &editor_config.sections()[0];
    assert_eq!(section.charset, Some(Charset::Utf8));
    assert_eq!(section.insert_final_newline, Some(true));
    assert_eq!(section.end_of_line, Some(EndOfLine::Lf));
    assert_eq!(section.indent_style, Some(IdentStyle::Space));
    assert_eq!(section.indent_size, Some(2));
    assert_eq!(section.max_line_length, Some(MaxLineLength::Number(80)));
}

#[test]
fn max_line_length_off() {
    let editor_config = EditorConfig::parse(
        "
        [*]
        max_line_length = off",
    );
    assert_eq!(editor_config.sections().len(), 1);
    let section = &editor_config.sections()[0];
    assert_eq!(section.max_line_length, Some(MaxLineLength::Off));
}
