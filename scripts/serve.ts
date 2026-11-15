// TypeScript case adapter.
// Run: bun scripts/serve.ts

import { type Command, command } from "@nyaaai/adapter/runtime";
import { serveStdio } from "@nyaaai/adapter/stdio";
import { kindsOf } from "@nyaaai/testkit/caseGroup";
import { CASE_PREFIX, runKind } from "./caseAdapter";
import { loadProject } from "./project";

////////////////////////////////
//  Main

if (import.meta.main) {
	const project = await loadProject();
	const kinds = [...kindsOf(project.groups)].map(([name, held]): [string, Command] => [
		`${CASE_PREFIX}${name}`,
		command(held.input, held.result, (input) => runKind(held, input) as never),
	]);
	const commands = new Map<string, Command>([...project.commands, ...kinds]);
	process.exit(await serveStdio(commands));
}
