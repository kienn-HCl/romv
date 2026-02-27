# romv

日本語ファイル名をローマ字に変換してリネームするCLIツール。

[English README](README.md)

## インストール

### Nix

```bash
nix profile install github:frort/romv

# ローカルビルド
nix build
./result/bin/romv
```

### Cargo

```bash
cargo install --path .

# ローカルビルド
cargo build --release
./target/release/romv
```

## 使い方

```bash
# プレビュー（dry-run、デフォルト）
romv テスト.txt 日本語.md
# テスト.txt -> tesuto.txt (dry-run)
# 日本語.md -> nihongo.md (dry-run)

# 実行
romv -y テスト.txt 日本語.md

# パイプ入力
ls | romv -y

# 対話モード（1件ずつ確認）
romv -i *.txt
```

## オプション

```
romv [OPTIONS] [FILES]...

引数:
  [FILES]...    リネーム対象（省略時はstdinから読み取り）

オプション:
  -y, --yes            実行する（省略時はdry-runでプレビューのみ）
  -i, --interactive    各リネームで y/N 確認
  -v, --verbose        各操作を表示
  -s, --separator <C>  スペース置換文字（デフォルト: '_'）
  -h, --help           ヘルプ表示
  -V, --version        バージョン表示
```

## 変換ルール

- 漢字・ひらがな・カタカナをローマ字に変換（[kakasi](https://crates.io/crates/kakasi)使用）
- 拡張子は保持: `テスト.txt` → `tesuto.txt`
- 隠しファイルの先頭ドットを保持: `.設定.conf` → `.settei.conf`
- ASCII文字・数字はそのまま保持: `第10回.mp4` → `dai10kai.mp4`
- 元のファイル名にあるスペースのみ `_` に置換（`-s` で変更可能）
- 変換後にファイル名が変わらない場合はスキップ

## 安全機構

- **dry-runデフォルト**: `-y` なしではプレビューのみ
- **no-clobber**: 変換先が既に存在する場合はスキップしてエラー
- **バッチ衝突検出**: 複数ファイルが同じ名前に変換される場合、実行前に全体中止

## 開発

```bash
# direnv のセットアップ（任意）
echo 'use flake' > .envrc && direnv allow

# または手動で dev shell に入る
nix develop

# テスト
cargo test

# clippy
cargo clippy --all-targets -- --deny warnings

# フォーマット
nix fmt

# ドキュメントビルド
nix build .#doc
```

## ライセンス

GPL-3.0（kakasi crateの制約による）
