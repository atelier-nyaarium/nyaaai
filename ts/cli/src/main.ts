// CLI entry point. Run: bun ts/cli/src/main.ts serve

import { serve } from "./adapter";
import { COMMANDS } from "./commands";

////////////////////////////////
//  Main

if (import.meta.main) {
	const [name] = process.argv.slice(2);
	if (name !== "serve") {
		console.error("usage:\n  nyaaai serve");
		process.exit(1);
	}
	const write = (line: string) => void process.stdout.write(`${line}\n`);
	process.exit(await serve({ input: Bun.stdin.stream(), write, commands: COMMANDS }));
}
