use std::path::Path;

fn is_japanese(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{309F}' | // Hiragana
        '\u{30A0}'..='\u{30FF}' | // Katakana
        '\u{4E00}'..='\u{9FFF}' | // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' | // CJK Extension A
        '\u{F900}'..='\u{FAFF}' | // CJK Compatibility Ideographs
        '\u{FF66}'..='\u{FF9F}'   // Halfwidth Katakana
    )
}

/// Convert only Japanese segments of a string via kakasi, preserving
/// ASCII and other non-Japanese characters as-is. This prevents kakasi
/// from mangling digits adjacent to kanji (e.g. `第10回` → `dai10kai`
/// instead of the broken `daiichi 0 kai`).
fn convert_segments(s: &str, separator: char) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(&c) = chars.peek() {
        if is_japanese(c) {
            // Collect a run of Japanese characters
            let mut japanese = String::new();
            while let Some(&c) = chars.peek() {
                if is_japanese(c) {
                    japanese.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            let converted = kakasi::convert(&japanese);
            let romaji = converted.romaji.to_lowercase();
            let romaji = romaji.replace(' ', "");
            result.push_str(&romaji);
        } else if c == ' ' {
            result.push(separator);
            chars.next();
        } else {
            result.push(c);
            chars.next();
        }
    }

    result
}

/// Convert a filename from Japanese to romaji.
///
/// Converts only the stem (filename without the final extension), treating
/// it as the user-intentional naming part. Structural parts of the filename
/// are preserved as-is:
/// - Leading dots (hidden file marker): `.設定.conf` → `.settei.conf`
/// - Extension (file type identifier): `テスト.txt` → `tesuto.txt`
/// - Spaces in the original name are replaced with `separator`
pub fn convert_filename(name: &str, separator: char) -> String {
    if name.is_empty() {
        return String::new();
    }

    // Preserve leading dot(s) for hidden files
    let (prefix, rest) = if name.starts_with('.') {
        let dot_end = name.find(|c: char| c != '.').unwrap_or(name.len());
        (&name[..dot_end], &name[dot_end..])
    } else {
        ("", name)
    };

    if rest.is_empty() {
        return name.to_string();
    }

    // Split extension from stem (use the rest after leading dots)
    let path = Path::new(rest);
    let ext = path.extension().and_then(|e| e.to_str());
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(rest);

    let converted = convert_segments(stem, separator);

    match ext {
        Some(e) => format!("{prefix}{converted}.{e}"),
        None => format!("{prefix}{converted}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_unchanged() {
        assert_eq!(convert_filename("hello.txt", '_'), "hello.txt");
    }

    #[test]
    fn hiragana() {
        let result = convert_filename("にほんご.txt", '_');
        assert_eq!(result, "nihongo.txt");
    }

    #[test]
    fn katakana() {
        let result = convert_filename("テスト.txt", '_');
        assert_eq!(result, "tesuto.txt");
    }

    #[test]
    fn kanji() {
        assert_eq!(convert_filename("日本語.md", '_'), "nihongo.md");
    }

    #[test]
    fn hidden_file() {
        assert_eq!(convert_filename(".にほんご", '_'), ".nihongo");
    }

    #[test]
    fn hidden_file_with_ext() {
        assert_eq!(convert_filename(".テスト.conf", '_'), ".tesuto.conf");
    }

    #[test]
    fn space_replacement() {
        assert_eq!(
            convert_filename("新しい ファイル.txt", '_'),
            "atarashii_fairu.txt"
        );
    }

    #[test]
    fn custom_separator() {
        assert_eq!(
            convert_filename("新しい ファイル.txt", '-'),
            "atarashii-fairu.txt"
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(convert_filename("", '_'), "");
    }

    #[test]
    fn dots_only() {
        assert_eq!(convert_filename(".", '_'), ".");
        assert_eq!(convert_filename("..", '_'), "..");
    }

    #[test]
    fn no_extension() {
        let result = convert_filename("テスト", '_');
        assert_eq!(result, "tesuto");
    }

    #[test]
    fn digits_preserved_adjacent_to_kanji() {
        assert_eq!(convert_filename("第10回.mp4", '_'), "dai10kai.mp4");
    }

    #[test]
    fn mixed_japanese_ascii_digits() {
        assert_eq!(convert_filename("報告書_v2.pdf", '_'), "houkokusho_v2.pdf");
    }
}
