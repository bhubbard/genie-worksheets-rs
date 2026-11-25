//! Semantic parsing of user utterances and deterministic Genie agent policy.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::schema::{FieldType, WorksheetInstance};
use crate::WorksheetError;

/// Structured slot update or confirmation extracted from a user turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotUpdate {
    /// Set `field` to `value`.
    Set {
        /// Target field name.
        field: String,
        /// Parsed JSON value.
        value: Value,
    },
    /// Confirm the pending proposed value for `field`.
    Confirm {
        /// Target field name.
        field: String,
    },
}

/// Next dialogue act chosen by [`AgentPolicy`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentAct {
    /// Ask the user to confirm a proposed field value.
    ConfirmField {
        /// Field name.
        field: String,
        /// Proposed value.
        value: Value,
    },
    /// Prompt the user for a missing required field.
    AskField {
        /// Field name.
        field: String,
        /// Field description.
        prompt: String,
    },
    /// Execute the worksheet backend action.
    RunAction {
        /// Action identifier.
        action_name: String,
    },
    /// Worksheet already completed.
    Done,
}

/// Parses user utterances into [`SlotUpdate`]s against a [`WorksheetInstance`].
pub trait SemanticParser {
    /// Extracts slot updates from `utterance`.
    ///
    /// # Errors
    /// Returns [`WorksheetError`] if parsing fails.
    fn parse(
        &self,
        utterance: &str,
        ws: &WorksheetInstance,
        pending_confirm: Option<&str>,
    ) -> Result<Vec<SlotUpdate>, WorksheetError>;
}

/// Deterministic rule-based [`SemanticParser`] supporting `key=value` pairs,
/// confirmation replies (`yes`/`confirm`), and single-slot direct answers.
#[derive(Debug, Default)]
pub struct RuleBasedParser;

impl SemanticParser for RuleBasedParser {
    fn parse(
        &self,
        utterance: &str,
        ws: &WorksheetInstance,
        pending_confirm: Option<&str>,
    ) -> Result<Vec<SlotUpdate>, WorksheetError> {
        let trimmed = utterance.trim();
        let mut updates = Vec::new();

        if let Some(field) = pending_confirm {
            if matches!(
                trimmed.to_ascii_lowercase().as_str(),
                "yes" | "y" | "confirm" | "ok" | "sure"
            ) {
                updates.push(SlotUpdate::Confirm {
                    field: field.to_string(),
                });
                return Ok(updates);
            }
        }

        // Parse explicit `field=value` tokens
        for token in trimmed.split_whitespace() {
            if let Some((k, v)) = token.split_once('=') {
                if let Some(spec) = ws.spec.fields.iter().find(|f| f.name == k) {
                    updates.push(SlotUpdate::Set {
                        field: k.to_string(),
                        value: coerce_value(&spec.field_type, v)?,
                    });
                }
            }
        }
        if !updates.is_empty() {
            return Ok(updates);
        }

        // Fallback: bind directly to the next missing required field
        if let Some(missing) = ws.next_missing_required() {
            updates.push(SlotUpdate::Set {
                field: missing.name.clone(),
                value: coerce_value(&missing.field_type, trimmed)?,
            });
        }

        Ok(updates)
    }
}

fn coerce_value(ft: &FieldType, raw: &str) -> Result<Value, WorksheetError> {
    match ft {
        FieldType::Text | FieldType::Enum(_) => Ok(Value::String(raw.to_string())),
        FieldType::Int => {
            raw.parse::<i64>()
                .map(Value::from)
                .map_err(|_| WorksheetError::Validation {
                    field: "int".into(),
                    reason: format!("cannot parse `{raw}` as integer"),
                })
        }
        FieldType::Bool => match raw.to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(Value::Bool(true)),
            "false" | "no" | "0" => Ok(Value::Bool(false)),
            _ => Err(WorksheetError::Validation {
                field: "bool".into(),
                reason: format!("cannot parse `{raw}` as boolean"),
            }),
        },
    }
}

/// Deterministic Genie Worksheets agent policy selecting the next [`AgentAct`].
#[derive(Debug, Default)]
pub struct AgentPolicy;

impl AgentPolicy {
    /// Selects the next [`AgentAct`] given the current [`WorksheetInstance`] state.
    #[must_use]
    pub fn next_act(ws: &WorksheetInstance) -> AgentAct {
        if ws.completed {
            return AgentAct::Done;
        }
        if let Some((spec, val)) = ws.next_unconfirmed() {
            return AgentAct::ConfirmField {
                field: spec.name.clone(),
                value: val.clone(),
            };
        }
        if let Some(missing) = ws.next_missing_required() {
            return AgentAct::AskField {
                field: missing.name.clone(),
                prompt: format!("Please provide {}: {}", missing.name, missing.description),
            };
        }
        if ws.is_ready_for_action() {
            return AgentAct::RunAction {
                action_name: ws.spec.action_name.clone(),
            };
        }
        AgentAct::Done
    }
}
