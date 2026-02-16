use async_trait::async_trait;
use uuid::Uuid;
use crate::error::DatabaseError;
use crate::agent::routine::{Routine, RoutineRun};
use crate::history::{ConversationMessage, ConversationSummary, JobEventRecord, SandboxJobRecord, SandboxJobSummary, SettingRecord};

/// Database abstraction layer.
#[async_trait]
pub trait Database: Send + Sync {
    /// Persist a job-related event (e.g., status update, tool use).
    async fn save_job_event(
        &self,
        job_id: Uuid,
        event_type: &str,
        data: &serde_json::Value,
    ) -> Result<(), DatabaseError>;

    // --- Conversations ---

    async fn ensure_conversation(
        &self,
        id: Uuid,
        channel: &str,
        user_id: &str,
    ) -> Result<(), DatabaseError>;

    async fn get_or_create_assistant_conversation(
        &self,
        user_id: &str,
        channel: &str,
    ) -> Result<Uuid, DatabaseError>;

    async fn list_conversations_with_preview(
        &self,
        user_id: &str,
        channel: &str,
        limit: usize,
    ) -> Result<Vec<ConversationSummary>, DatabaseError>;

    async fn list_conversation_messages_paginated(
        &self,
        conversation_id: Uuid,
        limit: usize,
        before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<ConversationMessage>, bool), DatabaseError>;

    async fn conversation_belongs_to_user(
        &self,
        conversation_id: Uuid,
        user_id: &str,
    ) -> Result<bool, DatabaseError>;

    async fn update_conversation_metadata_field(
        &self,
        conversation_id: Uuid,
        field: &str,
        value: &serde_json::Value,
    ) -> Result<(), DatabaseError>;

    async fn update_conversation_title(
        &self,
        conversation_id: Uuid,
        title: &str,
    ) -> Result<(), DatabaseError>;

    async fn delete_conversation(
        &self,
        id: Uuid,
        user_id: &str,
    ) -> Result<bool, DatabaseError>;

    // --- Sandbox Jobs ---

    async fn save_sandbox_job(&self, job: &SandboxJobRecord) -> Result<(), DatabaseError>;

    async fn get_sandbox_job(&self, id: Uuid) -> Result<Option<SandboxJobRecord>, DatabaseError>;

    async fn get_sandbox_job_mode(&self, id: Uuid) -> Result<Option<String>, DatabaseError>;

    async fn list_sandbox_jobs_for_user(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<SandboxJobRecord>, DatabaseError>;

    async fn update_sandbox_job_status(
        &self,
        id: Uuid,
        status: &str,
        success: Option<bool>,
        failure_reason: Option<String>,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), DatabaseError>;

    async fn sandbox_job_belongs_to_user(&self, id: Uuid, user_id: &str)
        -> Result<bool, DatabaseError>;

    async fn sandbox_job_summary_for_user(
        &self,
        user_id: &str,
    ) -> Result<SandboxJobSummary, DatabaseError>;

    async fn list_job_events(
        &self,
        job_id: Uuid,
    ) -> Result<Vec<JobEventRecord>, DatabaseError>;

    // --- Routines ---

    async fn list_routines(&self, user_id: &str) -> Result<Vec<Routine>, DatabaseError>;

    async fn get_routine(&self, id: Uuid) -> Result<Option<Routine>, DatabaseError>;

    async fn update_routine(&self, routine: &Routine) -> Result<(), DatabaseError>;

    async fn delete_routine(&self, id: Uuid) -> Result<bool, DatabaseError>;

    async fn list_routine_runs(&self, routine_id: Uuid, limit: usize) -> Result<Vec<RoutineRun>, DatabaseError>;

    // --- Settings ---

    async fn list_settings(&self, user_id: &str) -> Result<Vec<SettingRecord>, DatabaseError>;

    async fn get_setting_full(&self, user_id: &str, key: &str) -> Result<Option<SettingRecord>, DatabaseError>;

    async fn set_setting(&self, user_id: &str, key: &str, value: &serde_json::Value) -> Result<(), DatabaseError>;

    async fn delete_setting(&self, user_id: &str, key: &str) -> Result<(), DatabaseError>;

    async fn get_all_settings(&self, user_id: &str) -> Result<std::collections::HashMap<String, serde_json::Value>, DatabaseError>;

    async fn set_all_settings(&self, user_id: &str, settings: &std::collections::HashMap<String, serde_json::Value>) -> Result<(), DatabaseError>;
}
