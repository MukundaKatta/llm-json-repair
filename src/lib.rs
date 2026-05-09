//! # llm-json-repair
//!
//! Clean and parse JSON emitted by LLMs.
//!
//! Models still emit invalid JSON. They wrap it in markdown code fences,
//! leave trailing commas, write prose around it, or use smart quotes. This
//! crate does three local repair passes before you reach for a retry:
//!
//! 1. Strip markdown code fences (triple-backtick or triple-tilde).
//! 2. Extract the first balanced `{...}` or `[...]` from surrounding prose.
//! 3. Remove trailing commas before `}` or `]`.
//!
//! ```
//! let raw = r#"Sure, here you go:
//! ```json
//! { "answer": "Paris", "confidence": 0.95, }
//! ```"#;
//! let cleaned = llm_json_repair::repair(raw);
//! assert_eq!(cleaned, r#"{ "answer": "Paris", "confidence": 0.95 }"#);
//! ```
//!
//! With the default `serde` feature you can parse straight to a typed value:
//!
//! ```
//! # #[cfg(feature = "serde")]
//! # {
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Answer { value: String }
//!
//! let raw = "```\n{\"value\": \"42\",}\n```";
//! let parsed: Answer = llm_json_repair::parse(raw).unwrap();
//! assert_eq!(parsed.value, "42");
//! # }
//! ```

#![deny(missing_docs)]

mod balanced;
mod commas;
mod fences;

pub use balanced::extract_balanced;
pub use commas::strip_trailing_commas;
pub use fences::strip_fences;

use thiserror::Error;

/// Errors returned by [`parse`].
#[derive(Debug, Error)]
pub enum RepairError {
    /// The input could not be parsed even after repair.
    #[cfg(feature = "serde")]
    #[error("could not parse JSON after repair: {source}")]
    Parse {
        /// Underlying serde_json error from the final parse attempt.
        #[source]
        source: serde_json::Error,
        /// The repaired text we tried to parse.
        repaired: String,
    },

    /// No balanced `{…}` or `[…]` found in the input.
    #[error("no JSON object or array found in input")]
    NoJsonFound,
}

/// Apply all three repair passes and return the cleaned string.
///
/// This always returns *something* — even if the input contains no JSON, you
/// get the trimmed, fence-stripped text back. Use [`parse`] when you want
/// validation against a real JSON parser.
pub fn repair(input: &str) -> String {
    let unfenced = strip_fences(input);
    let extracted = extract_balanced(&unfenced)
        .unwrap_or(unfenced.trim())
        .to_string();
    strip_trailing_commas(&extracted)
}

/// Repair the input and parse it as JSON into the given type.
///
/// Equivalent to `serde_json::from_str(&repair(input))`, with cleaner errors
/// and an early failure when no JSON is present at all.
#[cfg(feature = "serde")]
pub fn parse<T>(input: &str) -> Result<T, RepairError>
where
    T: serde::de::DeserializeOwned,
{
    let unfenced = strip_fences(input);
    let extracted = match extract_balanced(&unfenced) {
        Some(s) => s.to_string(),
        None => return Err(RepairError::NoJsonFound),
    };
    let cleaned = strip_trailing_commas(&extracted);
    serde_json::from_str::<T>(&cleaned).map_err(|source| RepairError::Parse {
        source,
        repaired: cleaned,
    })
}

/// Parse without repair. Same as `serde_json::from_str` but typed under the
/// crate's error.
///
/// Useful when you want to record whether repair was needed:
///
/// ```ignore
/// let parsed = parse_strict(raw).or_else(|_| parse(raw))?;
/// ```
#[cfg(feature = "serde")]
pub fn parse_strict<T>(input: &str) -> Result<T, RepairError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str::<T>(input).map_err(|source| RepairError::Parse {
        source,
        repaired: input.to_string(),
    })
}
