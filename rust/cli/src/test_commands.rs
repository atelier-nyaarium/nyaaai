//! Scripted test commands.

use nyaaai_adapter::runtime::{Inbound, RunContext, RunError, typed};
use nyaaai_contract::json::{Object, Value};

use super::cases::shapes::{TestkitConverseInput, TestkitConverseResult};

pub fn run(name: &str, input: &Value, context: &RunContext) -> Option<Result<Value, RunError>> {
    Some(match name {
        "testkit.converse" => typed(input, |input| testkit_converse(input, context)),
        // Avoids panic hook report.
        "testkit.crash" => std::panic::resume_unwind(Box::new("crash")),
        _ => return None,
    })
}

fn testkit_converse(
    input: TestkitConverseInput,
    context: &RunContext,
) -> Result<TestkitConverseResult, RunError> {
    let said = |key: &str, value: Value| {
        let mut event = Object::new();
        event.insert(key, value);
        context.emit(Value::Object(event));
    };
    let echo = |count: u64| {
        let mut asked = Object::new();
        asked.insert("count", Value::Number(count as f64));
        context.ask("echo", Value::Object(asked))
    };
    let together: Vec<String> = if input.together {
        (1..=input.requests).map(echo).collect()
    } else {
        Vec::new()
    };
    for id in &together {
        match context.reply(id) {
            Ok(value) => said("reply", value),
            Err(RunError::HostRequest(message)) => said("failed", Value::from(message)),
            Err(error) => return Err(error),
        }
    }
    for count in (1..=input.requests).filter(|_| !input.together) {
        let id = echo(count);
        loop {
            match context.receive() {
                Inbound::Interrupt => return Err(RunError::Interrupted),
                Inbound::Steer(text) => said("steer", Value::from(text)),
                Inbound::Reply { id: held, value } if held == id => {
                    said("reply", value);
                    break;
                }
                Inbound::ReplyFailed { id: held, message } if held == id => {
                    said("failed", Value::from(message));
                    break;
                }
                Inbound::Reply { .. } | Inbound::ReplyFailed { .. } => {}
            }
        }
    }
    Ok(TestkitConverseResult {
        ids: vec![context.ids.new_id(), context.ids.new_id()],
        now_ms: context.clock.now_ms(),
        steers: context.steers(),
    })
}
