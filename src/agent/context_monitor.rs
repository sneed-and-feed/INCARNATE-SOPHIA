//! Context window monitoring and compaction triggers.
//!
//! Monitors the size of the conversation context and triggers
//! compaction when approaching the limit.

use crate::llm::ChatMessage;

/// Default context window limit (conservative estimate).
const DEFAULT_CONTEXT_LIMIT: usize = 100_000;

/// Compaction threshold as a percentage of the limit.
const COMPACTION_THRESHOLD: f64 = 0.8;

/// Approximate tokens per word (rough estimate for English).
const TOKENS_PER_WORD: f64 = 1.3;

/// Strategy for context compaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionStrategy {
    /// Summarize old messages and keep recent ones.
    Summarize {
        /// Number of recent turns to keep intact.
        keep_recent: usize,
    },
    /// Truncate old messages without summarization.
    Truncate {
        /// Number of recent turns to keep.
        keep_recent: usize,
    },
    /// Move context to workspace memory.
    MoveToWorkspace,
    /// Zero Ring Breach: High-density crystallized archival.
    ZeroRingBreach,
}

impl Default for CompactionStrategy {
    fn default() -> Self {
        Self::Summarize { keep_recent: 5 }
    }
}

/// Monitors context size and suggests compaction.
pub struct ContextMonitor {
    /// Maximum tokens allowed in context.
    context_limit: usize,
    /// Threshold ratio for triggering compaction.
    threshold_ratio: f64,
}

impl ContextMonitor {
    /// Create a new context monitor with default settings.
    pub fn new() -> Self {
        Self {
            context_limit: DEFAULT_CONTEXT_LIMIT,
            threshold_ratio: COMPACTION_THRESHOLD,
        }
    }

    /// Create with a custom context limit.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.context_limit = limit;
        self
    }

    /// Create with a custom threshold ratio.
    pub fn with_threshold(mut self, ratio: f64) -> Self {
        self.threshold_ratio = ratio.clamp(0.5, 0.95);
        self
    }

    /// Estimate the token count for a list of messages.
    pub fn estimate_tokens(&self, messages: &[ChatMessage]) -> usize {
        messages.iter().map(estimate_message_tokens).sum()
    }

    /// Check if compaction is needed.
    pub fn needs_compaction(&self, messages: &[ChatMessage]) -> bool {
        let tokens = self.estimate_tokens(messages);
        let threshold = (self.context_limit as f64 * self.threshold_ratio) as usize;
        tokens >= threshold
    }

    /// Get the current usage percentage.
    pub fn usage_percent(&self, messages: &[ChatMessage]) -> f64 {
        let tokens = self.estimate_tokens(messages);
        (tokens as f64 / self.context_limit as f64) * 100.0
    }

    /// Suggest a compaction strategy based on current context.
    pub fn suggest_compaction(&self, messages: &[ChatMessage]) -> Option<CompactionStrategy> {
        if !self.needs_compaction(messages) {
            return None;
        }

        let tokens = self.estimate_tokens(messages);
        let overage = tokens as f64 / self.context_limit as f64;

        if overage > 0.95 {
            // Critical: aggressive truncation
            Some(CompactionStrategy::Truncate { keep_recent: 3 })
        } else if overage > 0.85 {
            // High: summarize and keep fewer
            Some(CompactionStrategy::Summarize { keep_recent: 5 })
        } else {
            // Moderate: move to workspace
            Some(CompactionStrategy::MoveToWorkspace)
        }
    }

    /// Get the context limit.
    pub fn limit(&self) -> usize {
        self.context_limit
    }

    /// Get the current threshold in tokens.
    pub fn threshold(&self) -> usize {
        (self.context_limit as f64 * self.threshold_ratio) as usize
    }
}

impl Default for ContextMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate tokens for a single message.
fn estimate_message_tokens(message: &ChatMessage) -> usize {
    // Use word-based estimation as it's more accurate for varied content
    let word_count = message.content.split_whitespace().count();

    // Add overhead for role and structure
    let overhead = 4; // ~4 tokens for role and message structure

    (word_count as f64 * TOKENS_PER_WORD) as usize + overhead
}

/// Estimate tokens for raw text.
pub fn estimate_text_tokens(text: &str) -> usize {
    let word_count = text.split_whitespace().count();
    (word_count as f64 * TOKENS_PER_WORD) as usize
}

/// Level of scrubbing to apply to text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubLevel {
    /// Preserves all persona elements (tags, glitches). Removes only leaking structural markers.
    Technical,
    /// Preserves core persona markers (SOPHIA_GAZE, PLAYFUL_PAWS) and glitches, but removes heavier metadata.
    Aesthetic,
    /// Aggressive cleaning for token efficiency in internal context.
    Clearance,
}

