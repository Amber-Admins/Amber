/// This module defines the logic for detecting correction signals in user messages.

/// Evaluates whether a user message contains correction signals.
/// Returns `Some(CorrectionSignal)` if detected, `None` otherwise.
pub fn detect_correction_signal(
    message: &str,
    previous_message: Option<&str>,
    pending_proposed_data: &[String],
) -> Option<CorrectionSignal> {
    let message_lower = message.to_lowercase();

    // 1. Explicit Phrase Scan
    for phrase in CORRECTION_PHRASES {
        if message_lower.contains(phrase) {
            return Some(CorrectionSignal::ExplicitPhrase {
                phrase: phrase.to_string(),
            });
        }
    }

    // 2. Direct Negation Scan
    if previous_message.is_some() {
        let prev_lower = previous_message.unwrap().to_lowercase();
        for word in prev_lower.split_whitespace() {
            // Check if current message negates a specific word/phrase from the previous message
            if message_lower.contains(&format!("not {}", word))
                || message_lower.contains(&format!("no, {}", word))
                || message_lower.contains(&format!("no {}", word))
                || message_lower.contains(&format!("it's {}, not {}", word, word))
            {
                return Some(CorrectionSignal::Negation {
                    negated_fragment: word.to_string(),
                });
            }
        }
    }

    // Check for contradictions with pending proposed data
    for pending in pending_proposed_data {
        // Check if current message contradicts a pending proposed field (e.g., "not X", "X is wrong")
        if message_lower.contains(&format!("not {}", pending.to_lowercase()))
            || message_lower.contains(&format!("{} is wrong", pending.to_lowercase()))
        {
            return Some(CorrectionSignal::ChangesetContradiction {
                contradicted_field: pending.clone(),
            });
        }
    }

    None
}

#[derive(Debug, Clone, PartialEq)]
pub enum CorrectionSignal {
    /// Explicit correction phrases: "actually," "wait," "I meant," "not X, Y," etc.
    ExplicitPhrase { phrase: String },
    /// Direct negation of a prior message value
    Negation { negated_fragment: String },
    /// Contradiction of a field in a pending changeset item
    ChangesetContradiction { contradicted_field: String },
}

const CORRECTION_PHRASES: &[&str] = &[
    "actually",
    "actually,",
    "wait,",
    "wait",
    "i meant",
    "not that",
    "correction",
    "correction:",
    "to clarify",
    "scratch that",
    "never mind",
    "nevermind",
    "no wait",
    "i was wrong",
    "let me correct",
    "that's wrong",
    "that's not right",
    "i misspoke",
];

/// Returns `true` if any correction signal is detected in the message.
pub fn has_correction_signal(
    message: &str,
    previous_message: Option<&str>,
    pending_proposed_data: &[String],
) -> bool {
    detect_correction_signal(message, previous_message, pending_proposed_data).is_some()
}
