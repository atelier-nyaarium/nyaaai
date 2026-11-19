# NyaaAI

Shared AI framework for Nyaarium tools. TypeScript and Rust twins, judged by one set of cases.

## Layout

- `ts/`: Bun workspace. Packages `contract`, `process`, `agentic`, `operator`, `ui`, `testkit`, `adapter`, `cli`, scope `@nyaaai`.
- `rust/`: Cargo workspace. Crates `contract`, `process`, `agentic`, `operator`, `testkit`, `adapter`, `cli`.
- Tables TypeScript owns and Rust reads as generated constants (`scripts/rustConstants.ts`, `parity.ts`): the strict schema keyword policy and limits (`strict_policy.rs`), the HTTP date tokens (`time_tokens.rs`), the adapter's limits and texts (`adapter_texts.rs`, in `contract` so `adapter` and `testkit` both reach it), the curated model list as one JSON text of `ModelRow` (`models/curated_table.rs` in `agentic`, read through the shape).
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

- `contract`: primitives, ports and shapes. Rust adds `js`, `future`, `cancel`.
  - `json`: `parseJson`, `stringifyJson`, `canonicalJson`, `jsString` (Rust: `js::to_js_string`).
  - `text`: UTF-16 lengths, pair-safe cuts, finds, `trimText`, `collapseSpace`, `nfc`, `Utf8Decoder` (`push`, `end`), `encodePathSegment`, `hasControl` (Rust `has_control`). `order`: `compareText`, `sortedTexts`. `base64`.
  - `number`: `numberText`, `roundHalfUp`, `fixedText`, `numberOfText`, `integerOfText`.
  - `time`: `parseIsoTime`, `isoTime`, `parseHttpDate`, `parseRetryAfter`, `resetStampMs`, `dayKey`. Ports `Clock`, `Scheduler` (`sleep`, `schedule`); scripted `steppedClock`. Rust's `Scheduler` is awaited: `sleep` answers `Slept`, `schedule` answers `Scheduled`; `SystemScheduler` needs a `tokio` runtime with time enabled and takes its deadline at the call. Delays are whole milliseconds, at most 2^53-1; TypeScript's `systemScheduler` waits zero for a negative or non-finite one and chains timers past 2^31-1. `deadline(scheduler, ms, caller)` ends at `ms` or with the caller's signal (Rust: a `CancelToken`, giving a child token and a drop guard), says which came first through `ended`, and `clear` drops its timer and the link to the caller (a later cancel reaches nothing, `ended` keeps its answer); it stands for `AbortSignal.timeout` and `AbortSignal.any`.
  - `id`: `randomUuid`. Port `Ids`; scripted `countedIds`. `random`: port `Random`; scripted `sequenceRandom`.
  - `pattern`: `Pattern`, one matcher written in both languages.
  - `shape`: `z`, shape sets, hooks (`preprocessed`, `refined`, `matching`), `decode`, `decodeText`, `CONTRACT_VERSION`. Rust's `shape` is the runtime generated code calls; `version` holds the constant.
  - `schemaExport`: Zod to JSON Schema, with the dialect check.
  - `strictSchema`: the JSON Schema subset: `normalizeSchema`, `schemaProblems`, `validate`. `issues`: its messages; Rust's is generated.
  - `shapes/frames`: stdio adapter frames. `shapes/sessionCut`: the cut's data shapes. `shapes/step`: a model request and its reply, effort, failure, stream items, header list. `shapes/provider`: entries, kinds, plans, key states, provider views, and the run types they name (`Billed`, `Skip`, `Passed`, `NoticeKind`, `Disposition`). `shapes/model`: rows, views, prices, test records, the stored registry, browse shapes, `Via`, `Unresolved`. Rust modules are generated; `shapes/model_hooks.rs` is by hand.
  - `secret`: `secretSpellings(secret)`, `scrubSpellings` (one pass, longest match, every match becomes `[key]`, what was written is never reread), `leaksSpelling`, `scrubSecret`. The spellings: as is; path-encoded with hex digits in either case; base64 at each of the three byte alignments, unpadded, as the characters holding only the secret's bits and as that run to the end of the text. An empty secret has none.
  - `headers`: `headerOf`, the first value of a name, ASCII case aside. `responseHeaders` (Rust `response_headers`): lowercase names, sorted by code unit, repeats joined with `, `, `set-cookie` one entry per value.
  - `url`: `originOf` (Rust `origin_of`), the origin of an http, https, ws, wss or ftp URL as the WHATWG parser writes it; `undefined` for another scheme, a malformed URL or a non-ASCII or percent-encoded host.
    `parseUrl` (`UrlParts`: scheme, credentials, host, port, origin, path, query, fragment) reads the same URLs, and `originOf` is its `origin`. The path is the platform's `pathname`; credentials, query and fragment are raw text. Rust has none yet.
  - `fetch`: port `Fetch`, one call: `FetchRequest` and a cancel signal answer `FetchResponse`, its body read as chunks. Rust: `dyn Fetch`, `Chunks`, `FetchError`.
    - An adapter owes: the cancel covers the whole exchange, so a pending or later chunk read ends cancelled (Rust `FetchError::Cancelled`; TypeScript rejects with the signal's reason). Response headers are in the `responseHeaders` form. A `GET` carries no body. The connection is released at the end of the chunks, when the reader stops early (iterator `return`, Rust drop), or at the cancel.
  - Ports of the provider layer, each with a scripted fake in the test kit: `EntryKeys` (`get(id)` answers a key, none, refused or unready; `onChange` names the entry), `ProviderList` (`entries()` in priority order; `onChange`), `ModelRegistry` (`rows()`, `stored()`, awaited `record(entry, id, record)` answering null or why not; `onChange`). A consumer reacts to `onChange`, never to a changed reference. Rust: `entry_keys`, `provider_list`, `model_registry`; `cancel::Unsubscribe` ends a subscription.
  - `future` (Rust): `BoxFuture`. `cancel` (Rust): `CancelToken`, cloneable; `cancel`, `is_cancelled`, `cancelled`, `run_until`, and `child`, a token that follows its parent's cancel and not the other way.
- `process`: `paths` (`resolvedPath`, `isInside`).
- `operator`: `claude/sessionCut` cuts a Claude Code session file. `claude/sessionFile` reads its rows and holds cut ports over files. `claude/cliVersion` reads the installed version. `prompts` is generated.
- `adapter`: `runtime` serves commands over frames: `serve`, `command`, `RunContext` (Rust: `serve`, `serve_blocking`, `blocking`, `typed`, `typed_async`). `stdio` (TypeScript): `serveStdio` runs it on the process's own streams and flushes before exit.
- `cli`: `commands` lists NyaaAI's own. Rust adds the `cut` command and `cases/`, the case handlers.
- `testkit`: what a repository's parity setup is made of.
  - `caseGroup`, `caseContext`, `caseFile` shape the cases. `project` shapes `parity.ts`. `tree` plants a folder tree.
  - `claudeSession` builds synthetic session files. `tortureShapes` holds every contract construct, for the generator.
  - `trace` reads a recorded trace; `replayFetch(trace, wheel)` plays it as a `Fetch` on a wheel; nothing waits. `scheduler` holds a timer wheel on virtual time; `fakeScheduler` (Rust `FakeScheduler`) runs sleeps and scheduled calls from it as it advances. `cases` (Rust) holds `block_on`, so a handler can await.
  - `scriptedPorts`: `scriptedEntryKeys`, `scriptedProviderList`, `scriptedModelRegistry` (Rust `ScriptedEntryKeys`, `ScriptedProviderList`, `ScriptedModelRegistry`). The test sets values; listeners are called in subscription order; `reads()` counts reads.
  - `scriptedFetch(exchanges, wheel)`: a `Fetch` that plays exchanges in order and records each request. An exchange holds a status, raw headers, chunks (each with a delay), a delay before the head, and an end: `closed`, `dropped` after the last chunk, or `refused` before the head. Answering the head or a chunk first advances the wheel by its delay, then fails with the signal's reason if the signal aborted meanwhile. `read()` counts the chunks handed out. `keys`: `fakeKeys(values)`, a key source answering each read from a list, the last repeating; `reads()`.
    Rust: `ScriptedFetch`, `replay_fetch` and `FakeKeys` (a sync `new`, an awaited `get`); a `Fetch` takes a `CancelToken` where TypeScript's takes a signal. `loopback` (Rust only): `Loopback`, an HTTP/1.1 server on `127.0.0.1` that answers each connection with one `LoopbackExchange`, its chunks flushed apart, and holds the connection before the head or after the chunks so a test sees the client close it.
  - `recorder` (TypeScript): `recordingFetch(inner, secrets)` answers `{ fetch, trace }`. Every call passes to `inner`; `trace()` answers the `Trace` of the request bodies (JSON parsed, else text), statuses, the replay headers (`content-type`, `retry-after`, `x-ratelimit-*`) and each chunk's bytes with its edges. A request header is never kept. `trace()` throws when a secret, as is, path-encoded or in base64 without padding, occurs in a request body, a kept header or the joined bytes of a body, across edges too; the message names the exchange and the place, never the secret. Bytes are never rewritten.
  - `loopback` (TypeScript): `loopbackProvider(trace, holds?)`, an HTTP server on `127.0.0.1` that answers each request with the next exchange, its chunks flushed apart. A hold keeps an exchange open before its head (`head`) or after its chunks (`chunks`); `arrived(at)`, `closed(at)` and `headers` let a test see the request and the close.
  - `rejection` (TypeScript): `rejection`, `rethrown` read what a promise rejects with.
  - `samples` (TypeScript): `sampleTexts` derives round-trip inputs from an exported schema.
  - `rustNames`: `snake`, `pascal`, `quote`, for a `parity.ts`'s generated Rust.
- `agentic`: each module an entry (`@nyaaai/agentic/sse`); only `noticeText` is a browser entry.
  - `noticeText`: `overloadedText`, `unavailableText`, the sentences a notice shows.
  - `provider/entry`: `PROVIDER_KINDS`, `PROVIDER_PLANS`, `isEntryId`, `secretKey(id, namespace)` (the consumer names the namespace), `sameOrigin`, `parseEntry`, `parseEntries`. A reader of a stored value: what does not fit is dropped, item by item. A local URL is an origin; a `compatible` URL keeps its path without trailing slashes; no user, query or fragment; a non-ASCII or percent-encoded host is refused.
  - `provider/disposition`: `SKIP_REASONS`, `dispositionOf`, `skipReasonOf`.
  - `models/rowKey`: `rowKey`, `parseRowKey`, `shownId`, `shownKey`, `servedIdOf`. A key is a curated key as it is, `<kind>:<id>` or `local:<entry>/<tag>`.
  - `models/curated`: `MODELS`, the curated list; Rust reads it from the generated `CURATED_JSON`.
  - `models/registry`: `parseTestRecord`, `parseRegistry` (settings form in, `StoredRegistry` out), `registryFor`, `registryOf`, `registryRows`, `rowOf`, `offered`, `modelList`, `select`, `passed`, `failedGate`, `isModelId`, `curatedUnder`, `testedUnder`. `RegistrySettings` is the in-memory form: a `Map` of records in stored order, an array, a `Set`.
    Rust: `RegistrySettings` holds three vectors (records in stored order, added, hidden without repeats); `parse_registry` takes a `RawRegistry` of the three settings values and the entry ids; `parse_row_key_text` reads a `&str`, `parse_row_key` a `Value`; `Gate` names the failed gate.
  - `models/view`: `unknownView`, `tiersOf`, `codecFactsOf` (an output limit is floored, left out below one or when not finite), `runEffort`, `raced` (TypeScript only: a signal is the platform's).
  - `kinds/opencode/retention`: `retentionOf`, `isZdr` over the plan tables.
  - `provider/kind`: what a kind gives a host: `KindDeps` (`fetch`, `scheduler`, `log`, `userAgent`, `baseUrl?`), `CatalogDeps` (`scheduler`, `rows`: the registry's `rows` and `onChange`, `refreshed`), `EntryCatalog`, `Browser`, `Tester`, `Pins`, `Route` (transport and catalog), `ProviderKind`.
  - `kinds/readers`: readers of loose catalog JSON (`fieldsOf`, `textField`, `numberField`, `booleanField`, `textsField`, `objectField`, `dataList`). A missing, null or mistyped field is null; a number is finite.
  - `kinds/openrouter/catalog`: `parseModels`, `parseEndpoints` (readers by hand, a bad item dropped, none when the body holds no list), `perMillion`, `expiryOf`, `chatStanding`, `joinCatalog`, `ModelCatalog` (12 hours, one shared refresh, a failure keeps the last data and waits 60 seconds, a deadline of 30 seconds from the scheduler; a join is reused until the registry tells a change, the data is refetched, the flag differs or a removal day passes; `dispose` stops listening). `expiryOf` reads a day or an ISO time, else NaN. `perMillion` reads a number or a JSON number spelling.
  - `kinds/openrouter/browse`: `browseModels`, `parseProviders`. `kinds/openrouter/key`: `checkKey`. `kinds/openrouter/browser`: `openRouterBrowser`, `openRouterTester` and its `Pins` (`hop` has a 30 second deadline that follows the caller).
  - `kinds/opencode/catalog`: `parseOpenCode`, `joinOpenCode`, `OpenCodeCatalog`, `opencodeBrowser`, `opencodeTester`. A model is flat: only what the join reads, each field null when missing.
    Rust: `kinds/fetcher` (`Fetcher`, `Waiter`, `Aborted`) holds the shared refresh both catalogs use: the refresh is spawned on `tokio`, so a caller that cancels ends only its own wait; `Ordered` keeps keys in first order; `KindError` is how a browser, tester or hop ends (`Cancelled`, `TimedOut`, `Provider`); the browser caches nothing across calls (a pure function of the listing and the time). `Transport::get_json` yields once after its key and once after the head, as the TypeScript awaits them, so two reads started together interleave alike.
  - `kinds/local/catalog`: `LocalCatalog`, `LocalModel`, `LocalServer`, `localView`, `contextOf`, `labelOf`, `effortsOf`, `itemsOf`, `localBrowser`, `localTester`. One entry's discovered models, kept after the first success. A read of the server has a `deadline` of 10 seconds from the scheduler and shares one run; a failure before any success waits 60 seconds. `settled(tag)` (a reply was seen) rereads a model whose facts were not read at most once a minute per tag, else asks the server for a loaded context once per tag (every reply on a `live` server) unless the model has a configured context. `kinds/local/kind`: `localKind(spec)`, `LocalKindSpec`, `Terms`, `HOME`. The check answers `rejected` for `refused`, `keyRejected` and `keyMissing`, else `unreachable`.
  - `kinds/ollama`, `kinds/lmstudio`, `kinds/llamacpp`, `kinds/compatible`: `OLLAMA`, `LMSTUDIO`, `LLAMACPP`, `COMPATIBLE` and the readers of their bodies by hand (a field missing, null or mistyped falls back; an item with no usable name drops; a body with no list gives none). `numCtxOf` reads a parameters text as the multiline `^num_ctx\s+(\d+)\s*$` does. `termsOf` calls a host private when `parseUrl` writes it as loopback, a private range, a LAN name or one label, and remote otherwise, a URL that does not read included; `baseOf` throws on such a URL, so a check answers `unreachable` and asks nothing.
    Rust: `ProviderKind` is a trait with defaults (`KindField` for the add form's third field); `LocalKind` implements it from a `LocalKindSpec` of plain fields and function pointers, and the four kinds are `LazyLock` statics (`OLLAMA`, `LMSTUDIO`, `LLAMACPP`, `COMPATIBLE`). `transport` refuses a URL the kind cannot read with its text, and `check` turns that into `unreachable`. `LocalServer` has `reads_loaded`, `live` and `warms` flags beside its calls, and the catalog's transport listener holds the catalog weakly. The catalog stays on `Fetcher` with no age limit (`Fetcher::refresh_now` is the fresh read, `Fetcher::good` the last list); a reader that needs the last list takes it before the refresh starts, since a refresh starts under the fetcher's lock. `kinds/join` (`join_all`) runs futures at once, polled in order, where TypeScript uses `Promise.all`; `KindError::Message` is any other refusal by its text.
  - `effort`: `EFFORTS`, `resolveEffort`.
  - `sse`: `SseDecoder` (`push`, `end`).
  - `step`: `StepAssembler`, `replyText`, `replyCalls`, `replyThoughts`.
  - `failure`: `WireError`, `ProviderError`, `failed`, `failureOf`, `messageOf`.
    Rust adds `NumberOrText`, `FailureExtra` (what `failed` takes beside the category and message); `ProviderError` boxes its `Failure`; `failure_of` reads a `dyn Error`.
  - `native`: `Host` and the helpers that read loose provider JSON: `isJson`, `stringOf`, `numberOf`, `argumentsOf`, `parsedChunk`, `wireError`, `streamFailure`, `assembled`, `servingEvents`, `schemaPrompt`, `systemTexts`, `currentTurnFrom`.
    Rust adds `HostStatus` (`Http` or `Stream`); `Host` is a trait.
  - `schemaProjection`: `cleanSchema`, `nullableOptionals`, `withoutNulls`, `strictSchema`.
  - `codec`: `Codec` (`encode`, `stream`, `complete`, `failure`) and `StepTurn`.
  - `chat/codec`: `chatCodec(dialect)`, `ChatDialect`. `chat/text`: `normalizeResponseValue`, `extractTextFromSegments`, `textOf`, `thoughtText`. `chat/wire`: the wire types and the readers of a reply (`readChunk`, `readCompletion`, `readUsage`, `readError`): a mistyped field is missing, a mistyped count is zero.
  - `dialects/openrouter`: `OPENROUTER_CHAT`, `PROVIDER`, `thoughtsOf`, `detailsOf`. `dialects/opencode`: `opencodeChat(facts, clock)`, `opencodeErrors(clock)`. `dialects/local`: `localChat(facts, host, local)`, `LocalDialect`. A dialect reads a `CodecFacts`.
  - `responses/codec`: `responsesCodec(host, facts)`. `messages/codec`: `messagesCodec(host, facts)`. `gemini/codec`: `geminiCodec(host, facts)`, `SKIP_SIGNATURE`. Each has a `wire` file of its wire types. They read provider JSON loosely: a field of the wrong type is missing, a usage count of the wrong type or a negative one is zero; an `error` field is an error unless absent, `null`, `false`, zero or the empty text, a text being its own message.
  - `hosts/opencode`: `SESSION_HEADER`, `opencodeHost(prefix, clock)`, `opencodeFailure(clock, ...)`, `opencodeUsage`. Reset stamps count from the clock given.
  - `fetch`: `platformFetch()`, the `Fetch` over the platform's `fetch`, the one engine module that names it (the residue test holds it). Follows no redirect (a 3xx answers as its status), sets no timeout, answers no chunks for a response with no body, and keeps what the port's doc says an adapter owes.
  - `transport`: `Transport` (`stream`, `complete`, `getJson`, `postJson`, `onSettled`), `KeySource`, `staticKey`, `isKeyShape`, `normalizeKey`. It takes a `Fetch`, a `Scheduler`, a `KeySource`, a log, the base URL, a label and the consumer's `name/version` as `User-Agent`; no retry.
    Rust: `stream` answers an `EventStream` read with an awaited `next`, nothing running before the first; a call ends `TransportError::Cancelled` when the caller's token is cancelled, else a `ProviderError`. `StreamDecoder::over` tells the transport to read no further chunk. `fetch` (Rust only, Cargo feature `http`, off): `ReqwestFetch`, the `Fetch` over `reqwest`, built with no proxy from the environment and no redirect followed; its error text has no URL and `Authorization` is marked sensitive. A dropped `next` while the stream opens leaves it ended: it sends nothing again, logs once and tells nobody. `KeyAuth` and `FetchRequest` print `[key]` for a key.
    - A request's headers: the codec's own, less any named `Content-Type`, `Accept`, `Authorization` or `User-Agent` (ASCII case aside), then `Content-Type`, `Accept`, `Authorization` when there is a key, `User-Agent`. A JSON call sends `Accept`, `User-Agent`, `Authorization`, then `Content-Type` when it has a body.
    - One idle timer: `scheduler.schedule`, armed before the request and after the head, after every chunk and after the end of the body. It aborts the exchange; a caller's `AbortSignal` is the other cancel. A JSON call has no idle timer. An unstreamed body has one window after its head. An error body is read to 64 KiB and has one window from the request.
    - A header name must be token characters and a value visible ASCII, space or tab; else `badRequest` before the key is read, naming the header and never its value. `maxBytes` (default 16 MiB) caps an unstreamed or JSON reply (`network`) and a stream's bytes since a chunk last finished an item (`stream`). `log` and listeners that throw are dropped.
    - The key source is read once per call. A caller already aborted gets its reason at once, with no request. `Retry-After` goes through `parseRetryAfter` with the scheduler's time, when the codec's failure holds none.
    - Failures, serving events and the log line are scrubbed of the key in every spelling of `secret`. Text, thought and call events are not.
    Rust: `Codec` and `ChatDialect` are traits; `stream()` answers a `StreamDecoder` pushed one item at a time, events appended to the caller's list so those before a failure stay. `ToolChoiceRule`, `Mode`, `HostStatus`, `Billing` and `Usage` are Rust's own spellings of what TypeScript leaves loose; `opencode_usage` takes the decoded `Usage`.
    Rust adds `loose` (readers of loose provider JSON fields, shared by the native codecs), `opencode_chat_usage` (the Chat dialects' `opencode_usage`, over a decoded `Usage`), and `OpenCodeHost`, which `opencode_host` returns; `Host` is a trait.
- `ui`: empty.

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
bun scripts/record.ts openrouter [scenario]            # paid, run by hand; key on standard input; writes cases/traces/openrouter.<scenario>.json
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

An adapter serves kind `a.b` as command `case:a.b`. Rust does so only with the `cases` feature. Both languages hold a kind's result to its shape. The runner fails when a run changes the project tree.

A case file writes every banned character, and U+FEFF, as a `\uXXXX` escape.

Generated files are the ones `parity.generated.json` lists. A refresh removes a listed file the fresh run no longer produces, writes the fresh files, then rewrites its list. A file nobody generated is never read, reported or removed. Drift is a fresh file that is missing or differs, a listed file nothing generates, or a stale list.

A scripted run lists steps: `send`, `sendText`, `sendBase64`, `sendTail`, `sendOverCap`, `await`, `close`. `$TMP` is the run's temp folder. Frames compare per run, plus the connection's own in order, so runs may interleave. Never script a failure whose text the OS writes.

Codec cases name a codec by data: `{ family, dialect, facts?, local?, nowMs?, prefix? }`; a native family takes the host's `prefix`. `scripts/caseGroups/codecOf.ts` builds it from a table keyed `<family>/<dialect>`; a new family adds a row there and the same arm in `rust/cli/src/cases/codec.rs`.

Transport cases (`transport.exchange`) run calls in order on one `Transport` over a scripted network (`scriptedFetch`, or a trace in `cases/traces/` through `replayFetch`) and key source, on a wheel that starts at `startMs`. A call may be aborted by the caller at a wheel time after its start. The result holds each call's events, result, answer, failure or thrown message, the requests as sent, the log lines, the models `onSettled` was told, the key reads, the chunks read per exchange and the timers left waiting.

Catalog cases (`catalog.openrouterRun`, `catalog.opencodeRun`) run a catalog on a scripted network and the wheel. `exchanges` are what the network answers in the order the catalog asks (a reply holds `json` or raw `text`, a head delay and an end). `steps` are `call` (one call, with the caller's abort as in the transport cases), `together` (calls at once), `advance` (wheel time) and `rows` (the registry's rows change by key and it tells). A step's result holds the wheel's time, each call's answer and the registry reads during it, which show whether a join was reused. A call that a refresh has already started does not stop when its caller aborts. `browse.*Browser` cases do the same for a browser and a tester.

Local cases (`local.*`): decoders take a body and answer what they read. `local.reach` runs a kind's check on a scripted network. `local.server` runs a kind's server on one. `local.flow` runs a kind's transport, catalog, browser and tester on one, with chats through the transport (the reply of a chat triggers `settled`); each step ends when background work waits for the wheel. `local.catalogRun` runs a `LocalCatalog` on a scripted server and the wheel: `server` says what it offers (`live`, `loaded`, `warm`) and lists its replies per call in the order they are asked (`models`, `loads`, `warms`; a reply spends `delayMs` of the wheel, or fails); `rows` are the registry's row keys; `steps` are `call` (`views`, `discovered`, `viewFor`, `warmed`), `together`, `settled` (`times` calls at once), `rediscover`, `advance` and `rows`. A step's result holds the wheel's time, the answers, the calls the server got during it (with the signal's state when each ended) and the times the catalog said it refreshed.

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
- Every message text is fixed and the same in both languages. A command that ends with none, by a thrown non-Error or a panic, fails with `the command crashed`. A result that does not fit the command's result shape fails with `the result does not fit the command`, in both languages.

Rust's adapter is awaited, as TypeScript's is. A command is `fn(&str, Value, RunContext) -> Option<Command>`, `Command` a boxed future; `None` is an unknown command. `receive`, `reply` and `request` are awaited; `emit`, `ask` and `steers` are plain calls. `RunContext` is `Clone`, `Send` and `Sync`, and holds the run's `CancelToken`: an interrupt frame and the input closing cancel it, besides ending the awaits on the inbox. `typed` holds a blocking command to its shapes, `typed_async` an awaited one. `serve` owns a multi-thread `tokio` runtime and reads frames on the calling thread; each run is a task, and at end of input every run is cancelled and awaited. A table of blocking work uses `serve_blocking` with `fn(&str, &Value, &BlockingContext)`, which offers the clock, the ids and `is_cancelled()`; one blocking arm in an awaited table wraps in `blocking(&context, ..)`. A panic in either kind fails the run with `the command crashed`. A run carries a cancel signal in both languages: Rust's `CancelToken`, TypeScript's `signal`.

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
- `Utf8Decoder` is WHATWG's: one U+FFFD per maximal invalid subpart, a split sequence held until it completes, a leading byte order mark dropped once, `end` turns a held tail into U+FFFD. Never the platform's `TextDecoder`.
- `hasControl` is `/\p{Cc}/u`: C0, DEL and C1. A lone surrogate is not one.
- `originOf` follows the WHATWG parser for the schemes it reads: lowercase scheme and host, default port dropped, credentials, path, query and fragment ignored, a trailing dot kept, IPv4 numbers and IPv6 literals written in canonical form. Hosts with non-ASCII text or percent escapes are not read.
- `encodePathSegment` is `encodeURIComponent`: `-_.!~*'()` and ASCII letters and digits kept, UTF-8, uppercase hex. A lone surrogate becomes U+FFFD first.

## Contract

A shape is Zod, written with `z` from `@nyaaai/contract/shape`, in `ts/contract/src/shapes/<set>.ts`. Its set lists the named ones: `shapeSet("<set>", { Name, ... })`. A named shape is a `$defs` entry and a Rust type. An unlisted object inside one gets a derived Rust name: parent plus key.

Pipeline: Zod, `schema/<set>.<form>.json`, one intermediate form (`scripts/contract/ir.ts`), one emitter per language (`scripts/contract/rust.ts`). TypeScript types are `z.infer`. Never edit a generated file.

To add a set:

1. Module with its `shapeSet`, exported in `ts/contract/package.json`.
2. Import and Rust target in `parity.ts`. `pub mod` in Rust.
3. `bun scripts/contract.ts --refresh`.
4. Round-trip cases for every named shape in `scripts/caseSources/contract.ts`. A test fails on a shape without one.

`samples` holds every shape to more than its hand-written cases. For each named shape it derives values from the exported schema: a full one, a minimal one, each branch, an unknown key, then one broken rule at a time. Both languages decode each through the project's `roundTrip` kind, and a difference fails the gate. A shape's values go out 2,000 a request, and none is dropped. It stores nothing.

Dialect. `dialectProblems` lists, at once and by shape path, what `contract` refuses about a shape; `contract.ts` runs it before it generates:

- Object, array, tuple of fixed places, record with text keys, nullable, union, discriminated union with a text tag.
- String, number, `z.int()`, boolean, null, unknown, one-value literal, enum of texts.
- Bounds: `min`, `max`, `gt`, `lt` on numbers; `min`, `max` on strings and arrays. String bounds count code points.
- No string format (`z.base64()`, `z.email()`, `z.uuid()`, `z.iso.datetime()`): write `matching`, or a `refined` hook over a primitive.
- No `.describe()` or `.meta()`: write a comment.
- A discriminated union holds objects, each with a required one-value text literal tag. A boolean tag is a plain union; an enum tag is one branch per value, each a one-value literal.
- A shape name is PascalCase. A property, hook, enum value, tag value and literal union branch each come to a Rust name: `a-b` and `a_b` come to one, and `.` to none. The empty text comes to `Empty` as an enum value or literal branch, and to none elsewhere. Two that come to one are refused; so is one that comes to none: name the shape, or use other values.
- An integer bound is whole. `matching` takes a string.
- Optional and default sit on an object property, outermost. Use `exactOptional`. No optional property holds a default.
- Recursion passes through a named shape.
- A tuple holds no rest, no optional place, no preprocess place, and at most 12 places.
- A named shape that is not an object or a union never names itself, through other such shapes either. A recursive shape is an object or a union.
- Hooks, ported by hand: `preprocessed(name, fix, schema)` maps the raw value before the decode, and never sees a missing one; `refined(name, schema, holds, message?)` judges the decoded value. Rust holds them in `<module>_hooks.rs`, snake case, beside a keyword set's `move_.rs` as `move_hooks.rs`: one function per name, serving one type. A transform is written as a preprocess.
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
- A consumer bundles the source itself. `contract`, `process`, `agentic`, `operator` and `ui` fit one CommonJS file: no top-level `await`, no `import.meta`, no asset import.
- A browser bundle may take `contract`, `ui` and `@nyaaai/agentic/noticeText`. They reach no platform module (`node:*`, `bun`) or global (`Bun`, `process`, `Buffer`). A new browser entry joins `BROWSER_ENTRIES` in the residue test, which holds both rules.

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
- Its `main` serves with `nyaaai_adapter::runtime::serve`. A `case:` command goes to `cases::dispatch::dispatch(kind, input, root)` inside `blocking`, the root being the current directory read once at start. `rust/cli/src/main.rs` is the example.
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
- `tokio` (Rust, `rt`, `sync`, `time`) is the one crate of the engine: it brings `pin-project-lite`. Engine code uses no `serde`. The test kit adds tokio `net` and `io-util` for the loopback.
- `reqwest` 0.13 (Rust, only behind `agentic`'s `http` feature, which is off): `default-features = false`, `rustls` on. TLS is rustls with the platform verifier and `aws-lc-rs`; no `native-tls`, no OpenSSL, no HTTP/2, no compression, no cookies, no `json`, no charset, no system proxy settings; the adapter's client takes no proxy from the environment and follows no redirect.
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
- `scripts/record.ts` refuses a recording that holds its key, and replaces the same ids by the same rules (`scripts/traceIds.ts`) inside each chunk; an id across a chunk edge fails the recording. It sees no other secret. Read a recorded trace before committing it.

## Text

- No em dash, smart quote or zero-width character. `symbols` fails on one, and on U+FEFF.
- In source, write `\u{2028}`, not the four-digit form: some editors decode it on save.
