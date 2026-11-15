# NyaaAI

Shared AI framework for Nyaarium tools. TypeScript and Rust twins, judged by one set of cases.

## Layout

- `ts/`: Bun workspace. Packages `contract`, `process`, `agentic`, `operator`, `ui`, `testkit`, `cli`, scope `@nyaaai`.
- `rust/`: Cargo workspace. Crates `contract`, `process`, `agentic`, `operator`, `testkit`, `cli`.
- `prompts/`: prompt texts, one file each, by package. Both languages get a generated module.
- `schema/`: JSON Schema exported from the Zod contract, input and output form per set. Generated, committed.
- `cases/`: language-neutral cases. Generated from `scripts/caseSources/`, committed.
  - A case is data. `paths` cases also name a folder tree, built in a temp folder by each language.
  - `cases/traces/`: recorded exchanges in byte form, read by cases.
- `scripts/`: gate, contract generator, case runner, crate check, snapshot, consumer script.

## Modules

Same names in both languages; Rust files are snake case.

- `contract`: primitives, ports and shapes. Rust adds `js`.
  - `json`: `parseJson`, `stringifyJson`, `canonicalJson`, `jsString` (Rust: `js::to_js_string`).
  - `text`: UTF-16 lengths, pair-safe cuts, finds. `order`: `compareText`, `sortedTexts`. `base64`.
  - `number`: `numberText`, `roundHalfUp`, `fixedText`, `numberOfText`, `integerOfText`.
  - `time`: `parseIsoTime`, `isoTime`, `parseHttpDate`, `parseRetryAfter`, `resetStampMs`, `dayKey`. Ports `Clock`, `Scheduler` (TypeScript only); scripted `steppedClock`.
  - `id`: `randomUuid`. Port `Ids`; scripted `countedIds`. `random`: port `Random`; scripted `sequenceRandom`.
  - `pattern`: `Pattern`, one regex dialect for both languages.
  - `shape`: shape sets, hooks, `decode`, `decodeText`, `CONTRACT_VERSION`. Rust's `shape` is the runtime generated code calls; `version` holds the constant.
  - `schemaExport`: Zod to JSON Schema, with the dialect check.
  - `strictSchema`: the JSON Schema subset: `normalizeSchema`, `schemaProblems`, `validate`. `issues`: its messages; Rust's is generated.
  - `shapes/frames`: stdio adapter frames. `shapes/sessionCut`: the cut's data shapes. Rust modules are generated.
- `process`: `paths` (`resolvedPath`, `isInside`).
- `operator`: `claude/sessionCut` cuts a Claude Code session file. `claude/sessionFile` reads its rows and holds cut ports over files. `claude/cliVersion` reads the installed version. `prompts` is generated.
- `cli`: `adapter` serves commands over stdio. `commands` lists them. Rust adds the `cut` command.
- `testkit`: `claudeSession` builds synthetic session files. `tortureShapes` holds every contract construct, for the generator. `caseKinds` and `caseFile` shape the cases. `trace` reads a recorded trace and replays it. `scheduler` holds a timer wheel on virtual time.
- `agentic`, `ui`: empty.

## Patterns

- Flags: `i`, `m`, `s`; literals, classes, groups, alternation, quantifiers.
- `\d`, `\w`, `\b`: ASCII; `\s`: ECMAScript whitespace. `.` and `m` anchors: LF ends a line.
- Matching uses Unicode scalars; offsets use UTF-16 units.
- Refused, since the two engines match them differently: a repeat whose item can match nothing; a capture a repeat round can skip.
- Limits: groups nest 32 deep; cost 10,000, counting atoms once counted repeats are unrolled, a class by its ranges.
- `split` follows ECMAScript. A capture that took no part is empty text.
- TypeScript backtracks; Rust does not. Nested unbounded repeats can stall TypeScript.
- Engine code compiles patterns through `Pattern`.

## Commands

Run from the repository root.

