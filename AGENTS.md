# NyaaAI

Shared AI framework for Nyaarium tools. TypeScript and Rust twins, judged by one set of cases.

## Layout

- `ts/`: Bun workspace. Packages `contract`, `process`, `agentic`, `operator`, `ui`, `testkit`, `adapter`, `cli`, scope `@nyaaai`.
- `rust/`: Cargo workspace. Crates `contract`, `process`, `agentic`, `operator`, `testkit`, `adapter`, `cli`.
- `prompts/`: prompt texts, one file each, by package. Both languages get a generated module.
- `schema/`: JSON Schema exported from the Zod contract, input and output form per set. Generated, committed.
- `cases/`: language-neutral cases. Generated from the case groups, committed.
  - A case is data. One that needs files names a folder tree, planted in a temp folder by each language.
  - `cases/traces/`: recorded exchanges in byte form, read by cases.
- `parity.ts`: this repository's project file: shape sets, case groups, scripted runs, adapters.
- `parity.generated.json`: what the last refresh generated, as sorted paths. Committed. `contract.ts` writes its `contract` list, `cases.ts` its `cases` list.
- `scripts/`: gate, contract generator, case runner, crate check, snapshot, consumer script.
  - `scripts/caseGroups/`: one module per case group. `scripts/caseSources/`: their inputs.

## Modules

Same names in both languages; Rust files are snake case.

- `contract`: primitives, ports and shapes. Rust adds `js`.
  - `json`: `parseJson`, `stringifyJson`, `canonicalJson`, `jsString` (Rust: `js::to_js_string`).
  - `text`: UTF-16 lengths, pair-safe cuts, finds, `trimText`, `collapseSpace`, `nfc`. `order`: `compareText`, `sortedTexts`. `base64`.
  - `number`: `numberText`, `roundHalfUp`, `fixedText`, `numberOfText`, `integerOfText`.
  - `time`: `parseIsoTime`, `isoTime`, `parseHttpDate`, `parseRetryAfter`, `resetStampMs`, `dayKey`. Ports `Clock`, `Scheduler` (TypeScript only); scripted `steppedClock`.
  - `id`: `randomUuid`. Port `Ids`; scripted `countedIds`. `random`: port `Random`; scripted `sequenceRandom`.
  - `pattern`: `Pattern`, one matcher written in both languages.
  - `shape`: `z`, shape sets, hooks (`preprocessed`, `refined`, `matching`), `decode`, `decodeText`, `CONTRACT_VERSION`. Rust's `shape` is the runtime generated code calls; `version` holds the constant.
  - `schemaExport`: Zod to JSON Schema, with the dialect check.
  - `strictSchema`: the JSON Schema subset: `normalizeSchema`, `schemaProblems`, `validate`. `issues`: its messages; Rust's is generated.
  - `shapes/frames`: stdio adapter frames. `shapes/sessionCut`: the cut's data shapes. Rust modules are generated.
- `process`: `paths` (`resolvedPath`, `isInside`).
- `operator`: `claude/sessionCut` cuts a Claude Code session file. `claude/sessionFile` reads its rows and holds cut ports over files. `claude/cliVersion` reads the installed version. `prompts` is generated.
- `adapter`: `runtime` serves commands over frames: `serve`, `command`, `RunContext`. `stdio` (TypeScript): `serveStdio` runs it on the process's own streams and flushes before exit.
- `cli`: `commands` lists NyaaAI's own. Rust adds the `cut` command and `cases/`, the case handlers.
- `testkit`: what a repository's parity setup is made of.
  - `caseGroup`, `caseContext`, `caseFile` shape the cases. `project` shapes `parity.ts`. `tree` plants a folder tree.
  - `claudeSession` builds synthetic session files. `tortureShapes` holds every contract construct, for the generator.
  - `trace` reads a recorded trace and replays it. `scheduler` holds a timer wheel on virtual time.
  - `rejection` (TypeScript): `rejection`, `rethrown` read what a promise rejects with.
  - `samples` (TypeScript): `sampleTexts` derives round-trip inputs from an exported schema.
  - `rustNames`: `snake`, `pascal`, `quote`, for a `parity.ts`'s generated Rust.
- `agentic`, `ui`: empty.

## Patterns

