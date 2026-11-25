//! Stateful Genie Worksheets conversation runtime.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::dialogue::{AgentAct, AgentPolicy, SemanticParser, SlotUpdate};
use crate::schema::{FieldState, WorksheetInstance};
use crate::WorksheetError;

/// Callback interface executed when a worksheet's required slots are confirmed.
pub trait ActionHandler {
    /// Executes `action_name` with the confirmed slot values.
    ///
    /// # Errors
    /// Returns [`WorksheetError::Action`] if execution fails.
    fn execute(
        &mut self,
        action_name: &str,
        slots: &HashMap<String, Value>,
    ) -> Result<String, WorksheetError>;
}

/// Output produced by one turn of [`GenieRuntime::step`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnOutcome {
    /// Chosen dialogue act.
    pub act: AgentAct,
    /// Rendered assistant message.
    pub response: String,
    /// Optional action execution result if [`AgentAct::RunAction`] fired.
    pub action_output: Option<String>,
}

/// Multi-turn conversational runtime driving a [`WorksheetInstance`].
pub struct GenieRuntime<P, A> {
    /// Active worksheet state.
    pub worksheet: WorksheetInstance,
    parser: P,
    actions: A,
    pending_confirm: Option<String>,
}

impl<P: SemanticParser, A: ActionHandler> GenieRuntime<P, A> {
    /// Creates a new [`GenieRuntime`] for `worksheet`.
    #[must_use]
    pub fn new(worksheet: WorksheetInstance, parser: P, actions: A) -> Self {
        Self {
            worksheet,
            parser,
            actions,
            pending_confirm: None,
        }
    }

    /// Processes a user utterance, updates worksheet state, and returns the next
    /// [`TurnOutcome`].
    ///
    /// # Errors
    /// Returns [`WorksheetError`] if parsing, validation, or action execution fails.
    pub fn step(&mut self, user_utterance: &str) -> Result<TurnOutcome, WorksheetError> {
        let updates = self.parser.parse(
            user_utterance,
            &self.worksheet,
            self.pending_confirm.as_deref(),
        )?;

        for upd in updates {
            match upd {
                SlotUpdate::Set { field, value } => self.worksheet.set_field(&field, value)?,
                SlotUpdate::Confirm { field } => {
                    self.worksheet.confirm_field(&field)?;
                    if self.pending_confirm.as_deref() == Some(field.as_str()) {
                        self.pending_confirm = None;
                    }
                }
            }
        }

        let act = AgentPolicy::next_act(&self.worksheet);
        match &act {
            AgentAct::ConfirmField { field, value } => {
                self.pending_confirm = Some(field.clone());
                Ok(TurnOutcome {
                    act: act.clone(),
                    response: format!("Please confirm {field} = {value}?"),
                    action_output: None,
                })
            }
            AgentAct::AskField { prompt, .. } => Ok(TurnOutcome {
                act: act.clone(),
                response: prompt.clone(),
                action_output: None,
            }),
            AgentAct::RunAction { action_name } => {
                let mut slots = HashMap::new();
                for (k, st) in &self.worksheet.states {
                    if let FieldState::Confirmed(v) = st {
                        slots.insert(k.clone(), v.clone());
                    }
                }
                let output = self.actions.execute(action_name, &slots)?;
                self.worksheet.completed = true;
                Ok(TurnOutcome {
                    act: act.clone(),
                    response: output.clone(),
                    action_output: Some(output),
                })
            }
            AgentAct::Done => Ok(TurnOutcome {
                act,
                response: "Worksheet already completed.".into(),
                action_output: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue::RuleBasedParser;
    use crate::schema::{FieldSpec, FieldType, WorksheetSpec};

    struct RecordingHandler;
    impl ActionHandler for RecordingHandler {
        fn execute(
            &mut self,
            action_name: &str,
            slots: &HashMap<String, Value>,
        ) -> Result<String, WorksheetError> {
            Ok(format!(
                "Executed {action_name} for {}",
                slots.get("service").and_then(Value::as_str).unwrap_or("?")
            ))
        }
    }

    #[test]
    fn multi_turn_slot_filling_with_confirmation_executes_action() {
        let spec = WorksheetSpec {
            name: "dental_booking".into(),
            action_name: "book_appointment".into(),
            fields: vec![
                FieldSpec {
                    name: "service".into(),
                    description: "Requested service".into(),
                    field_type: FieldType::Enum(vec!["Cleaning".into(), "Exam".into()]),
                    required: true,
                    requires_confirmation: false,
                },
                FieldSpec {
                    name: "patients".into(),
                    description: "Number of patients".into(),
                    field_type: FieldType::Int,
                    required: true,
                    requires_confirmation: true,
                },
            ],
        };
        let mut rt = GenieRuntime::new(
            WorksheetInstance::new(spec),
            RuleBasedParser,
            RecordingHandler,
        );

        let t1 = rt.step("Cleaning").expect("turn 1");
        assert!(matches!(t1.act, AgentAct::AskField { ref field, .. } if field == "patients"));

        let t2 = rt.step("2").expect("turn 2");
        assert!(matches!(t2.act, AgentAct::ConfirmField { ref field, .. } if field == "patients"));

        let t3 = rt.step("yes").expect("turn 3");
        assert!(matches!(t3.act, AgentAct::RunAction { .. }));
        assert_eq!(
            t3.action_output.as_deref(),
            Some("Executed book_appointment for Cleaning")
        );
    }
}