```sh
bun scripts/gate.ts               # everything below
bun scripts/gate.ts cases tsc     # `link`, then only the named parts
bun scripts/contract.ts           # schema/ and generated Rust current, shapes inside the dialect
bun scripts/contract.ts --refresh # rewrite them from Zod
bun scripts/cases.ts              # case files current, both adapters agree
bun scripts/cases.ts --refresh    # rewrite case files from TypeScript
bun scripts/serve.ts              # the TypeScript adapter, with the case kinds
bun scripts/convertTrace.ts <trace.jsonl> <out.json>   # an old recorded trace into the byte form, scrubbed
bun scripts/crates.ts             # crate ages and advisories
bun scripts/snapshot.ts           # commit the working tree for a consumer, print {ref, sha, subject}
bun scripts/realSessions.ts       # both languages on this machine's real session files; prints counts, and names a file only when it differs
bun scripts/surveyedSchemas.ts <folder> # checks surveyed schemas in both languages; prints counts only
```

Gate parts: `link`, `contract`, `biome`, `tsc`, `test`, `fmt`, `clippy`, `clippy-plain`, `cargo-test`, `crates`, `cases`, `symbols`, `demo`.

`link` makes `node_modules/@nyaaai` here, as a consumer's does. Scripts import packages by name, so the gate reads the same exports a consumer reads.

`cargo` comes from `PATH`, else `~/.cargo/bin`. `rust-toolchain.toml` pins the toolchain.
`demo` runs when `../nyaaai-demo` exists, or `NYAAAI_DEMO_DIR`.

Command line tool:

```sh
cargo run --manifest-path rust/Cargo.toml -p nyaaai-cli -- cut trim --source <file> --out <folder>
```

`cut` only reads its source. Exit 0 done, 2 refused, 1 failed.

```sh
cargo run --manifest-path rust/Cargo.toml -p nyaaai-cli -- serve    # the Rust adapter
bun ts/cli/src/main.ts serve                                        # the TypeScript adapter
```

## Cases

A case file is one of two types, shaped in `ts/testkit/src/caseFile.ts`:

- `function`: one command and its expected result.
- `run`: a script of frames against an adapter, and every frame expected back.

Expected output comes from TypeScript: function cases in process, scripted runs from the TypeScript adapter. The runner then holds both adapters to every case. Results compare as exact JSON text, key order included.

A function kind is `<group>.<operation>`. `CASE_KINDS` in `ts/testkit/src/caseKinds.ts` names each with its input and result shape. To add one:

1. Shapes `<GroupOperation>Input` and `<GroupOperation>Result`, in `CASE_KINDS` and the `cases` set.
2. `bun scripts/contract.ts --refresh` writes the Rust shapes and the dispatch table.
3. Handlers: `HANDLERS` in `scripts/caseAdapter.ts`, and `group_operation` in `rust/cli/src/case_handlers.rs`.
4. Inputs in `scripts/caseSources/`, listed in `SOURCES` of `scripts/cases.ts`.
5. `bun scripts/cases.ts --refresh`, then read the diff under `cases/`.

An adapter serves kind `a.b` as command `case:a.b`. Rust does so only with the `cases` feature.

A scripted run lists steps: `send`, `sendText`, `sendBase64`, `sendTail`, `sendOverCap`, `await`, `close`. `$TMP` is the run's temp folder. Frames compare per run, plus the connection's own in order, so runs may interleave. Never script a failure whose text the OS writes.

## Adapter

One process, many runs. `ts/cli/src/adapter.ts` and `rust/cli/src/adapter.rs` are twins. Frames are shaped in `shapes/frames`.

- Framing: UTF-8, one JSON frame per line, split on LF alone. Stdout holds frames only. A line over 8 MiB is refused unread. A blank line is skipped. An unterminated tail is dropped.
- Handshake: `hello` with the contract version, answered by `welcome`, or `reject` and exit 2. Compatible means the same major; before 1.0, the same minor too.
- `start` names its run and command. `fixed` scripts the run's clock and ids.
- A run sends `event` frames, numbered from 1, then one terminal `done` or `failed`.
- A run asks the host with `request`, ids `q1`, `q2`. The host answers `reply` or `reply-failed`, in any order. A command takes one answer with `reply(id)`; other messages keep their order.
- `steer` and `interrupt` reach a run while it waits. An interrupt ends a run that reads its inbox, and nothing reaches the run after it.
- A refused frame draws a `fault` and ends no run: `bad_frame`, `oversized_frame`, `not_greeted`, `unknown_run`, `duplicate_run`, `unknown_reply`.
- End of input interrupts open runs, then exit 0.
- Every message text is fixed and the same in both languages.