- Flags: `i`, `m`, `s`; literals, classes, groups, alternation, quantifiers.
- `\d`, `\w`, `\b`: ASCII; `\s`: ECMAScript whitespace. `.` and `m` anchors: LF ends a line.
- Matching uses Unicode scalars; offsets use UTF-16 units.
- Refused, since the two engines match them differently: a repeat whose item can match nothing; a capture a repeat round can skip.
- Limits: groups nest 32 deep; 99 capture groups; cost 10,000, counting atoms once counted repeats are unrolled, a class by its ranges.
- `split` follows ECMAScript. A capture that took no part is empty text.
- Syntax is a subset of ECMAScript's with the `u` flag: no source is accepted that `RegExp` refuses.
- Both languages run the same matcher. One search is linear in pattern size times text length. Listing every match can rescan, so its worst case is that times the match count.
- A lone surrogate in TypeScript text matches as U+FFFD.
- No package source holds a regex literal. A token test compiles through `Pattern`.

## Commands

Run from the repository root.

```sh
bun scripts/gate.ts               # everything below
bun scripts/gate.ts cases tsc     # `link`, then only the named parts
bun scripts/gate.ts --quiet       # prints a failed part's tail, then one summary line
bun scripts/contract.ts           # schema/ and generated Rust current, shapes inside the dialect
bun scripts/contract.ts --refresh # rewrite them from Zod
bun scripts/cases.ts              # case files current, both adapters agree
bun scripts/cases.ts --refresh    # rewrite case files from TypeScript
bun scripts/samples.ts            # values sampled from every schema, decoded in both languages
bun scripts/serve.ts              # the TypeScript adapter, with the case kinds
bun scripts/convertTrace.ts <trace.jsonl> <out.json>   # an old recorded trace into the byte form, scrubbed
bun scripts/unicodeTables.ts <ucd folder>              # the NFC tables, from Unicode 17.0 files; run by hand
bun scripts/crates.ts             # crate ages and advisories of rust/Cargo.lock
bun scripts/bannedSymbols.ts      # banned characters in tracked and untracked files
bun scripts/snapshot.ts           # commit the working tree for a consumer, print {ref, sha, subject}
bun scripts/realSessions.ts       # both languages on this machine's real session files; prints counts, and names a file only when a result differs or the file changed mid-run (`live`)
bun scripts/surveyedSchemas.ts <folder> # checks surveyed schemas in both languages; prints counts only
```

Gate parts: `link`, `contract`, `biome`, `tsc`, `test`, `fmt`, `clippy`, `clippy-plain`, `cargo-test`, `crates`, `cases`, `samples`, `symbols`, `demo`.

`contract.ts`, `cases.ts` and `serve.ts` work on the folder they are run from, through its `parity.ts`. `crates.ts` and `bannedSymbols.ts` work on that folder too: its `rust/Cargo.lock`, and its files minus nested checkouts, `node_modules` and `target`.

`link` makes `node_modules/@nyaaai` here, as a consumer's does. Scripts import packages by name, so the gate reads the same exports a consumer reads.

`cargo` comes from `PATH`, else `$CARGO_HOME/bin`, else `~/.cargo/bin`. `rust-toolchain.toml` pins the toolchain.
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

Expected output comes from TypeScript: function cases in process, scripted runs from the TypeScript adapter. The runner then holds both adapters to every case a source generates. Results compare as exact JSON text, key order included.

A function kind is `<group>.<kind>`. A group is one module in `scripts/caseGroups/`, listed in `parity.ts`:

```ts
export const text = caseGroup("text", {
	kinds: { cuts: kind(Input, Result, (input) => result) },
	sources: casesFor("text", textCases),
});
```

- `kind(input, result, run)`: two Zod objects and the TypeScript handler. The `cases` set names them `<Group><Kind>Input` and `Result`. Each kind holds its own two objects; a shared one is a factory.
- `shapes`: named shapes the group's kinds share.
- Rust handler: `fn <kind>(input)` in `<group>.rs` under `rust/cli/src/cases/`, snake case. `shapes.rs`, `dispatch.rs`, `normalizers.rs` and `mod.rs` there are generated.
- `kind(..., { context: true })` hands both handlers a `CaseContext`: `tmp()` a fresh folder, `root` the project root (read only), a scripted `clock` and `ids`. In a result, `$TMP` and `$ROOT` stand for the two folders.
- A folder tree is `TreeEntry` rows, planted with `plant`.

To add a kind:

1. The kind in its group, with its TypeScript handler.
2. `bun scripts/contract.ts --refresh`.
3. The Rust handler.
4. Inputs in `scripts/caseSources/`.
5. `bun scripts/cases.ts --refresh`, then read the diff under `cases/`.

An adapter serves kind `a.b` as command `case:a.b`. Rust does so only with the `cases` feature. The runner fails when a run changes the project tree.

A case file writes every banned character, and U+FEFF, as a `\uXXXX` escape.

