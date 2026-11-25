//! CLI for Genie Worksheets conversational state machine.

use std::collections::HashMap;

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::Value;
use worksheets_core::{
    ActionHandler, FieldSpec, FieldType, GenieRuntime, RuleBasedParser, WorksheetError,
    WorksheetInstance, WorksheetSpec,
};

#[derive(Parser)]
#[command(name = "genie-worksheets", about = "Pure-Rust Genie Worksheets CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a multi-turn worksheet slot-filling & confirmation demo.
    Demo,
}

struct PrintHandler;
impl ActionHandler for PrintHandler {
    fn execute(
        &mut self,
        action_name: &str,
        slots: &HashMap<String, Value>,
    ) -> Result<String, WorksheetError> {
        Ok(format!(
            "Action `{action_name}` succeeded with slots: {}",
            serde_json::to_string(slots).unwrap_or_default()
        ))
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Demo => {
            let spec = WorksheetSpec {
                name: "intake_triage".into(),
                action_name: "create_lead".into(),
                fields: vec![
                    FieldSpec {
                        name: "case_type".into(),
                        description: "Practice area".into(),
                        field_type: FieldType::Enum(vec!["AutoAccident".into(), "Premises".into()]),
                        required: true,
                        requires_confirmation: false,
                    },
                    FieldSpec {
                        name: "injury_count".into(),
                        description: "Number of injured parties".into(),
                        field_type: FieldType::Int,
                        required: true,
                        requires_confirmation: true,
                    },
                ],
            };
            let mut rt =
                GenieRuntime::new(WorksheetInstance::new(spec), RuleBasedParser, PrintHandler);
            for input in ["AutoAccident", "1", "yes"] {
                let out = rt.step(input)?;
                println!("User: {input} -> Agent: {}", out.response);
            }
        }
    }
    Ok(())
}
