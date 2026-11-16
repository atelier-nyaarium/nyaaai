//! Case handler context.

use std::cell::OnceCell;
use std::fs;
use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use nyaaai_contract::adapter_texts::MISFIT;
use nyaaai_contract::id::CountedIds;
use nyaaai_contract::json::{Object, Value};
use nyaaai_contract::shape::Shape;
use nyaaai_contract::time::SteppedClock;

/// Matches TypeScript's clock.
pub const CASE_NOW_MS: i64 = 1_794_535_200_000;
pub const CASE_STEP_MS: i64 = 1000;

/// Scripted ids: `case-1`, `case-2`.
pub const CASE_ID_PREFIX: &str = "case-";

/// Result path placeholders.
pub const TMP_MARK: &str = "$TMP";
pub const ROOT_MARK: &str = "$ROOT";

pub type Outcome<T> = Result<T, String>;

/// The result as its shape decodes it.
fn held_to<Output: Shape>(made: Value) -> Outcome<Value> {
    Output::decode(&made)
        .map(|held| held.encode())
        .map_err(|_| MISFIT.to_owned())
}

static MADE: AtomicU64 = AtomicU64::new(0);

/// Shared case resources.
pub struct CaseContext {
    root: PathBuf,
    tmp: OnceCell<Outcome<PathBuf>>,
    pub clock: SteppedClock,
    pub ids: CountedIds,
}

impl CaseContext {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_owned(),
            tmp: OnceCell::new(),
            clock: SteppedClock::new(CASE_NOW_MS, CASE_STEP_MS),
            ids: CountedIds::new(CASE_ID_PREFIX),
        }
    }

    /// Read-only project root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Per-case temporary folder.
    pub fn tmp(&self) -> Outcome<&Path> {
        let made = self.tmp.get_or_init(|| {
            let made = loop {
                let name = format!(
                    "nyaaai-case-{}-{}",
                    std::process::id(),
                    MADE.fetch_add(1, Ordering::Relaxed)
                );
                let folder = std::env::temp_dir().join(name);
                match fs::create_dir(&folder) {
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                    other => break other.map(|()| folder),
                }
            };
            made.and_then(|folder| fs::canonicalize(&folder))
                .map_err(failed)
        });
        made.as_deref().map_err(Clone::clone)
    }

    /// Replaces paths in results.
    fn marked(&self, value: Value) -> Value {
        let tmp = self.tmp.get().and_then(|made| made.as_ref().ok());
        let tmp = tmp.map(|folder| folder.to_string_lossy().into_owned());
        let root = self.root.to_string_lossy();
        replaced(value, &|text| {
            let text = match &tmp {
                Some(folder) => text.replace(folder.as_str(), TMP_MARK),
                None => text.to_owned(),
            };
            if root.is_empty() {
                text
            } else {
                text.replace(&*root, ROOT_MARK)
            }
        })
    }
}

impl Drop for CaseContext {
    fn drop(&mut self) {
        if let Some(Ok(folder)) = self.tmp.get() {
            let _ = fs::remove_dir_all(folder);
        }
    }
}

fn replaced(value: Value, change: &dyn Fn(&str) -> String) -> Value {
    match value {
        Value::String(text) => Value::String(change(&text)),
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|item| replaced(item, change))
                .collect(),
        ),
        Value::Object(held) => {
            let mut made = Object::new();
            for (key, item) in held.iter() {
                made.insert(key, replaced(item.clone(), change));
            }
            Value::Object(made)
        }
        other => other,
    }
}

pub fn failed(error: io::Error) -> String {
    error.to_string()
}

/// Runs shaped case handlers, and holds results to their shape.
pub fn run<Input: Shape, Output: Shape>(
    input: &Value,
    handler: fn(Input) -> Outcome<Output>,
) -> Outcome<Value> {
    let input = Input::decode(input).map_err(|refused| format!("the input: {refused}"))?;
    held_to::<Output>(handler(input)?.encode())
}

/// Runs a future to its end on a current-thread runtime.
pub fn block_on<F: Future>(future: F) -> Outcome<F::Output> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(failed)?;
    Ok(runtime.block_on(future))
}

/// Adds a per-case context.
pub fn run_with<Input: Shape, Output: Shape>(
    input: &Value,
    root: &Path,
    handler: fn(Input, &CaseContext) -> Outcome<Output>,
) -> Outcome<Value> {
    let input = Input::decode(input).map_err(|refused| format!("the input: {refused}"))?;
    let context = CaseContext::new(root);
    let result = handler(input, &context)?;
    held_to::<Output>(context.marked(result.encode()))
}

#[cfg(test)]
mod tests {
    use nyaaai_contract::json::parse_json;

    use super::*;
    use crate::torture_shapes::Bounded;

    const BOUNDED: &str = r#"{"name":"ab","ratio":0.5,"few":[true],"level":0,"count":0}"#;

    fn blank(mut held: Bounded) -> Outcome<Bounded> {
        held.few.clear();
        Ok(held)
    }

    fn same(held: Bounded) -> Outcome<Bounded> {
        Ok(held)
    }

    #[test]
    fn holds_a_result_to_its_shape() {
        let input = parse_json(BOUNDED).expect("json");
        assert_eq!(
            run::<Bounded, Bounded>(&input, blank),
            Err(MISFIT.to_owned())
        );
        let fits = run::<Bounded, Bounded>(&input, same).expect("fits");
        assert_eq!(fits, input);
        let root = std::env::temp_dir();
        assert_eq!(
            run_with::<Bounded, Bounded>(&input, &root, |held, _| blank(held)),
            Err(MISFIT.to_owned())
        );
    }
}