Generated files are the ones `parity.generated.json` lists. A refresh removes a listed file the fresh run no longer produces, writes the fresh files, then rewrites its list. A file nobody generated is never read, reported or removed. Drift is a fresh file that is missing or differs, a listed file nothing generates, or a stale list.

A scripted run lists steps: `send`, `sendText`, `sendBase64`, `sendTail`, `sendOverCap`, `await`, `close`. `$TMP` is the run's temp folder. Frames compare per run, plus the connection's own in order, so runs may interleave. Never script a failure whose text the OS writes.

## Adapter

One process, many runs. `ts/adapter/src/runtime.ts` and `rust/adapter/src/runtime.rs` are twins. Frames are shaped in `shapes/frames`.

- Framing: UTF-8, one JSON frame per line, split on LF alone. Stdout holds frames only, and every frame written reaches it before the process ends. A line over 8 MiB is refused unread. A blank line is skipped. An unterminated tail is dropped.
- Handshake: `hello` with the contract version, answered by `welcome`, or `reject` and exit 2. Compatible means the same major; before 1.0, the same minor too.
- `start` names its run and command. `fixed` scripts the run's clock and ids.
- A run sends `event` frames, numbered from 1, then one terminal `done` or `failed`.
- A run asks the host with `request`, ids `q1`, `q2`. The host answers `reply` or `reply-failed`, in any order. A command takes one answer with `reply(id)`; other messages keep their order.
- `steer` and `interrupt` reach a run while it waits. An interrupt ends a run that reads its inbox, and nothing reaches the run after it.
- A refused frame draws a `fault` and ends no run: `bad_frame`, `oversized_frame`, `not_greeted`, `unknown_run`, `duplicate_run`, `unknown_reply`.
- End of input interrupts open runs, then exit 0.
- Every message text is fixed and the same in both languages. A command that ends with none, by a thrown non-Error or a panic, fails with `the command crashed`.

Commands: `sessionCut.plan`, `sessionCut.apply`. A compact with no summary handed over asks the host: request `sessionCut.summarize`, reply the summary text.

## Prompts

`prompts/<package>/<name>.md`, kebab-case. `{{name}}` is a slot. `bun scripts/contract.ts --refresh` writes `ts/<package>/src/prompts.ts` and `rust/<package>/src/prompts.rs`: a constant, or a function of its slots. One trailing line end belongs to the file. A prompt built by code is judged by cases on its output.

## Primitives

Engine code calls these, never the platform's own. Each is judged by cases. A new platform difference becomes a new primitive.

- `scripts/__tests__/residue.test.ts` refuses the platform's own calls in engine TypeScript: JSON, rounding, trimming, case mapping, number text, `Date`, a bare sort. It reads the code with the TypeScript compiler's own parser (`scripts/syntax.ts`). It goes by name, so a method of your own named like a platform one is refused too.
- `rust/clippy.toml` refuses Rust's diverging methods through Clippy's `disallowed-methods`: trimming, case mapping, rounding, whitespace splits, `lines`, character classes. `contract` allows one where it implements the primitive.
- Text is measured in UTF-16 code units. A cut never splits a surrogate pair.
- Whitespace is ECMAScript's: `trimText`, `collapseSpace`.
- `nfc` is Unicode NFC on tables frozen at 17.0, never the platform's. A lone surrogate becomes U+FFFD first. A code point unassigned in 17.0 passes through. `bun scripts/unicodeTables.ts <ucd folder>` writes both table modules from `UnicodeData.txt` and `CompositionExclusions.txt`; a later Unicode version is added beside them, never in place.
- Order is by code unit. No locale compare.
- `canonicalJson` sorts keys by code unit. Fingerprints and dedup keys use it.
- `roundHalfUp` is `Math.round`. `fixedText` is `toFixed`. `numberOfText` takes a JSON number spelling and nothing else.
- Time is integer milliseconds. `parseRetryAfter` takes seconds, an HTTP date or an ISO time. `resetStampMs` reads a number below 1e11 as seconds.
- `dayKey` takes its offset as an argument.
- `base64` is the standard alphabet, padded, strict.

## Contract

A shape is Zod, written with `z` from `@nyaaai/contract/shape`, in `ts/contract/src/shapes/<set>.ts`. Its set lists the named ones: `shapeSet("<set>", { Name, ... })`. A named shape is a `$defs` entry and a Rust type. An unlisted object inside one gets a derived Rust name: parent plus key.

Pipeline: Zod, `schema/<set>.<form>.json`, one intermediate form (`scripts/contract/ir.ts`), one emitter per language (`scripts/contract/rust.ts`). TypeScript types are `z.infer`. Never edit a generated file.

To add a set:

