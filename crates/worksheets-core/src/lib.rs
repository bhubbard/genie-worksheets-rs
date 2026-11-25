//! Pure-Rust port of Stanford OVAL's Genie Worksheets.
//!
//! Provides declarative [`schema::WorksheetSpec`] definitions, typed slot
//! validation, deterministic [`dialogue::AgentPolicy`] transitions, and
//! [`runtime::GenieRuntime`] for multi-turn task-oriented dialogue.

pub mod dialogue;
pub mod error;
pub mod runtime;
pub mod schema;

pub use dialogue::{AgentAct, AgentPolicy, RuleBasedParser, SemanticParser, SlotUpdate};
pub use error::WorksheetError;
pub use runtime::{ActionHandler, GenieRuntime, TurnOutcome};
pub use schema::{FieldSpec, FieldState, FieldType, WorksheetInstance, WorksheetSpec};
