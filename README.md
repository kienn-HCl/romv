# romv

A CLI tool that renames Japanese filenames to romaji.

[日本語版 README](README.ja.md)

## Install

### Nix

```bash
nix profile install github:frort/romv

# or build locally
nix build
./result/bin/romv
```

### Cargo

```bash
cargo install --path .

# or build locally
cargo build --release
./target/release/romv
```

## Usage

```bash
# Preview (dry-run, default)
romv テスト.txt 日本語.md
# テスト.txt -> tesuto.txt (dry-run)
# 日本語.md -> nihongo.md (dry-run)

# Execute
romv -y テスト.txt 日本語.md

# Pipe input
ls | romv -y

# Interactive mode (confirm each rename)
romv -i *.txt
```

## Options

```
romv [OPTIONS] [FILES]...

Arguments:
  [FILES]...    Files to rename (reads from stdin if omitted)

Options:
  -y, --yes            Execute renames (default is dry-run preview)
  -i, --interactive    Confirm each rename with y/N
  -v, --verbose        Show each operation
  -s, --separator <C>  Character to replace spaces with (default: '_')
  -h, --help           Show help
  -V, --version        Show version
```

## Conversion rules

- Converts kanji, hiragana, and katakana to romaji (using [kakasi](https://crates.io/crates/kakasi))
- Preserves file extensions: `テスト.txt` → `tesuto.txt`
- Preserves leading dots for hidden files: `.設定.conf` → `.settei.conf`
- ASCII characters and digits are kept as-is: `第10回.mp4` → `dai10kai.mp4`
- Only spaces present in the original filename are replaced with `_` (configurable with `-s`)
- Skips filenames that are already ASCII-only

## Safety

- **Dry-run by default**: preview only without `-y`
- **No-clobber**: skips and reports an error if the target already exists
- **Batch collision detection**: aborts before renaming if multiple files would map to the same name

## Development

```bash
# Set up direnv (optional)
echo 'use flake' > .envrc && direnv allow

# Or enter the dev shell manually
nix develop

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets -- --deny warnings

# Format
nix fmt

# Build docs
nix build .#doc
```

## License

GPL-3.0 (due to the kakasi crate)