Commands: `sessionCut.plan`, `sessionCut.apply`. A compact with no summary handed over asks the host: request `sessionCut.summarize`, reply the summary text.

## Prompts

`prompts/<package>/<name>.md`, kebab-case. `{{name}}` is a slot. `bun scripts/contract.ts --refresh` writes `ts/<package>/src/prompts.ts` and `rust/<package>/src/prompts.rs`: a constant, or a function of its slots. One trailing line end belongs to the file. A prompt built by code is judged by cases on its output.

## Primitives

Engine code calls these, never the platform's own. Each is judged by cases. A new platform difference becomes a new primitive.

- Text is measured in UTF-16 code units. A cut never splits a surrogate pair.
- Order is by code unit. No locale compare.
- `canonicalJson` sorts keys by code unit. Fingerprints and dedup keys use it.
- `roundHalfUp` is `Math.round`. `fixedText` is `toFixed`. `numberOfText` takes a JSON number spelling and nothing else.
- Time is integer milliseconds. `parseRetryAfter` takes seconds, an HTTP date or an ISO time. `resetStampMs` reads a number below 1e11 as seconds.
- `dayKey` takes its offset as an argument.
- `base64` is the standard alphabet, padded, strict.

## Contract

A shape is Zod, in `ts/contract/src/shapes/<set>.ts`. Its set lists the named ones: `shapeSet("<set>", { Name, ... })`. A named shape is a `$defs` entry and a Rust type. An unlisted object inside one gets a derived Rust name: parent plus key.

Pipeline: Zod, `schema/<set>.<form>.json`, one intermediate form (`scripts/contract/ir.ts`), one emitter per language (`scripts/contract/rust.ts`). TypeScript types are `z.infer`. Never edit a generated file.

To add a set:

1. Module with its `shapeSet`, exported in `ts/contract/package.json`.
2. Import and target in `scripts/contract.ts`. `pub mod` in Rust.
3. `bun scripts/contract.ts --refresh`.
4. Round-trip cases for every named shape in `scripts/caseSources/contract.ts`. A test fails on a shape without one.

Dialect. `contract` fails on anything else:

- Object, array, record with text keys, nullable, union, discriminated union with a text tag.
- String, number, `z.int()`, boolean, null, unknown, one-value literal, enum of texts.
- Bounds: `min`, `max`, `gt`, `lt` on numbers; `min`, `max` on strings and arrays. String bounds count code points.
- Optional and default sit on an object property, outermost. Use `exactOptional`. No optional property holds a default.
- Recursion passes through a named shape.
- Hooks, ported by hand: `preprocessed(name, fix, schema)` maps the raw value before the decode; `refined(name, schema, holds)` judges the decoded value. Rust holds them in `<module>_hooks.rs`, snake case. A transform is written as a preprocess. A hook wraps nullable, not the reverse.

Rust mapping: optional and nullable are each `Option`; both is `Option<Option<T>>`. `z.int()` is `i64`, or `u64` with a minimum of zero or more. A required literal property has no field. A record is `Record<T>`. A by-value reference that closes a loop is boxed.

`CONTRACT_VERSION` lives in `shape.ts`. Generation copies it into each schema file and `rust/contract/src/version.rs`.

## Strict schemas

- Keep: `type`, `enum`, `const`, numeric and size bounds, `items`, `properties`, `required`, `additionalProperties`, `propertyNames`, `anyOf`, `oneOf`, `$ref`, `$defs`.
- Annotations: `description`, `title`, `default`, `examples`.
- Normalize, at schema nodes only: drop `$schema`, `pattern`, `format`; move the root under `$defs` when a `$ref` is `#`. A property named `pattern` stays.
- Refuse: all other keywords, malformed forms, unresolved references, `$ref` beside a validated keyword other than the root's `$defs`, loops that read no input, nesting past 64.
- Issue order: reference, type, equality, bounds, length, list, object, unions. A union member's issues are not reported.
- `number` and `integer` take finite numbers. `enum` and `const` compare by structure: key order aside, `1` equals `1.0`, and an overflowed number such as `1e400` equals no `null`.
- Limits: 256 nested checks and 1,000,000 visited schema nodes. Either ends the run with a `limit` issue, inside a union member too. A run also stops at 64 issues.
- `strictSchemaOf` (TypeScript): Zod to the subset. Refuses `any` and intersections.

