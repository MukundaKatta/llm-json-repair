//! Find the first balanced JSON object or array in a longer string.
//!
//! Models often wrap JSON in prose ("Sure, here you go: {…} let me know"),
//! so we scan the bytes from left to right, find the first `{` or `[`,
//! then walk forward tracking string state and brace depth until we close
//! the structure. Strings are honored so that `{` inside a quoted value
//! doesn't increase depth. Backslash-escaped quotes are honored too.
//!
//! Returns a slice of the input pointing at the matched substring, so the
//! caller pays no allocation when the input was already valid.

/// Return the first balanced `{…}` or `[…]` substring found in `input`,
/// or `None` if no balanced structure exists.
pub fn extract_balanced(input: &str) -> Option<&str> {
    let bytes = input.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        let b = bytes[i];
        if b == b'{' || b == b'[' {
            if let Some(end) = scan_balanced(bytes, i) {
                // Safety: scan_balanced returns indices on UTF-8 boundaries
                // because we only advance past ASCII structural bytes ({, [,
                // }, ], "); inside strings we step byte-by-byte but the slice
                // still cuts at the matching close brace.
                return Some(&input[i..=end]);
            }
        }
        i += 1;
    }
    None
}

/// Walk forward from `start` (which must be `{` or `[`) and return the index
/// of the matching close brace. Returns None if the structure never closes.
fn scan_balanced(bytes: &[u8], start: usize) -> Option<usize> {
    let open = bytes[start];
    let close = match open {
        b'{' => b'}',
        b'[' => b']',
        _ => return None,
    };

    let mut depth: i32 = 0;
    let mut in_str = false;
    let mut escaped = false;

    let mut i = start;
    while i < bytes.len() {
        let b = bytes[i];

        if escaped {
            escaped = false;
            i += 1;
            continue;
        }

        if in_str {
            match b {
                b'\\' => escaped = true,
                b'"' => in_str = false,
                _ => {}
            }
            i += 1;
            continue;
        }

        match b {
            b'"' => in_str = true,
            x if x == open => depth += 1,
            x if x == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_object() {
        let s = r#"{"a": 1}"#;
        assert_eq!(extract_balanced(s), Some(s));
    }

    #[test]
    fn passthrough_array() {
        let s = r#"[1, 2, 3]"#;
        assert_eq!(extract_balanced(s), Some(s));
    }

    #[test]
    fn extracts_from_prose() {
        let s = r#"Here you go: {"a": 1} let me know."#;
        assert_eq!(extract_balanced(s), Some(r#"{"a": 1}"#));
    }

    #[test]
    fn handles_nested_objects() {
        let s = r#"{"outer": {"inner": [1, 2]}}"#;
        assert_eq!(extract_balanced(s), Some(s));
    }

    #[test]
    fn ignores_braces_inside_strings() {
        let s = r#"{"text": "hi { there } } } [", "n": 1}"#;
        assert_eq!(extract_balanced(s), Some(s));
    }

    #[test]
    fn ignores_escaped_quotes() {
        let s = r#"{"text": "she said \"hi\"", "n": 1}"#;
        assert_eq!(extract_balanced(s), Some(s));
    }

    #[test]
    fn returns_none_when_no_json() {
        assert_eq!(extract_balanced("just some prose, no json here"), None);
    }

    #[test]
    fn returns_none_for_unclosed() {
        assert_eq!(extract_balanced(r#"{"a": 1, "b": "#), None);
    }

    #[test]
    fn first_balanced_wins() {
        let s = r#"first {"a": 1} then {"b": 2}"#;
        assert_eq!(extract_balanced(s), Some(r#"{"a": 1}"#));
    }

    #[test]
    fn array_inside_prose() {
        let s = r#"My answer is [1, 2, 3]. Hope that helps!"#;
        assert_eq!(extract_balanced(s), Some(r#"[1, 2, 3]"#));
    }

    #[test]
    fn empty_object() {
        assert_eq!(extract_balanced(r#"prose {} more"#), Some(r#"{}"#));
    }
}
