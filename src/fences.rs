//! Strip markdown code fences from LLM output.
//!
//! Matches the patterns models actually emit: triple-backtick-with-tag
//! (`json` or `JSON`), triple-backtick-no-tag, and triple-tilde fences.
//! Some models pick `~~~` when the body contains backticks.
//!
//! We do not assume balanced fences. If the model only emitted the opener,
//! we still strip it. Trailing whitespace is removed.

/// Remove leading and trailing markdown fences if present, returning the
/// inner text. Whitespace around the result is trimmed.
pub fn strip_fences(input: &str) -> String {
    let mut text = input.trim();

    // Leading fence: optional language tag, e.g. ```json
    for fence in ["```", "~~~"] {
        if let Some(rest) = text.strip_prefix(fence) {
            // Optional language tag: an alphanumeric word right after the
            // opener. Only treat it as a tag if what follows it is a newline,
            // whitespace, or a JSON opener (`{`/`[`). Otherwise the word is the
            // body itself (e.g. a bare `true` / `null` / number in
            // "```true```") and stripping it would discard real content.
            let after_word = rest.trim_start_matches(|c: char| c.is_alphanumeric() || c == '_');
            let stripped_a_tag = after_word.len() != rest.len();
            let next = after_word.chars().next();
            let looks_like_tag = !stripped_a_tag
                || matches!(next, Some('\n') | Some('\r') | Some(' ') | Some('\t'))
                || matches!(next, Some('{') | Some('['));
            let after_tag = if looks_like_tag { after_word } else { rest };
            // accept "```json\n..." or "```\n..." — drop the newline if present
            text = after_tag
                .strip_prefix("\r\n")
                .or_else(|| after_tag.strip_prefix('\n'))
                .unwrap_or(after_tag);
            break;
        }
    }

    // Trailing fence
    for fence in ["```", "~~~"] {
        if let Some(stripped) = text.trim_end().strip_suffix(fence) {
            text = stripped;
            break;
        }
    }

    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_fences_passthrough() {
        assert_eq!(strip_fences("{\"a\": 1}"), "{\"a\": 1}");
    }

    #[test]
    fn strips_lower_json_tag() {
        let s = "```json\n{\"a\": 1}\n```";
        assert_eq!(strip_fences(s), "{\"a\": 1}");
    }

    #[test]
    fn strips_upper_json_tag() {
        let s = "```JSON\n{\"a\": 1}\n```";
        assert_eq!(strip_fences(s), "{\"a\": 1}");
    }

    #[test]
    fn strips_no_tag() {
        let s = "```\n{\"a\": 1}\n```";
        assert_eq!(strip_fences(s), "{\"a\": 1}");
    }

    #[test]
    fn strips_tilde_fence() {
        let s = "~~~\n{\"a\": 1}\n~~~";
        assert_eq!(strip_fences(s), "{\"a\": 1}");
    }

    #[test]
    fn strips_dangling_opener() {
        let s = "```json\n{\"a\": 1}";
        assert_eq!(strip_fences(s), "{\"a\": 1}");
    }

    #[test]
    fn strips_surrounding_whitespace() {
        let s = "   \n```json\n{\"a\": 1}\n```\n  ";
        assert_eq!(strip_fences(s), "{\"a\": 1}");
    }

    #[test]
    fn does_not_break_inline_code() {
        // No fence at start, internal backticks should survive
        let s = "the code `foo` is fine";
        assert_eq!(strip_fences(s), "the code `foo` is fine");
    }

    #[test]
    fn single_line_bare_scalar_body_is_not_eaten_as_tag() {
        // A bare scalar glued to the fences (no newline) must not be mistaken
        // for a language tag and discarded.
        assert_eq!(strip_fences("```true```"), "true");
        assert_eq!(strip_fences("```null```"), "null");
        assert_eq!(strip_fences("```42```"), "42");
    }

    #[test]
    fn single_line_tag_then_object() {
        // ```json{...} on one line: `json` is a tag, the object is the body.
        assert_eq!(strip_fences("```json{\"a\": 1}```"), "{\"a\": 1}");
    }

    #[test]
    fn strips_crlf_after_tag() {
        let s = "```json\r\n{\"a\": 1}\r\n```";
        assert_eq!(strip_fences(s), "{\"a\": 1}");
    }
}