## Consumers

A consumer holds this repository as a submodule and runs the script from its own root.

```sh
bun <submodule>/scripts/consumer.ts link                # node_modules/@nyaaai links; run at postinstall
bun <submodule>/scripts/consumer.ts dev                 # snapshot of NYAAAI_DIR into the submodule
bun <submodule>/scripts/consumer.ts dev --reset         # back to the recorded pin
bun <submodule>/scripts/consumer.ts bump                # submodule to origin/main
bun <submodule>/scripts/consumer.ts assert              # build check; allows a snapshot and says so
bun <submodule>/scripts/consumer.ts assert --package    # also refuses a snapshot, an unpushed commit, a dirty tree
```

- Packages used: `"nyaaai": { "packages": ["contract", "agentic"] }` in the consumer's `package.json`. Absent means all.
- `assert` fails on `node_modules` inside the submodule, a missing link, and a package dependency the consumer does not pin at the same version.
- A snapshot's subject starts with `Dev snapshot of `. Never pin one.

## Rules

- Contract first. Message shape written in Zod; Rust types generated.
- Cases judge parity. Slice done when both languages pass the same cases.
- Cases first. A moved module's tests become cases before it moves. Case runs use fakes; none makes a live request.
- TypeScript is a move of tested code. Rust is a port against the cases.
- Pairs land together: TypeScript module, Rust module and cases in one commit.
- Same module map and public names in both languages. `sessionCut.ts` is `session_cut.rs`.
  - Rust alone exports stand-ins for JavaScript built-ins: `contract::js`, and `quote_json`. A port calls them, never its own copy.
- Expected case output comes from TypeScript. Review the diff of every refresh.
- `agentic` holds keys and local URLs only. `operator` holds logins. `agentic` never imports `operator`; `scripts/__tests__/residue.test.ts` holds it.
- Clean break. No old code path survives its change.
- A change rewrites the documents it makes false, in the same commit.
- Engine names no tool. Tools declare their traits.
- Rust is synchronous where nothing waits on IO.
- A path check goes through `paths` in `process`. Cases with a folder tree judge it in both languages.
- Case kinds sit behind the `cases` feature. A plain `nyaaai` build holds no test code.
- Ids, the clock and randomness come through ports. Cases script them.
- No build output, no large corpus.
- Consumers import TypeScript source through a submodule. No `node_modules` inside one.
- A package lists what it needs in `dependencies`. The consumer pins each at that version.

## Decode rules

Same in both languages. `parseJson` and `parse_json` apply them.

- Lone surrogate escape becomes U+FFFD.
- Nesting past 1024 containers is refused.
- Rust spells JSON as `JSON.stringify` does: ECMAScript key order and number text.
- A file row is decoded text. Bad bytes become U+FFFD, in a kept row too.

`decode` and a generated `decode` apply these to a shape:

- Unknown keys are dropped at every depth.
- A default fills an absent key.
- A union takes its first matching branch.
- A refusal names the path of the first issue, in declaration order. A failed union names itself; an unknown tag names the tag.
- A list over its maximum refuses before any item is read. Under its minimum, after.
- A record drops a `__proto__` key.

## Dependencies

- Seven days old, or named by an advisory. `ts/bunfig.toml` enforces the age for Bun.
- `zod` is pinned at 4.6.7. An advisory names every version below 4.6.6.
- `scripts/crates.ts` checks `Cargo.lock` against crates.io publish dates and OSV advisories.
  - `crate-ages.json` beside a lock holds the dates.
  - `crate-exemptions.json` beside a lock lists reviewed exceptions: `young` crates, each with its advisory id, and accepted `advisories`. OSV must know a young crate's advisory, and the advisory must name the crate.
  - A package from a git source or another registry fails.

## Public repository

- No key, account id, real transcript or personal content. Test data is synthetic.
- No license file.
- Scan fixtures, cases and prompts before a push.

## Text

- No em dash, smart quote or zero-width character. `symbols` fails on one.
- In source, write `\u{2028}`, not the four-digit form: some editors decode it on save.
