//! Confidence labels for retrieval answers.
//!
//! `AnswerState` is the serialized verdict later runtime work attaches to
//! every search answer: whether the pipeline is confident, borderline, or has
//! no answer at all. The serde representation (`confident` / `borderline` /
//! `no_answer`) is a stable contract — do not rename variants without a
//! migration.

/// Confidence state of a retrieval answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerState {
    Confident,
    Borderline,
    NoAnswer,
}