1. Module with its `shapeSet`, exported in `ts/contract/package.json`.
2. Import and Rust target in `parity.ts`. `pub mod` in Rust.
3. `bun scripts/contract.ts --refresh`.
4. Round-trip cases for every named shape in `scripts/caseSources/contract.ts`. A test fails on a shape without one.

`samples` holds every shape to more than its hand-written cases. For each named shape it derives values from the exported schema: a full one, a minimal one, each branch, an unknown key, then one broken rule at a time. Both languages decode each through the project's `roundTrip` kind, and a difference fails the gate. It stores nothing.

Dialect. `contract` fails on anything else:

- Object, array, tuple of fixed places, record with text keys, nullable, union, discriminated union with a text tag.
- String, number, `z.int()`, boolean, null, unknown, one-value literal, enum of texts.
- Bounds: `min`, `max`, `gt`, `lt` on numbers; `min`, `max` on strings and arrays. String bounds count code points.
- No string format (`z.base64()`, `z.email()`, `z.uuid()`, `z.iso.datetime()`): write `matching`, or a `refined` hook over a primitive.
- Optional and default sit on an object property, outermost. Use `exactOptional`. No optional property holds a default.
- Recursion passes through a named shape.
- A tuple holds no rest, no optional place, no preprocess place, and at most 12 places.
- A named shape that is not an object or a union never names itself, through other such shapes either. A recursive shape is an object or a union.
- Hooks, ported by hand: `preprocessed(name, fix, schema)` maps the raw value before the decode, and never sees a missing one; `refined(name, schema, holds, message?)` judges the decoded value. Rust holds them in `<module>_hooks.rs`, snake case: one function per name, serving one type. A transform is written as a preprocess.
- `matching(schema, source, flags, message?)` checks text through `Pattern`. Rust is generated.
- Hooks stack, innermost first. A bound comes before a hook. A hook wraps nullable, not the reverse.
- A `message` is the refusal's text in both languages; `fixed` marks such a refusal. Any other refusal text is the language's own.

Rust mapping: optional and nullable are each `Option`; both is `Option<Option<T>>`. `z.int()` is `i64`, or `u64` with a minimum of zero or more. A required literal property has no field. A record is `Record<T>`. A tuple is a tuple. A by-value reference that closes a loop is boxed, across sets too; a loop across crates is refused.

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
- `assert` fails on `node_modules` inside the submodule, a missing link, and a package dependency (Zod) that the consumer's root, or a workspace `package.json` naming it, does not pin at the same version.
  - With a `rust/Cargo.toml`: a `rust-toolchain.toml` whose `channel` differs, and a `rust/clippy.toml` lacking a `disallowed-methods` path of the submodule's. A missing file fails.
- A snapshot's subject starts with `Dev snapshot of `. Never pin one.

Parity of its own:

- Packages `contract`, `testkit`, `adapter`, and a `parity.ts` at its root: `export default project({ sets, schemaOnly, groups, runs, cases, commands, rustAdapter, generated })`. NyaaAI's own `parity.ts` is the example.
- It runs `consumer.ts`, `contract.ts`, `cases.ts`, `samples.ts`, `crates.ts` and `bannedSymbols.ts` from its root, and Clippy with the shared `clippy.toml`. `consumer.ts assert` runs with `--package` on its release path. The other scripts are NyaaAI's own. A residue test keeps what a consumer runs on the three packages, and `assert` fails when one is not linked.
- A `sets` value is `{ crate, module, file }`, `file` relative to its root. A set `parity.ts` imports sits in `sets`, `schemaOnly` or `external`; `contract.ts` fails on any other.
- `external`: sets generated elsewhere, each with its Rust target, or `null` for none. NyaaAI's own (`frames`, `tree`, `caseFile`) are external without being listed.
- Opt in: `tests`, a command that must pass before a case refresh; `prompts`, the prompt folder; `roundTrip`, the kind `samples.ts` runs.
  - The kind takes `{ set, shape, texts }` and answers `{ results }`, one per text. `scripts/caseGroups/contract.ts` and `rust/cli/src/cases/contract.rs` are the example.
  - `samples.ts` covers every named shape of `sets`. A shape whose built value does not decode needs a case of that kind with a text that does.
  - Absent, `samples.ts` samples nothing.