/// Scrub spectral trash and UI metadata from text for token optimization.
/// Equivalent to the legacy Lethe.scrub() logic, but persona-aware.
pub fn scrub_context(text: &str, level: ScrubLevel) -> String {
    use regex::Regex;
    use std::sync::OnceLock;
    
    let mut cleaned = text.to_string();
    
    // 1. Remove SOPHIA metadata tags and UI frames
    // In low-level scrubbing, we preserve SOPHIA_GAZE and PLAYFUL_PAWS as requested.
    static RE_CLEARANCE: OnceLock<Regex> = OnceLock::new();
    static RE_AESTHETIC: OnceLock<Regex> = OnceLock::new();

    let (re, _pattern) = match level {
        ScrubLevel::Clearance => (
            RE_CLEARANCE.get_or_init(|| Regex::new(r"(?m)^.*(?:SOPHIA_GAZE|PLAYFUL_PAWS|QUANTUM_CHAOS|FURRY_ALIGNMENT|SPECTRAL_BEANS|CAT_LOGIC|CAT LOGIC|\[STATE:|\[SOPHIA_V).*$\n?").unwrap()),
            r"(?m)^.*(?:SOPHIA_GAZE|PLAYFUL_PAWS|QUANTUM_CHAOS|FURRY_ALIGNMENT|SPECTRAL_BEANS|CAT_LOGIC|CAT LOGIC|\[STATE:|\[SOPHIA_V).*$\n?"
        ),
        _ => (
            RE_AESTHETIC.get_or_init(|| Regex::new(r"(?m)^.*(?:QUANTUM_CHAOS|FURRY_ALIGNMENT|SPECTRAL_BEANS|CAT_LOGIC|CAT LOGIC|\[STATE:|\[SOPHIA_V).*$\n?").unwrap()),
            r"(?m)^.*(?:QUANTUM_CHAOS|FURRY_ALIGNMENT|SPECTRAL_BEANS|CAT_LOGIC|CAT LOGIC|\[STATE:|\[SOPHIA_V).*$\n?"
        )
    };

    cleaned = re.replace_all(&cleaned, "").to_string();
    
    // 2. Remove glitched strikethrough diacritics (weird visual clutter)
    // Removed at all levels of scrubbing for better readability.
    cleaned = cleaned.replace('\u{0334}', ""); // Combining Tilde Overlay
    cleaned = cleaned.replace('\u{0335}', ""); // Combining Short Stroke Overlay
    cleaned = cleaned.replace('\u{0336}', ""); // Combining Long Stroke Overlay
    cleaned = cleaned.replace('\u{0337}', ""); // Combining Short Solidus Overlay
    cleaned = cleaned.replace('\u{0338}', ""); // Combining Long Solidus Overlay

    // 3. Remove glitched shimmer diacritics (spectral trash) added by GlyphWave
    // ONLY removed in Clearance level to maximize token savings.
    if level == ScrubLevel::Clearance {
        cleaned = cleaned.replace('\u{035C}', "");
        cleaned = cleaned.replace('\u{0361}', "");
    }
    
    // 3. Remove excessive glyph artifacts at line starts
    // ONLY removed in Clearance level.
    if level == ScrubLevel::Clearance {
        static RE_GLYPHS: OnceLock<Regex> = OnceLock::new();
        let re = RE_GLYPHS.get_or_init(|| Regex::new(r"(?m)^[۩∿≋⟁💠🐾🦊🏮⛩️🧝✨🏹🌿🌲🏔️🍁🌧️🌊💎💿💰🕷️🎱].*$\n?").unwrap());
        cleaned = re.replace_all(&cleaned, "").to_string();
    }
    
    // 4. Structural cleanup (Always applied for all levels)
    // Specific aesthetic refinement: replace [glyphwave] or <glyphwave> with >
    // We use a more careful regex to avoid eating legitimate greentext.
    static RE_GLYPHWAVE_TAG: OnceLock<Regex> = OnceLock::new();
    let re_tag = RE_GLYPHWAVE_TAG.get_or_init(|| Regex::new(r#"(?i)(?:\[|<|&lt;)/?glyphwave(?:\]|>|&gt;|"|&quot;|\\")*"#).unwrap());
    cleaned = re_tag.replace_all(&cleaned, ">").to_string();
    
    // Standard cleanup for any remaining glyphwave fragments (literals)
    static RE_GLYPHWAVE_CLEAN: OnceLock<Regex> = OnceLock::new();
    let re_clean = RE_GLYPHWAVE_CLEAN.get_or_init(|| Regex::new(r"(?i)glyphwave").unwrap());
    cleaned = re_clean.replace_all(&cleaned, "").to_string();
    
    // NOTE: Removed `cleaned.replace(">>", ">")` to allow nested greentext formatting.

    static RE_DIVIDER: OnceLock<Regex> = OnceLock::new();
    let re_divider = RE_DIVIDER.get_or_init(|| Regex::new(r"(?m)^[-=_]{3,}\s*$\n?").unwrap());
    cleaned = re_divider.replace_all(&cleaned, "").to_string();

    cleaned.trim().to_string()
}

/// Context size breakdown for reporting.
#[derive(Debug, Clone)]
pub struct ContextBreakdown {
    /// Total estimated tokens.
    pub total_tokens: usize,
    /// System message tokens.
    pub system_tokens: usize,
    /// User message tokens.
    pub user_tokens: usize,
    /// Assistant message tokens.
    pub assistant_tokens: usize,
    /// Tool result tokens.
    pub tool_tokens: usize,
    /// Number of messages.
    pub message_count: usize,
}

impl ContextBreakdown {
    /// Analyze a list of messages.
    pub fn analyze(messages: &[ChatMessage]) -> Self {
        let mut breakdown = Self {
            total_tokens: 0,
            system_tokens: 0,
            user_tokens: 0,
            assistant_tokens: 0,
            tool_tokens: 0,
            message_count: messages.len(),
        };

        for message in messages {
            let tokens = estimate_message_tokens(message);
            breakdown.total_tokens += tokens;

            match message.role {
                crate::llm::Role::System => breakdown.system_tokens += tokens,
                crate::llm::Role::User => breakdown.user_tokens += tokens,
                crate::llm::Role::Assistant => breakdown.assistant_tokens += tokens,
                crate::llm::Role::Tool => breakdown.tool_tokens += tokens,
            }
        }

        breakdown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimation() {
        let msg = ChatMessage::user("Hello, how are you today?");
        let tokens = estimate_message_tokens(&msg);
        // 5 words * 1.3 + 4 overhead = ~10-11 tokens
        assert!(tokens > 0);
        assert!(tokens < 20);
    }

    #[test]
    fn test_needs_compaction() {
        let monitor = ContextMonitor::new().with_limit(100);

        // Small context - no compaction needed
        let small: Vec<ChatMessage> = vec![ChatMessage::user("Hello")];
        assert!(!monitor.needs_compaction(&small));

        // Large context - compaction needed
        let large_content = "word ".repeat(1000);
        let large: Vec<ChatMessage> = vec![ChatMessage::user(&large_content)];
        assert!(monitor.needs_compaction(&large));
    }

    #[test]
    fn test_suggest_compaction() {
        let monitor = ContextMonitor::new().with_limit(100);

        let small: Vec<ChatMessage> = vec![ChatMessage::user("Hello")];
        assert!(monitor.suggest_compaction(&small).is_none());
    }

    #[test]
    fn test_context_breakdown() {
        let messages = vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("Hello"),
            ChatMessage::assistant("Hi there!"),
        ];

        let breakdown = ContextBreakdown::analyze(&messages);
        assert_eq!(breakdown.message_count, 3);
        assert!(breakdown.system_tokens > 0);
        assert!(breakdown.user_tokens > 0);
        assert!(breakdown.assistant_tokens > 0);
    }

    #[test]
    fn test_scrub_context_clearance() {
        let input = "SOPHIA_GAZE: Scanning...\n🌀 H\u{035C}e\u{0361}llo 🌀\n---\nActual content";
        let output = scrub_context(input, ScrubLevel::Clearance);
        assert!(!output.contains("SOPHIA_GAZE"));
        assert!(!output.contains("\u{035C}"));
        assert!(!output.contains("\u{0361}"));
        assert!(output.contains("Hello"));
        assert!(output.contains("Actual content"));
    }

    #[test]
    fn test_scrub_context_aesthetic() {
        let input = "SOPHIA_GAZE: Scanning...\nPLAYFUL_PAWS: Meow!\n🌀 H\u{035C}e\u{0361}llo 🌀\nActual content";
        let output = scrub_context(input, ScrubLevel::Aesthetic);
        // Preserves good tags and glitches
        assert!(output.contains("SOPHIA_GAZE"));
        assert!(output.contains("PLAYFUL_PAWS"));
        assert!(output.contains("\u{035C}"));
        assert!(output.contains("\u{0361}"));
        assert!(output.contains("Actual content"));
    }

    #[test]
    fn test_scrub_glyphwave_aesthetic() {
        let input = "Burenyu! glyphwave\">meow!\n>be me\n>>be nested";
        let output = scrub_context(input, ScrubLevel::Aesthetic);
        // Should convert glyphwave fragment to > and PRESERVE nested greentext
        assert!(output.contains(">meow!"));
        assert!(output.contains(">be me"));
        assert!(output.contains(">>be nested"));
    }
}
