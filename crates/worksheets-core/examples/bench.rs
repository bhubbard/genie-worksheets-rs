//! Release benchmark for `worksheets-core` multi-turn slot filling and policy transitions.

use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use serde_json::Value;
use worksheets_core::{
    ActionHandler, FieldSpec, FieldType, GenieRuntime, RuleBasedParser, WorksheetError,
    WorksheetInstance, WorksheetSpec,
};

struct NoopHandler;
impl ActionHandler for NoopHandler {
    fn execute(
        &mut self,
        action_name: &str,
        _slots: &HashMap<String, Value>,
    ) -> Result<String, WorksheetError> {
        Ok(action_name.to_string())
    }
}

fn main() {
    let spec = WorksheetSpec {
        name: "intake".into(),
        action_name: "create_lead".into(),
        fields: vec![
            FieldSpec {
                name: "service".into(),
                description: "Service".into(),
                field_type: FieldType::Enum(vec!["Cleaning".into(), "Exam".into()]),
                required: true,
                requires_confirmation: false,
            },
            FieldSpec {
                name: "patients".into(),
                description: "Patients".into(),
                field_type: FieldType::Int,
                required: true,
                requires_confirmation: true,
            },
        ],
    };

    let mut times = Vec::with_capacity(15);
    for i in 0..18 {
        let t0 = Instant::now();
        for _ in 0..1000 {
            let mut rt = GenieRuntime::new(
                WorksheetInstance::new(spec.clone()),
                RuleBasedParser,
                NoopHandler,
            );
            let _ = black_box(rt.step("Cleaning"));
            let _ = black_box(rt.step("2"));
            let _ = black_box(rt.step("yes"));
        }
        if i >= 3 {
            times.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
    }
    times.sort_by(f64::total_cmp);
    println!(
        "GenieRuntime 3-turn dialogue (1000x): median={:.4} ms (min={:.4} ms)",
        times[times.len() / 2],
        times[0]
    );
}
