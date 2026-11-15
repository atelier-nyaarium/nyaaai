# nyaaai

AI framework stuff for Nyaarium tools and workflows.

One design in two languages:

- `ts/`: TypeScript packages, taken as source through a git submodule.
- `rust/`: Rust crates and the `nyaaai` command line tool.
- `schema/`: the contract as JSON Schema, generated from Zod.
- `prompts/`: prompt texts both halves embed.
- `cases/`: cases both halves must pass.

Layout, commands and rules are in `AGENTS.md`.
