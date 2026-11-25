//! Declarative worksheet schema, typed fields, and runtime slot state.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::WorksheetError;

/// Supported slot value types in a Genie Worksheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    /// Free-form text.
    Text,
    /// Signed 64-bit integer.
    Int,
    /// Boolean flag.
    Bool,
    /// Closed set of allowed string values.
    Enum(Vec<String>),
}

/// Specification for a single worksheet field/slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldSpec {
    /// Unique field identifier.
    pub name: String,
    /// Human-readable description shown to users/LLMs.
    pub description: String,
    /// Expected value type.
    pub field_type: FieldType,
    /// Whether this field must be filled before executing the worksheet action.
    pub required: bool,
    /// Whether the agent must explicitly ask the user to confirm this value.
    pub requires_confirmation: bool,
}

/// Declarative definition of a conversational worksheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorksheetSpec {
    /// Worksheet identifier (e.g. `"appointment_booking"`).
    pub name: String,
    /// Ordered list of fields.
    pub fields: Vec<FieldSpec>,
    /// Name of the backend action triggered once all required fields are confirmed.
    pub action_name: String,
}

/// Lifecycle state of a single field in a [`WorksheetInstance`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldState {
    /// Not yet provided.
    Empty,
    /// Provided by the user, awaiting explicit confirmation if required.
    Proposed(Value),
    /// Validated and confirmed.
    Confirmed(Value),
}

/// Live stateful instance of a [`WorksheetSpec`] during a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetInstance {
    /// Underlying schema.
    pub spec: WorksheetSpec,
    /// Current state per field name.
    pub states: HashMap<String, FieldState>,
    /// Whether the worksheet action has been executed.
    pub completed: bool,
}

impl WorksheetInstance {
    /// Instantiates a fresh worksheet with all fields [`FieldState::Empty`].
    #[must_use]
    pub fn new(spec: WorksheetSpec) -> Self {
        let states = spec
            .fields
            .iter()
            .map(|f| (f.name.clone(), FieldState::Empty))
            .collect();
        Self {
            spec,
            states,
            completed: false,
        }
    }

    /// Validates and sets `field_name` to `raw_value`, transitioning it to
    /// [`FieldState::Proposed`] or [`FieldState::Confirmed`].
    ///
    /// # Errors
    /// Returns [`WorksheetError::Unknown`] if `field_name` is not in the schema,
    /// or [`WorksheetError::Validation`] if type/enum checks fail.
    pub fn set_field(&mut self, field_name: &str, raw_value: Value) -> Result<(), WorksheetError> {
        let spec = self
            .spec
            .fields
            .iter()
            .find(|f| f.name == field_name)
            .ok_or_else(|| WorksheetError::Unknown(field_name.to_string()))?
            .clone();

        validate_value(&spec, &raw_value)?;
        let next_state = if spec.requires_confirmation {
            FieldState::Proposed(raw_value)
        } else {
            FieldState::Confirmed(raw_value)
        };
        self.states.insert(spec.name, next_state);
        Ok(())
    }

    /// Confirms a currently [`FieldState::Proposed`] field.
    ///
    /// # Errors
    /// Returns [`WorksheetError::Unknown`] if `field_name` is missing.
    pub fn confirm_field(&mut self, field_name: &str) -> Result<(), WorksheetError> {
        let state = self
            .states
            .get_mut(field_name)
            .ok_or_else(|| WorksheetError::Unknown(field_name.to_string()))?;
        if let FieldState::Proposed(v) = state.clone() {
            *state = FieldState::Confirmed(v);
        }
        Ok(())
    }

    /// Returns the first required field that is still [`FieldState::Empty`].
    #[must_use]
    pub fn next_missing_required(&self) -> Option<&FieldSpec> {
        self.spec.fields.iter().find(|f| {
            f.required && matches!(self.states.get(&f.name), None | Some(FieldState::Empty))
        })
    }

    /// Returns the first field currently in [`FieldState::Proposed`].
    #[must_use]
    pub fn next_unconfirmed(&self) -> Option<(&FieldSpec, &Value)> {
        for f in &self.spec.fields {
            if let Some(FieldState::Proposed(v)) = self.states.get(&f.name) {
                return Some((f, v));
            }
        }
        None
    }

    /// Returns `true` when all required fields are [`FieldState::Confirmed`].
    #[must_use]
    pub fn is_ready_for_action(&self) -> bool {
        self.spec.fields.iter().all(|f| {
            !f.required || matches!(self.states.get(&f.name), Some(FieldState::Confirmed(_)))
        })
    }
}

fn validate_value(spec: &FieldSpec, val: &Value) -> Result<(), WorksheetError> {
    match &spec.field_type {
        FieldType::Text => {
            if !val.is_string() {
                return Err(WorksheetError::Validation {
                    field: spec.name.clone(),
                    reason: "expected string".into(),
                });
            }
        }
        FieldType::Int => {
            if !val.is_i64() {
                return Err(WorksheetError::Validation {
                    field: spec.name.clone(),
                    reason: "expected integer".into(),
                });
            }
        }
        FieldType::Bool => {
            if !val.is_boolean() {
                return Err(WorksheetError::Validation {
                    field: spec.name.clone(),
                    reason: "expected boolean".into(),
                });
            }
        }
        FieldType::Enum(choices) => {
            let Some(s) = val.as_str() else {
                return Err(WorksheetError::Validation {
                    field: spec.name.clone(),
                    reason: "expected enum string".into(),
                });
            };
            if !choices.iter().any(|c| c.eq_ignore_ascii_case(s)) {
                return Err(WorksheetError::Validation {
                    field: spec.name.clone(),
                    reason: format!("`{s}` not in allowed choices {choices:?}"),
                });
            }
        }
    }
    Ok(())
}
