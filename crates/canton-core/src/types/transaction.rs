//! Transaction and TransactionTree types.
//! See research/08, 04.

use crate::types::event::{
    ArchivedEvent, CreatedEvent, Event, ExercisedEvent,
};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub transaction_id: String,
    pub command_id: String,
    pub workflow_id: String,
    pub effective_at: chrono::DateTime<chrono::Utc>,
    pub events: Vec<Event>,
    pub offset: String,
}

#[derive(Debug, Clone)]
pub struct TransactionTree {
    pub transaction_id: String,
    pub command_id: String,
    pub workflow_id: String,
    pub effective_at: chrono::DateTime<chrono::Utc>,
    pub events_by_id: std::collections::HashMap<String, TreeEvent>,
    pub root_event_ids: Vec<String>,
    pub offset: String,
}

#[derive(Debug, Clone)]
pub enum TreeEvent {
    Created(CreatedEvent),
    Archived(ArchivedEvent),
    Exercised(ExercisedEvent),
}
