<div align="center">

# editorconfig-parser

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]

[![MIT licensed][license-badge]][license-url]
[![Build Status][ci-badge]][ci-url]
[![Code Coverage][code-coverage-badge]][code-coverage-url]
[![CodSpeed Badge][codspeed-badge]][codspeed-url]
[![Sponsors][sponsors-badge]][sponsors-url]
[![Discord chat][discord-badge]][discord-url]

</div>

A fast, spec-compliant Rust implementation of an [EditorConfig](https://editorconfig.org/) parser.

## Features

- **Spec-compliant** - fully implements the [EditorConfig specification](https://spec.editorconfig.org/)
- **Zero dependencies** - pure Rust implementation with no external dependencies
- **Fast and safe** - no unsafe code, optimized for performance
- **Comprehensive property support** - handles all standard EditorConfig properties
- **Path resolution** - resolves properties for specific file paths

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
editorconfig-parser = "0.0.1"
```

### Parsing an EditorConfig file

```rust
use editorconfig_parser::EditorConfig;

let config_text = r#"
root = true

[*]
indent_style = space
indent_size = 4
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true

[*.md]
max_line_length = off

[Makefile]
indent_style = tab
"#;

let config = EditorConfig::parse(config_text);

// Check if this is a root config
assert!(config.root());

// Access sections
for section in config.sections() {
    println!("Section: {}", section.name);
    if let Some(indent_style) = section.properties.indent_style {
        println!("  indent_style: {:?}", indent_style);
    }
}
```

### Resolving properties for a file path

```rust
use editorconfig_parser::EditorConfig;
use std::path::Path;

let config = EditorConfig::parse(config_text);
let properties = config.resolve(Path::new("src/main.rs"));
```

## Supported Properties

The parser supports all standard EditorConfig properties:

| Property | Type | Values |
|----------|------|--------|
| `indent_style` | `IdentStyle` | `tab`, `space` |
| `indent_size` | `usize` | Positive integer |
| `tab_width` | `usize` | Positive integer |
| `end_of_line` | `EndOfLine` | `lf`, `cr`, `crlf` |
| `charset` | `Charset` | `latin1`, `utf-8`, `utf-8-bom`, `utf-16be`, `utf-16le` |
| `trim_trailing_whitespace` | `bool` | `true`, `false` |
| `insert_final_newline` | `bool` | `true`, `false` |
| `max_line_length` | `MaxLineLength` | Positive integer or `off` |

Note: `max_line_length` is not part of the official EditorConfig spec but is commonly used by tools like [Prettier](https://prettier.io/docs/next/configuration#editorconfig).

## How It Works

The parser follows the [EditorConfig specification](https://spec.editorconfig.org/index.html#id6):

1. Reads the file line by line
2. Removes leading and trailing whitespace
3. Ignores blank lines and comments (`#` or `;`)
4. Parses `root = true` in the preamble (before any sections)
5. Parses section headers `[pattern]` as glob patterns
6. Parses key-value pairs `key = value` within sections
7. All values are case-insensitive

## Development

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

## License

MIT

## References

- [EditorConfig Specification](https://spec.editorconfig.org/)
- [EditorConfig Official Site](https://editorconfig.org/)

## [Sponsored By](https://github.com/sponsors/Boshen)

<p align="center">
  <a href="https://github.com/sponsors/Boshen">
    <img src="https://raw.githubusercontent.com/Boshen/sponsors/main/sponsors.svg" alt="My sponsors" />
  </a>
</p>

[discord-badge]: https://img.shields.io/discord/1079625926024900739?logo=discord&label=Discord
[discord-url]: https://discord.gg/9uXCAwqQZW
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-url]: https://github.com/oxc-project/editorconfig-parser/blob/main/LICENSE
[ci-badge]: https://github.com/oxc-project/editorconfig-parser/actions/workflows/ci.yml/badge.svg?event=push&branch=main
[ci-url]: https://github.com/oxc-project/editorconfig-parser/actions/workflows/ci.yml?query=event%3Apush+branch%3Amain
[code-coverage-badge]: https://codecov.io/github/oxc-project/editorconfig-parser/branch/main/graph/badge.svg
[code-coverage-url]: https://codecov.io/gh/oxc-project/editorconfig-parser
[sponsors-badge]: https://img.shields.io/github/sponsors/Boshen
[sponsors-url]: https://github.com/sponsors/Boshen
[codspeed-badge]: https://img.shields.io/endpoint?url=https://codspeed.io/badge.json
[codspeed-url]: https://codspeed.io/oxc-project/editorconfig-parser
[crates-badge]: https://img.shields.io/crates/d/editorconfig-parser?label=crates.io
[crates-url]: https://crates.io/crates/editorconfig-parser
[docs-badge]: https://img.shields.io/docsrs/editorconfig-parser
[docs-url]: https://docs.rs/editorconfig-parser
