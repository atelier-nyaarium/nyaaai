# NyaaAI

Shared AI framework for Nyaarium tools. TypeScript and Rust twins, judged by one set of cases.

## Layout

- `ts/`: Bun workspace. Packages `contract`, `process`, `agentic`, `operator`, `ui`, `testkit`, scope `@nyaaai`.
- `rust/`: Cargo workspace. Crates `contract`, `process`, `agentic`, `operator`, `testkit`, `cli`.
- `cases/`: language-neutral cases. Generated from `scripts/caseSources/`, committed.
  - A case is data. `paths` cases also name a folder tree, built in a temp folder by each language.
- `scripts/`: gate, case runner, crate check, snapshot, consumer script.

## Commands

Run from the repository root.

```sh
bun scripts/gate.ts               # everything below
bun scripts/gate.ts cases tsc     # only the named parts
bun scripts/cases.ts              # case files current, Rust agrees
bun scripts/cases.ts --refresh    # rewrite case files from TypeScript
bun scripts/crates.ts             # crate ages and advisories
bun scripts/snapshot.ts           # commit the working tree for a consumer, print {ref, sha, subject}
bun scripts/realSessions.ts       # both languages on this machine's real session files; prints counts only
```

Gate parts: `link`, `biome`, `tsc`, `test`, `fmt`, `clippy`, `cargo-test`, `crates`, `cases`, `symbols`, `demo`.

`link` makes `node_modules/@nyaaai` here, as a consumer's does. Scripts import packages by name, so the gate reads the same exports a consumer reads.

`cargo` comes from `PATH`, else `~/.cargo/bin`. `rust-toolchain.toml` pins the toolchain.
`demo` runs when `../nyaaai-demo` exists, or `NYAAAI_DEMO_DIR`.

Command line tool:

```sh
cargo run --manifest-path rust/Cargo.toml -p nyaaai-cli -- cut trim --source <file> --out <folder>
```

`cut` only reads its source. Exit 0 done, 2 refused, 1 failed.

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
- The case runner is `nyaaai-cases`, behind the `cases` feature. The `nyaaai` tool holds no test code.
- No build output, no large corpus.
- Consumers import TypeScript source through a submodule. No `node_modules` inside one.
- A package lists what it needs in `dependencies`. The consumer pins each at that version.

## Decode rules

Same in both languages. `parseJson` and `parse_json` apply them.

- Lone surrogate escape becomes U+FFFD.
- Nesting past 1024 containers is refused.
- Rust spells JSON as `JSON.stringify` does: ECMAScript key order and number text.
- A file row is decoded text. Bad bytes become U+FFFD, in a kept row too.

## Dependencies

- Seven days old, or named by an advisory. `ts/bunfig.toml` enforces the age for Bun.
- `scripts/crates.ts` checks `Cargo.lock` against crates.io publish dates and OSV advisories.
  - `crate-ages.json` beside a lock holds the dates.
  - `crate-exemptions.json` beside a lock lists reviewed exceptions.

## Public repository

- No key, account id, real transcript or personal content. Test data is synthetic.
- No license file.
- Scan fixtures, cases and prompts before a push.

## Text

- No em dash, smart quote or zero-width character. `symbols` fails on one.
- In source, write `\u{2028}`, not the four-digit form: some editors decode it on save.
