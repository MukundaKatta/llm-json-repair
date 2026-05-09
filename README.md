# llm-json-repair

[![Crates.io](https://img.shields.io/crates/v/llm-json-repair.svg)](https://crates.io/crates/llm-json-repair)
[![Documentation](https://docs.rs/llm-json-repair/badge.svg)](https://docs.rs/llm-json-repair)
[![CI](https://github.com/MukundaKatta/llm-json-repair/actions/workflows/ci.yml/badge.svg)](https://github.com/MukundaKatta/llm-json-repair/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/llm-json-repair.svg)](https://crates.io/crates/llm-json-repair)

**Clean and parse JSON emitted by LLMs.**

Models still emit invalid JSON. They wrap it in markdown fences, leave
trailing commas, write prose around it, or only emit a half-fence. This
crate does three local repair passes before you reach for an LLM retry:

1. Strip markdown code fences (` ```json … ``` `, `~~~ … ~~~`).
2. Extract the first balanced `{…}` or `[…]` from surrounding prose.
3. Remove trailing commas before `}` or `]`.

Single small dependency on `thiserror`. Optional `serde` feature for
typed parsing.

## Install

```toml
[dependencies]
llm-json-repair = "0.1"
```

## Use

```rust
use llm_json_repair::repair;

let raw = "Sure, here you go:\n```json\n{\"answer\": \"Paris\", \"confidence\": 0.95,}\n```";
let cleaned = repair(raw);
assert_eq!(cleaned, r#"{"answer": "Paris", "confidence": 0.95}"#);
```

Typed parse with the default `serde` feature:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Answer {
    text: String,
    confidence: f64,
}

let raw = "```json\n{\"text\": \"hi\", \"confidence\": 0.5,}\n```";
let a: Answer = llm_json_repair::parse(raw)?;
```

## What it does NOT do

- No LLM retry loop. This is local repair only — wire your own retry path.
- No "fix anything" magic. We do exactly the three passes above. Nothing
  guesses missing fields, fixes spelling, or rewrites types.
- No regex. Single byte scan for the balanced extractor and the comma stripper.
- No JSON5/HJSON support. If the input has unquoted keys or single-quoted
  strings, repair won't fix that — by design.

## Why a separate crate

Most Rust LLM clients embed their own ad-hoc cleanup or punt to the user.
This crate exists so you can pull in one focused dependency instead of
copying a regex into every project.

Sibling to [`bedrock-kit`](https://github.com/MukundaKatta/bedrock-kit)
on the Python side, which does the same job for AWS Bedrock callers.

## License

MIT OR Apache-2.0