- Generated output sits at its root: `schema/`, `cases/<group>/`, `parity.generated.json`, and the Rust files its targets name.
- A refresh writes what `parity.generated.json` then lists: case files, schema files, generated Rust and prompt modules. Any other file is left alone.
- The first refresh has no list to compare with, so it removes nothing. Delete generated files left from before by hand, once.
- Group and kind names are letters and digits, a letter first. Case and run names are letters and digits in parts joined by one `.`, `_` or `-`.
- Rust: path dependencies on the submodule's `rust/contract`, `rust/adapter` and `rust/testkit`. `cases` names the crate, module and folder the generator writes `shapes.rs`, `dispatch.rs` and `mod.rs` into. Handlers are by hand: `cases/<group>.rs`, `pub fn <kind>(input: <Group><Kind>Input) -> Outcome<<Group><Kind>Result>`. A context kind takes `context: &CaseContext` second.
- Its `main` serves with `nyaaai_adapter::runtime::serve`. A `case:` command goes to `cases::dispatch::dispatch(kind, input, root)`, the root being the current directory read once at start. `rust/cli/src/main.rs` is the example.
- `rustAdapter.build` and `.run` are relative to its root. `rust/target` is ignored in git: the runner fails on a case run that changes the tree.
- Toolchain: crates of edition 2024; its `rust-toolchain.toml` pins the submodule's channel exactly, with `rustfmt` and `clippy`. `cargo fmt` runs from `rust/`. Biome of its own, under `node_modules` at its root or in `ts/`.
- Copy `rust/clippy.toml`; `assert` checks it. Clippy lints the path crates too.
- `Outcome` and `CaseContext` live in `nyaaai_testkit::cases`.
- Round trip: the generator writes `normalizers.rs` beside `shapes.rs`. `normalizers::normalize(set, shape, value)` calls the `normalize(shape, value)` of each set's generated module, and answers `None` for an unknown set or shape. `rust/cli/src/cases/contract.rs` is the example.
- `generated` helpers import `snake`, `pascal` and `quote` from `@nyaaai/testkit/rustNames`, never from `scripts/`.
- TypeScript: a consumer type-checks NyaaAI's source. A `tsconfig` at its root extends `<submodule>/ts/tsconfig.base.json`, includes the `src` of the packages it uses, emits to a folder at its own root, and is referenced by its projects. NyaaAI builds with TypeScript 7.0.2; one consumer compiles it with 6.0.3.
- A workspace package of the consumer imports `@nyaaai/*` through the root link and declares nothing. `nyaaai.packages` at the root is the declaration. The root pin of a package dependency (Zod) is what makes it resolvable from the linked source.
- Case files hold banned characters and U+FEFF as escapes; a consumer's residue and symbols checks need no exemption.
- An in-tree submodule is swept by the consumer's own tools: ignore it in its formatter, its test runner and its own residue tests.

## Rules

- Contract first. Message shape written in Zod; Rust types generated.
- Cases judge parity. Slice done when both languages pass the same cases.
- Cases first. A moved module's tests become cases before it moves. Case runs use fakes; none makes a live request.
- TypeScript is a move of tested code. Rust is a port against the cases.
- Pairs land together: TypeScript module, Rust module and cases in one commit.
- Same module map and public names in both languages. `sessionCut.ts` is `session_cut.rs`.
  - Rust alone exports stand-ins for JavaScript built-ins: `contract::js`, and `quote_json`. A port calls them, never its own copy.
- Expected case output comes from TypeScript. Review the diff of every refresh.
- A test reads a rejection with `rejection` or `rethrown`, never `expect(...).rejects`.
- `agentic` holds keys and local URLs only. `operator` holds logins. `agentic` never imports `operator`; `scripts/__tests__/residue.test.ts` holds it.
- Clean break. No old code path survives its change.
- A change rewrites the documents it makes false, in the same commit.
- Engine names no tool. Tools declare their traits.
- Rust is synchronous where nothing waits on IO.
- A path check goes through `paths` in `process`. Cases with a folder tree judge it in both languages.
- Raw text and raw streams are read by a real parser: the platform's, the language's own, or a hand-written scanner that tracks state. Never a regex.
  - A pattern may test a token a parser already produced, such as a name beginning with `ghp_`.
  - A token that is a language of its own is parsed again: a date, a number spelling, a version, a reference. A pattern never pulls the parts out of one.
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
  - Zod reports a branch's issue when one branch failed softly. TypeScript's `decode` moves such a refusal to the union.
- A list over its maximum refuses before any item is read. Under its minimum, after.
- A tuple of the wrong length refuses at its own path, before any place is read.
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
- `scripts/convertTrace.ts` replaces generation ids and every id found under a `session_id` key, wherever that id recurs. It sees no other secret. Read a converted trace before committing it.

## Text

- No em dash, smart quote or zero-width character. `symbols` fails on one, and on U+FEFF.
- In source, write `\u{2028}`, not the four-digit form: some editors decode it on save.
