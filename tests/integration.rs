//! End-to-end tests that exercise the public API the way callers will.

use llm_json_repair::repair;

#[cfg(feature = "serde")]
use llm_json_repair::{parse, parse_strict, RepairError};

#[cfg(feature = "serde")]
use serde::Deserialize;

#[cfg(feature = "serde")]
#[derive(Deserialize, Debug, PartialEq)]
struct Answer {
    text: String,
    confidence: f64,
}

#[test]
fn repair_clean_json_is_idempotent() {
    let s = r#"{"a": 1, "b": [1, 2, 3]}"#;
    assert_eq!(repair(s), s);
}

#[test]
fn repair_handles_fenced_with_trailing_comma_and_prose() {
    let s = "Here is the answer:\n```json\n{\"a\": 1, \"b\": 2,}\n```\nlet me know!";
    let cleaned = repair(s);
    assert_eq!(cleaned, "{\"a\": 1, \"b\": 2}");
}

#[test]
fn repair_returns_input_when_no_json_present() {
    let s = "no json here, just prose";
    assert_eq!(repair(s), s);
}

#[cfg(feature = "serde")]
#[test]
fn parse_clean_json() {
    let s = r#"{"text": "hello", "confidence": 0.9}"#;
    let a: Answer = parse(s).unwrap();
    assert_eq!(
        a,
        Answer {
            text: "hello".into(),
            confidence: 0.9
        }
    );
}

#[cfg(feature = "serde")]
#[test]
fn parse_repairs_fenced_with_trailing_comma() {
    let s = "```json\n{\"text\": \"hi\", \"confidence\": 0.5,}\n```";
    let a: Answer = parse(s).unwrap();
    assert_eq!(
        a,
        Answer {
            text: "hi".into(),
            confidence: 0.5
        }
    );
}

#[cfg(feature = "serde")]
#[test]
fn parse_extracts_from_prose() {
    let s = r#"Sure, here's the answer: {"text": "ok", "confidence": 1.0} :)"#;
    let a: Answer = parse(s).unwrap();
    assert_eq!(a.text, "ok");
}

#[cfg(feature = "serde")]
#[test]
fn parse_returns_no_json_found_for_pure_prose() {
    let err = parse::<Answer>("nothing parseable here").unwrap_err();
    assert!(matches!(err, RepairError::NoJsonFound));
}

#[cfg(feature = "serde")]
#[test]
fn parse_returns_parse_error_with_repaired_text() {
    let s = r#"{"text": "hi"}"#; // missing required `confidence`
    let err = parse::<Answer>(s).unwrap_err();
    match err {
        RepairError::Parse { repaired, .. } => assert_eq!(repaired, s),
        _ => panic!("expected Parse"),
    }
}

#[cfg(feature = "serde")]
#[test]
fn parse_strict_does_not_repair() {
    let s = r#"{"text": "hi", "confidence": 0.5,}"#;
    assert!(parse_strict::<Answer>(s).is_err());
    // ...but repair-based parse handles it
    let a: Answer = parse(s).unwrap();
    assert_eq!(a.confidence, 0.5);
}

#[cfg(feature = "serde")]
#[test]
fn parse_handles_array_at_root() {
    let s = "```\n[1, 2, 3,]\n```";
    let v: Vec<i32> = parse(s).unwrap();
    assert_eq!(v, vec![1, 2, 3]);
}

#[cfg(feature = "serde")]
#[test]
fn parse_preserves_unicode() {
    #[derive(Deserialize)]
    struct E {
        emoji: String,
    }
    let s = "{\"emoji\": \"🦀\",}";
    let e: E = parse(s).unwrap();
    assert_eq!(e.emoji, "🦀");
}

#[cfg(feature = "serde")]
#[test]
fn parse_does_not_get_confused_by_braces_in_strings() {
    #[derive(Deserialize)]
    struct M {
        msg: String,
    }
    let s = r#"prose {"msg": "this } and { stay inside"} more prose"#;
    let m: M = parse(s).unwrap();
    assert_eq!(m.msg, "this } and { stay inside");
}
