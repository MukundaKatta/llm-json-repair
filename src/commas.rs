//! Strip trailing commas before `}` and `]`.
//!
//! Many LLMs emit `{"a": 1,}` or `[1, 2, 3,]` because their training data
//! includes JSON5 / JS object literals. serde_json rejects these. We do a
//! single-pass byte scan that respects string boundaries, removing any
//! comma followed by optional whitespace and then `}` or `]`.

/// Remove `, }` and `, ]` patterns outside of strings.
pub fn strip_trailing_commas(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());

    let mut in_str = false;
    let mut escaped = false;

    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];

        if in_str {
            out.push(b);
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }

        if b == b'"' {
            out.push(b);
            in_str = true;
            i += 1;
            continue;
        }

        if b == b',' {
            // peek forward through whitespace
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                // skip the comma; emit only the whitespace+close
                out.extend_from_slice(&bytes[(i + 1)..j]);
                out.push(bytes[j]);
                i = j + 1;
                continue;
            }
        }

        out.push(b);
        i += 1;
    }

    // out only contains valid UTF-8 because we copied bytes 1:1 (we only
    // skipped a single ASCII comma byte which can never split a multi-byte
    // codepoint).
    String::from_utf8(out).expect("repair preserves UTF-8 boundaries")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_change_on_valid() {
        assert_eq!(
            strip_trailing_commas(r#"{"a": 1, "b": 2}"#),
            r#"{"a": 1, "b": 2}"#
        );
    }

    #[test]
    fn strips_trailing_in_object() {
        assert_eq!(strip_trailing_commas(r#"{"a": 1,}"#), r#"{"a": 1}"#);
    }

    #[test]
    fn strips_trailing_with_whitespace() {
        assert_eq!(strip_trailing_commas("{\"a\": 1, \n}"), "{\"a\": 1 \n}");
    }

    #[test]
    fn strips_trailing_in_array() {
        assert_eq!(strip_trailing_commas("[1, 2, 3,]"), "[1, 2, 3]");
    }

    #[test]
    fn strips_in_nested_structure() {
        let input = r#"{"a": [1, 2,], "b": {"c": 3,},}"#;
        let want = r#"{"a": [1, 2], "b": {"c": 3}}"#;
        assert_eq!(strip_trailing_commas(input), want);
    }

    #[test]
    fn does_not_touch_commas_inside_strings() {
        let input = r#"{"x": "a, b, c,", "y": 1,}"#;
        let want = r#"{"x": "a, b, c,", "y": 1}"#;
        assert_eq!(strip_trailing_commas(input), want);
    }

    #[test]
    fn handles_escaped_quotes_in_strings() {
        let input = r#"{"x": "he said \", \"hi\"", "y": 1,}"#;
        let want = r#"{"x": "he said \", \"hi\"", "y": 1}"#;
        assert_eq!(strip_trailing_commas(input), want);
    }

    #[test]
    fn unicode_strings_preserved() {
        let input = "{\"emoji\": \"🦀\",}";
        let want = "{\"emoji\": \"🦀\"}";
        assert_eq!(strip_trailing_commas(input), want);
    }
}
