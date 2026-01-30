//! Command types for Ledger API.
//! See research/08, 04.

use crate::types::identifier::Identifier;
use crate::types::value::{DamlRecord, DamlValue};

/// Commands envelope for Ledger API submission.
#[derive(Debug, Clone)]
pub struct Commands {
    pub ledger_id: Option<String>,
    pub workflow_id: String,
    pub application_id: String,
    pub command_id: String,
    pub act_as: Vec<String>,
    pub read_as: Vec<String>,
    pub commands: Vec<Command>,
    pub min_ledger_time_abs: Option<chrono::DateTime<chrono::Utc>>,
    pub min_ledger_time_rel: Option<std::time::Duration>,
    pub deduplication_period: Option<std::time::Duration>,
    pub submission_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Command {
    Create(CreateCommand),
    Exercise(ExerciseCommand),
    ExerciseByKey(ExerciseByKeyCommand),
    CreateAndExercise(CreateAndExerciseCommand),
}

#[derive(Debug, Clone)]
pub struct CreateCommand {
    pub template_id: Identifier,
    pub create_arguments: DamlRecord,
}

#[derive(Debug, Clone)]
pub struct ExerciseCommand {
    pub template_id: Identifier,
    pub contract_id: String,
    pub choice: String,
    pub choice_argument: DamlValue,
}

#[derive(Debug, Clone)]
pub struct ExerciseByKeyCommand {
    pub template_id: Identifier,
    pub contract_key: DamlValue,
    pub choice: String,
    pub choice_argument: DamlValue,
}

#[derive(Debug, Clone)]
pub struct CreateAndExerciseCommand {
    pub template_id: Identifier,
    pub create_arguments: DamlRecord,
    pub choice: String,
    pub choice_argument: DamlValue,
}
