//! Routine management for scheduled and event-driven tasks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A trigger for a routine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trigger {
    /// Scheduled via cron expression.
    Cron { schedule: String },
    /// Triggered by a message pattern matching.
    Event {
        pattern: String,
        channel: Option<String>,
        user_id: Option<String>,
    },
    /// Triggered by a webhook.
    Webhook { path: Option<String> },
    /// Only triggered manually.
    Manual,
}

/// An action to be taken by a routine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutineAction {
    /// A lightweight agent call with a specific prompt.
    Lightweight { prompt: String },
    /// A full job creation.
    FullJob {
        title: String,
        description: String,
        category: Option<String>,
    },
}

/// A stored routine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Routine {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub trigger: Trigger,
    pub action: RoutineAction,
    pub guardrails: serde_json::Value,
    pub notify: serde_json::Value,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_fire_at: Option<DateTime<Utc>>,
    pub run_count: u64,
    pub consecutive_failures: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A record of a routine execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineRun {
    pub id: Uuid,
    pub routine_id: Uuid,
    pub trigger_type: String, // "cron", "event", "webhook", "manual"
    pub status: RoutineRunStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result_summary: Option<String>,
    pub tokens_used: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutineRunStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

impl std::fmt::Display for RoutineRunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}
