//! Scripted test commands.

use std::thread;
use std::time::Duration;

use nyaaai_adapter::runtime::{
    Command, Inbound, RunContext, RunError, blocking, typed, typed_async,
};
use nyaaai_contract::json::{Object, Value};

use super::cases::shapes::{
    TestkitConverseInput, TestkitConverseResult, TestkitMisfitInput, TestkitMisfitResult,
};

/// How long blocking work waits between looks.
const LOOK: Duration = Duration::from_millis(1);

pub fn run(name: &str, input: Value, context: RunContext) -> Option<Command> {
    Some(match name {
        "testkit.converse" => {
            Box::pin(
                async move { typed_async(&input, |input| testkit_converse(input, context)).await },
            )
        }
        "testkit.misfit" => Box::pin(async move { typed(&input, testkit_misfit) }),
        // Avoids panic hook report.
        "testkit.crash" => std::panic::resume_unwind(Box::new("crash")),
        "testkit.crashBlocking" => {
            blocking(&context, |_| std::panic::resume_unwind(Box::new("crash")))
        }
        "testkit.spin" => Box::pin(async move {
            working(&context);
            context.token().cancelled().await;
            Ok(saw_cancel())
        }),
        "testkit.spinBlocking" => {
            working(&context);
            blocking(&context, |blocked| {
                while !blocked.is_cancelled() {
                    thread::sleep(LOOK);
                }
                Ok(saw_cancel())
            })
        }
        _ => return None,
    })
}

/// Tells the host the work began.
fn working(context: &RunContext) {
    let mut event = Object::new();
    event.insert("working", Value::Bool(true));
    context.emit(Value::Object(event));
}

fn saw_cancel() -> Value {
    let mut result = Object::new();
    result.insert("saw", Value::from("cancelled"));
    Value::Object(result)
}

fn testkit_misfit(input: TestkitMisfitInput) -> Result<TestkitMisfitResult, RunError> {
    let text = if input.fits { "fits" } else { "" };
    Ok(TestkitMisfitResult {
        text: text.to_owned(),
    })
}

async fn testkit_converse(
    input: TestkitConverseInput,
    context: RunContext,
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
        match context.reply(id).await {
            Ok(value) => said("reply", value),
            Err(RunError::HostRequest(message)) => said("failed", Value::from(message)),
            Err(error) => return Err(error),
        }
    }
    for count in (1..=input.requests).filter(|_| !input.together) {
        let id = echo(count);
        loop {
            match context.receive().await {
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
